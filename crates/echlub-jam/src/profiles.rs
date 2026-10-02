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
pub enum Access {
    #[serde(rename = "fiber-wired")]
    FiberWired,
    #[serde(rename = "cable-wired")]
    CableWired,
    #[serde(rename = "wifi")]
    Wifi,
    #[serde(rename = "mobile-5g")]
    Mobile5g,
    /// 5G standalone core (lower and steadier radio latency than NSA).
    #[serde(rename = "mobile-5g-sa")]
    Mobile5gSa,
    #[serde(rename = "mobile-4g")]
    Mobile4g,
    /// Laptop on a phone's 4G hotspot over Wi-Fi.
    #[serde(rename = "hotspot-4g-wifi")]
    Hotspot4gWifi,
    /// Laptop on a phone's 4G connection via USB tethering.
    #[serde(rename = "hotspot-4g-usb")]
    Hotspot4gUsb,
    #[serde(rename = "hotspot-5g-wifi")]
    Hotspot5gWifi,
    #[serde(rename = "hotspot-5g-usb")]
    Hotspot5gUsb,
}

impl Access {
    pub const ALL: [Access; 10] = [
        Access::FiberWired,
        Access::CableWired,
        Access::Wifi,
        Access::Mobile5g,
        Access::Mobile5gSa,
        Access::Mobile4g,
        Access::Hotspot4gWifi,
        Access::Hotspot4gUsb,
        Access::Hotspot5gWifi,
        Access::Hotspot5gUsb,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Access::FiberWired => "fiber-wired",
            Access::CableWired => "cable-wired",
            Access::Wifi => "wifi",
            Access::Mobile5g => "mobile-5g",
            Access::Mobile5gSa => "mobile-5g-sa",
            Access::Mobile4g => "mobile-4g",
            Access::Hotspot4gWifi => "hotspot-4g-wifi",
            Access::Hotspot4gUsb => "hotspot-4g-usb",
            Access::Hotspot5gWifi => "hotspot-5g-wifi",
            Access::Hotspot5gUsb => "hotspot-5g-usb",
        }
    }

    pub fn parse(s: &str) -> Option<Access> {
        Access::ALL.into_iter().find(|a| a.name() == s)
    }

    /// Assumed last-mile segment (home network + access network).
    ///
    /// A hotspot is the phone's cellular link plus one local hop to the
    /// laptop. It does not improve the radio link; it lets the musician use
    /// a computer and audio interface instead of the phone's audio stack.
    pub fn last_mile(self) -> LinkProfile {
        let cellular = |base, jitter, loss| LinkProfile::new("cellular", base, jitter, loss);
        let lte = cellular(22.0, 10.0, 0.01);
        let nr = cellular(10.0, 5.0, 0.005);
        // Phone Wi-Fi access points add power-save scheduling jitter.
        let wifi_hop = LinkProfile::new("phone-wifi-hop", 2.0, 2.5, 0.003);
        let usb_hop = LinkProfile::new("usb-tether-hop", 0.5, 0.2, 0.0);
        let mut link = match self {
            Access::FiberWired => LinkProfile::new("", 1.0, 0.3, 0.000_5),
            Access::CableWired => LinkProfile::new("", 4.0, 1.5, 0.001),
            Access::Wifi => LinkProfile::new("", 2.0, 3.0, 0.005),
            Access::Mobile5g => nr,
            Access::Mobile5gSa => cellular(6.0, 2.0, 0.002),
            Access::Mobile4g => lte,
            Access::Hotspot4gWifi => lte.then(&wifi_hop),
            Access::Hotspot4gUsb => lte.then(&usb_hop),
            Access::Hotspot5gWifi => nr.then(&wifi_hop),
            Access::Hotspot5gUsb => nr.then(&usb_hop),
        };
        link.name = self.name().to_string();
        link
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
    pub const PRESETS: [&'static str; 12] = [
        "native-interface",
        "native-builtin",
        "phone-app",
        "phone-ios-app",
        "phone-ios-app-128",
        "phone-ios-bluetooth",
        "phone-android-low-latency",
        "phone-android-generic",
        "phone-interface",
        "phone-browser",
        "browser-worklet-tuned",
        "browser-webrtc-default",
    ];

    pub fn preset(name: &str) -> Option<EndpointProfile> {
        let (capture, playback, conv, codec, proc_, floor) = match name {
            // USB audio interface, 64-sample buffers, wired headphones.
            "native-interface" => (1.33, 1.33, 0.5, 0.0, 0.0, 0.0),
            // Laptop built-in audio, 256-sample buffers.
            "native-builtin" => (5.33, 5.33, 1.0, 0.0, 0.0, 0.0),
            // Native app on a phone using its own audio stack (wired
            // headset). Varies widely by OS and model.
            "phone-app" => (10.0, 15.0, 1.0, 0.0, 1.0, 0.0),
            // iOS app on Core Audio, 256-frame IO buffer, `.measurement`
            // mode (no voice processing), wired headphones (USB-C/Lightning
            // DAC included in converter time).
            "phone-ios-app" => (5.33, 5.33, 1.0, 0.0, 0.5, 0.0),
            // Same with a 128-frame IO buffer, if the device grants it.
            "phone-ios-app-128" => (2.67, 2.67, 1.0, 0.0, 0.5, 0.0),
            // Same app monitoring over Bluetooth earbuds (AAC/SBC codec path).
            "phone-ios-bluetooth" => (5.33, 150.0, 1.0, 0.0, 0.5, 0.0),
            // Android app on the AAudio/Oboe low-latency path (device must
            // support it), wired headset.
            "phone-android-low-latency" => (5.0, 10.0, 1.0, 0.0, 0.5, 0.0),
            // Android app on a device without a fast audio path.
            "phone-android-generic" => (20.0, 30.0, 1.0, 0.0, 1.0, 0.0),
            // Phone app with a class-compliant USB audio interface.
            "phone-interface" => (2.67, 2.67, 0.5, 0.0, 0.5, 0.0),
            // Mobile browser with AudioWorklet + custom datagram transport.
            "phone-browser" => (20.0, 30.0, 1.0, 0.0, 2.0, 0.0),
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
    fn access_serializes_as_cli_name() {
        for a in Access::ALL {
            let json = serde_json::to_string(&a).unwrap();
            assert_eq!(json, format!("\"{}\"", a.name()));
        }
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
    fn hotspot_adds_to_cellular_and_usb_beats_wifi() {
        let lte = Access::Mobile4g.last_mile();
        let usb = Access::Hotspot4gUsb.last_mile();
        let wifi = Access::Hotspot4gWifi.last_mile();
        assert!(usb.expected_ms() > lte.expected_ms());
        assert!(wifi.p99_jitter_ms() > usb.p99_jitter_ms());
        assert_eq!(usb.name, "hotspot-4g-usb");
    }

    #[test]
    fn wired_fiber_beats_mobile() {
        let fiber = relay_link(Site::Taipei, Access::FiberWired, false);
        let mobile = relay_link(Site::Taipei, Access::Mobile4g, false);
        assert!(fiber.expected_ms() < mobile.expected_ms());
        assert_eq!(fiber.name, "taipei/fiber-wired");
    }
}
