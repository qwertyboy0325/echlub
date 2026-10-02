//! Taiwan site, access-network, and endpoint presets.
//!
//! EVERY number here is an `Assumed` planning placeholder, not a measurement.
//! They exist so the simulator can be exercised end to end; replace them with
//! field data from the Taiwan network baseline work package before drawing
//! conclusions.

use serde::Serialize;

use crate::netsim::{LinkProfile, Provenance};

/// Where the relay is assumed to run: central Taiwan, minimising the worst
/// north/south/east path.
pub const RELAY_LOCATION: &str = "central-taiwan (assumed Taichung/Changhua)";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Site {
    Taipei,
    Taichung,
    Tainan,
    Kaohsiung,
    Hualien,
}

impl Site {
    pub const ALL: [Site; 5] = [
        Site::Taipei,
        Site::Taichung,
        Site::Tainan,
        Site::Kaohsiung,
        Site::Hualien,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Site::Taipei => "taipei",
            Site::Taichung => "taichung",
            Site::Tainan => "tainan",
            Site::Kaohsiung => "kaohsiung",
            Site::Hualien => "hualien",
        }
    }

    pub fn parse(s: &str) -> Option<Site> {
        Site::ALL.into_iter().find(|site| site.name() == s)
    }

    /// Assumed backbone segment from this site to the central relay.
    pub fn backbone(self) -> LinkProfile {
        let (base, jitter) = match self {
            Site::Taipei => (2.5, 0.3),
            Site::Taichung => (0.8, 0.2),
            Site::Tainan => (1.8, 0.3),
            Site::Kaohsiung => (2.3, 0.3),
            // East coast traffic often hairpins through the north.
            Site::Hualien => (4.5, 0.6),
        };
        LinkProfile::new(format!("backbone-{}", self.name()), base, jitter, 0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    FiberWired,
    CableWired,
    Wifi,
    Mobile5g,
    Mobile4g,
}

impl Access {
    pub const ALL: [Access; 5] = [
        Access::FiberWired,
        Access::CableWired,
        Access::Wifi,
        Access::Mobile5g,
        Access::Mobile4g,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Access::FiberWired => "fiber-wired",
            Access::CableWired => "cable-wired",
            Access::Wifi => "wifi",
            Access::Mobile5g => "mobile-5g",
            Access::Mobile4g => "mobile-4g",
        }
    }

    pub fn parse(s: &str) -> Option<Access> {
        Access::ALL.into_iter().find(|a| a.name() == s)
    }

    /// Assumed last-mile segment (home network + access network).
    pub fn last_mile(self) -> LinkProfile {
        let (base, jitter, loss) = match self {
            Access::FiberWired => (1.0, 0.3, 0.000_5),
            Access::CableWired => (4.0, 1.5, 0.001),
            Access::Wifi => (2.0, 3.0, 0.005),
            Access::Mobile5g => (10.0, 5.0, 0.005),
            Access::Mobile4g => (22.0, 10.0, 0.01),
        };
        LinkProfile::new(self.name(), base, jitter, loss)
    }
}

/// One-way path between a musician and the central relay.
pub fn relay_link(site: Site, access: Access, cross_isp: bool) -> LinkProfile {
    let mut link = access.last_mile().then(&site.backbone());
    if cross_isp {
        link = link.then(&LinkProfile::new("cross-isp", 2.0, 0.5, 0.0));
    }
    link.name = format!("{}/{}", site.name(), access.name());
    link
}

/// Audio-device and client-stack latency at one musician's end.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EndpointProfile {
    pub name: String,
    /// Input (capture) buffering before a frame can be sent.
    pub capture_ms: f64,
    /// Output (playback) buffering after a frame is mixed for the speaker.
    pub playback_ms: f64,
    /// Per-direction ADC or DAC conversion latency.
    pub converter_ms: f64,
    /// Extra encoder framing beyond the prototype's 128-sample frame.
    pub codec_extra_ms: f64,
    /// Receive-side processing (decoder, effects, voice processing).
    pub processing_ms: f64,
    /// Floor imposed by a stack-managed jitter buffer (e.g. WebRTC NetEQ).
    pub min_jitter_ms: f64,
    pub provenance: Provenance,
}

impl EndpointProfile {
    pub const PRESETS: [&'static str; 4] = [
        "native-interface",
        "native-builtin",
        "browser-worklet-tuned",
        "browser-webrtc-default",
    ];

    pub fn preset(name: &str) -> Option<EndpointProfile> {
        let (capture, playback, conv, codec, proc_, floor) = match name {
            // USB audio interface, 64-sample buffers, wired headphones.
            "native-interface" => (1.33, 1.33, 0.5, 0.0, 0.0, 0.0),
            // Laptop built-in audio, 256-sample buffers.
            "native-builtin" => (5.33, 5.33, 1.0, 0.0, 0.0, 0.0),
            // AudioWorklet + custom datagram transport, OS-dependent buffers.
            "browser-worklet-tuned" => (10.0, 10.0, 1.0, 0.0, 1.0, 0.0),
            // Stock getUserMedia + RTCPeerConnection audio.
            "browser-webrtc-default" => (10.0, 20.0, 1.0, 17.3, 10.0, 40.0),
            _ => return None,
        };
        Some(EndpointProfile {
            name: name.to_string(),
            capture_ms: capture,
            playback_ms: playback,
            converter_ms: conv,
            codec_extra_ms: codec,
            processing_ms: proc_,
            min_jitter_ms: floor,
            provenance: Provenance::Assumed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_resolves_and_is_assumed() {
        for name in EndpointProfile::PRESETS {
            let p = EndpointProfile::preset(name).unwrap();
            assert_eq!(p.provenance, Provenance::Assumed);
        }
        assert!(EndpointProfile::preset("nope").is_none());
    }

    #[test]
    fn names_round_trip() {
        for s in Site::ALL {
            assert_eq!(Site::parse(s.name()), Some(s));
        }
        for a in Access::ALL {
            assert_eq!(Access::parse(a.name()), Some(a));
        }
    }

    #[test]
    fn wired_fiber_beats_mobile() {
        let fiber = relay_link(Site::Taipei, Access::FiberWired, false);
        let mobile = relay_link(Site::Taipei, Access::Mobile4g, false);
        assert!(fiber.expected_ms() < mobile.expected_ms());
        assert_eq!(fiber.name, "taipei/fiber-wired");
    }
}
