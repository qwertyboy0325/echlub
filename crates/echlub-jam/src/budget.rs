//! Analytic one-way latency budget for the star (relay) topology.

use serde::Serialize;

use crate::frame_ms;
use crate::jitter::recommended_depth;
use crate::netsim::LinkProfile;
use crate::profiles::EndpointProfile;

/// One musician's setup: network path to the relay, device stack, and the
/// jitter-buffer depth used for this peer (at the relay for its uplink and at
/// its own client for the downlink).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PeerSetup {
    pub name: String,
    pub link: LinkProfile,
    pub endpoint: EndpointProfile,
    pub jitter_depth: usize,
}

impl PeerSetup {
    /// Depth chosen to cover the link's p99 jitter.
    pub fn auto(name: impl Into<String>, link: LinkProfile, endpoint: EndpointProfile) -> Self {
        let jitter_depth = recommended_depth(link.p99_jitter_ms(), frame_ms());
        Self {
            name: name.into(),
            link,
            endpoint,
            jitter_depth,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BudgetItem {
    pub label: String,
    pub ms: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PathBudget {
    pub from: String,
    pub to: String,
    pub items: Vec<BudgetItem>,
    pub total_ms: f64,
}

/// Latency outside the network pipeline: capture/playback buffers,
/// converters, codec framing, receive processing, and any stack jitter floor.
pub fn device_ms(src: &PeerSetup, dst: &PeerSetup) -> f64 {
    device_items(src, dst).iter().map(|i| i.ms).sum()
}

fn device_items(src: &PeerSetup, dst: &PeerSetup) -> Vec<BudgetItem> {
    let jitter_floor_surplus =
        (dst.endpoint.min_jitter_ms - dst.jitter_depth as f64 * frame_ms()).max(0.0);
    vec![
        item("capture buffer", src.endpoint.capture_ms),
        item("ADC", src.endpoint.converter_ms),
        item("codec framing", src.endpoint.codec_extra_ms),
        item("receive processing", dst.endpoint.processing_ms),
        item("stack jitter floor", jitter_floor_surplus),
        item("playback buffer", dst.endpoint.playback_ms),
        item("DAC", dst.endpoint.converter_ms),
    ]
}

fn item(label: &str, ms: f64) -> BudgetItem {
    BudgetItem {
        label: label.to_string(),
        ms,
    }
}

/// How the relay handles audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Topology {
    /// Relay jitter-buffers every peer on its own clock and sends one
    /// mix-minus per peer (two jitter buffers per path, constant downlink).
    Mix,
    /// Relay forwards each packet immediately; clients jitter-buffer each
    /// source and mix locally (one jitter buffer per path, N-1 downstreams).
    Forward,
}

impl Topology {
    pub fn parse(s: &str) -> Option<Topology> {
        match s {
            "mix" => Some(Topology::Mix),
            "forward" => Some(Topology::Forward),
            _ => None,
        }
    }
}

/// Client jitter depth for one source in [`Topology::Forward`]: covers the
/// combined uplink + downlink jitter.
pub fn forward_depth(src: &PeerSetup, dst: &PeerSetup) -> usize {
    recommended_depth(src.link.then(&dst.link).p99_jitter_ms(), frame_ms())
}

/// Expected mouth-to-ear latency from `src` to `dst` through the relay.
pub fn star_path(src: &PeerSetup, dst: &PeerSetup, topology: Topology) -> PathBudget {
    let f = frame_ms();
    let mut items = device_items(src, dst);
    items.extend([
        item("frame packetization", f),
        item("uplink", src.link.expected_ms()),
    ]);
    match topology {
        Topology::Mix => items.extend([
            item("relay jitter buffer", src.jitter_depth as f64 * f),
            item("relay tick alignment", f / 2.0),
            item("downlink", dst.link.expected_ms()),
            item("client jitter buffer", dst.jitter_depth as f64 * f),
        ]),
        Topology::Forward => items.extend([
            item("downlink", dst.link.expected_ms()),
            item("client jitter buffer", forward_depth(src, dst) as f64 * f),
        ]),
    }
    items.push(item("client tick alignment", f / 2.0));
    let total_ms = items.iter().map(|i| i.ms).sum();
    PathBudget {
        from: src.name.clone(),
        to: dst.name.clone(),
        items,
        total_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{relay_link, Access, Site};

    fn peer(site: Site, access: Access, ep: &str) -> PeerSetup {
        PeerSetup::auto(
            site.name(),
            relay_link(site, access, false),
            EndpointProfile::preset(ep).unwrap(),
        )
    }

    #[test]
    fn totals_match_items() {
        let a = peer(Site::Taipei, Access::FiberWired, "native-interface");
        let b = peer(Site::Kaohsiung, Access::FiberWired, "native-interface");
        let budget = star_path(&a, &b, Topology::Mix);
        let sum: f64 = budget.items.iter().map(|i| i.ms).sum();
        assert!((sum - budget.total_ms).abs() < 1e-9);
    }

    #[test]
    fn webrtc_default_costs_more_than_native() {
        let native = star_path(
            &peer(Site::Taipei, Access::FiberWired, "native-interface"),
            &peer(Site::Kaohsiung, Access::FiberWired, "native-interface"),
            Topology::Mix,
        );
        let web = star_path(
            &peer(Site::Taipei, Access::FiberWired, "browser-webrtc-default"),
            &peer(
                Site::Kaohsiung,
                Access::FiberWired,
                "browser-webrtc-default",
            ),
            Topology::Mix,
        );
        assert!(web.total_ms > native.total_ms + 30.0);
    }

    #[test]
    fn forwarding_saves_a_jitter_buffer() {
        let a = peer(Site::Taipei, Access::FiberWired, "native-interface");
        let b = peer(Site::Kaohsiung, Access::FiberWired, "native-interface");
        let mix = star_path(&a, &b, Topology::Mix).total_ms;
        let fwd = star_path(&a, &b, Topology::Forward).total_ms;
        assert!(fwd < mix - frame_ms(), "{fwd} vs {mix}");
    }
}
