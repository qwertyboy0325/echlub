//! Scenario definitions: who plays what, where, on which network and device.

use echlub_jam::budget::{PeerSetup, Topology};
use echlub_jam::pipeline::{self, PipelineConfig, PipelineReport};
use echlub_jam::profiles::{relay_link, Access, EndpointProfile, Site};
use echlub_musician_sim::ensemble::{
    simulate, EnsembleConfig, EnsembleReport, Thresholds, Verdict,
};
use echlub_musician_sim::{MusicianParams, Role};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerSpec {
    pub role: Role,
    pub site: Site,
    pub access: Access,
    /// Device override; the scenario-wide endpoint applies when `None`.
    pub endpoint: Option<String>,
    /// The band keeps time without locking to this player (slow link).
    pub follower: bool,
    /// Per-player latency compensation override (see `MusicianParams`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compensation: Option<f64>,
}

impl PlayerSpec {
    /// Parse `role@site/access[/endpoint][!follow|!ahead]`, e.g.
    /// `drums@taipei/fiber-wired` or `vocals@kaohsiung/mobile-4g/phone-app!follow`.
    pub fn parse(s: &str) -> Result<PlayerSpec, String> {
        let bad = || format!("expected role@site/access[/endpoint][!follow|!ahead]: {s}");
        let (body, follower, ahead) = if let Some(b) = s.strip_suffix("!ahead") {
            (b, true, true)
        } else if let Some(b) = s.strip_suffix("!follow") {
            (b, true, false)
        } else {
            (s, false, false)
        };
        let (role, rest) = body.split_once('@').ok_or_else(bad)?;
        let mut parts = rest.split('/');
        let site = parts.next().ok_or_else(bad)?;
        let access = parts.next().ok_or_else(bad)?;
        let endpoint = parts.next().map(str::to_string);
        if parts.next().is_some() {
            return Err(bad());
        }
        if let Some(e) = &endpoint {
            EndpointProfile::preset(e).ok_or(format!("unknown endpoint {e}"))?;
        }
        Ok(PlayerSpec {
            role: Role::parse(role).ok_or(format!("unknown role {role}"))?,
            site: Site::parse(site).ok_or(format!("unknown site {site}"))?,
            access: Access::parse(access).ok_or(format!("unknown access {access}"))?,
            endpoint,
            follower,
            compensation: ahead.then_some(2.0),
        })
    }

    pub fn label(&self) -> String {
        let mut l = format!(
            "{}@{}/{}",
            self.role.name(),
            self.site.name(),
            self.access.name()
        );
        if let Some(e) = &self.endpoint {
            l.push('/');
            l.push_str(e);
        }
        if self.compensation == Some(2.0) && self.follower {
            l.push_str("!ahead");
        } else if self.follower {
            l.push_str("!follow");
        }
        l
    }
}

pub const PRESETS: [(&str, &str); 11] = [
    (
        "taiwan-duo",
        "drums@taipei/fiber-wired,bass@kaohsiung/fiber-wired",
    ),
    (
        "taiwan-trio",
        "drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@kaohsiung/cable-wired",
    ),
    (
        "taiwan-rock-4",
        "drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/cable-wired,vocals@hualien/wifi",
    ),
    (
        "taiwan-rock-4-wired",
        "drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@tainan/fiber-wired,vocals@kaohsiung/fiber-wired",
    ),
    (
        "taiwan-rock-4-mobile",
        "drums@taipei/fiber-wired,bass@taichung/fiber-wired,guitar@kaohsiung/fiber-wired,vocals@tainan/mobile-4g",
    ),
    (
        "iphone-duo-ethernet",
        "drums@taipei/fiber-wired/phone-ios-app,bass@kaohsiung/fiber-wired/phone-ios-app",
    ),
    (
        "iphone-rock-4-ethernet",
        "drums@taipei/fiber-wired/phone-ios-app,bass@taichung/fiber-wired/phone-ios-app,guitar@tainan/fiber-wired/phone-ios-app,vocals@kaohsiung/fiber-wired/phone-ios-app",
    ),
    (
        "iphone-rock-4-wifi",
        "drums@taipei/wifi/phone-ios-app,bass@taichung/wifi/phone-ios-app,guitar@tainan/wifi/phone-ios-app,vocals@kaohsiung/wifi/phone-ios-app",
    ),
    (
        "iphone-rock-4-5g-sa",
        "drums@taipei/mobile-5g-sa/phone-ios-app,bass@taichung/mobile-5g-sa/phone-ios-app,guitar@tainan/mobile-5g-sa/phone-ios-app,vocals@kaohsiung/mobile-5g-sa/phone-ios-app",
    ),
    (
        "iphone-rock-4-5g",
        "drums@taipei/mobile-5g/phone-ios-app,bass@taichung/mobile-5g/phone-ios-app,guitar@tainan/mobile-5g/phone-ios-app,vocals@kaohsiung/mobile-5g/phone-ios-app",
    ),
    (
        "iphone-rock-4-4g",
        "drums@taipei/mobile-4g/phone-ios-app,bass@taichung/mobile-4g/phone-ios-app,guitar@tainan/mobile-4g/phone-ios-app,vocals@kaohsiung/mobile-4g/phone-ios-app",
    ),
];

pub fn preset(name: &str) -> Option<Vec<PlayerSpec>> {
    PRESETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, spec)| parse_players(spec).expect("presets are valid"))
}

pub fn parse_players(list: &str) -> Result<Vec<PlayerSpec>, String> {
    let players: Vec<_> = list
        .split(',')
        .map(|s| PlayerSpec::parse(s.trim()))
        .collect::<Result<_, _>>()?;
    if !(2..=4).contains(&players.len()) {
        return Err("scenario needs 2-4 players".into());
    }
    Ok(players)
}

#[derive(Debug, Clone)]
pub struct ScenarioConfig {
    pub name: String,
    pub players: Vec<PlayerSpec>,
    pub endpoint: EndpointProfile,
    pub cross_isp: bool,
    pub topology: Topology,
    pub bpm: f64,
    pub bars: usize,
    pub seeds: u64,
    pub duration_ms: f64,
    pub compensation: f64,
    /// Jitter-buffer coverage (fraction of packets caught in time).
    pub coverage: f64,
    /// Copies of each audio frame sent (1 = none).
    pub redundancy: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioReport {
    pub evidence_level: &'static str,
    pub disclaimer: &'static str,
    pub name: String,
    pub players: Vec<PlayerSpec>,
    pub endpoint: EndpointProfile,
    pub cross_isp: bool,
    pub bpm: f64,
    pub bars: usize,
    pub pipeline: PipelineReport,
    /// Latency matrix (p50 mouth-to-ear) fed to the musicians.
    pub latency_matrix_ms: Vec<Vec<f64>>,
    pub ensemble_runs: u64,
    pub tight_fraction: f64,
    pub playable_or_better_fraction: f64,
    pub mean_tempo_drift_pct: f64,
    pub mean_worst_heard_rms_ms: f64,
    pub coverage: f64,
    pub redundancy: usize,
    /// Worst path's dropouts per minute.
    pub worst_dropouts_per_min: f64,
    /// Worst path's audible (≥ 8 ms) dropouts per minute.
    pub worst_audible_dropouts_per_min: f64,
    pub worst_concealed_pct: f64,
    pub thresholds: Thresholds,
    /// First seed's run, used for audio rendering.
    pub example_run: EnsembleReport,
    #[serde(skip)]
    pub runs: Vec<EnsembleReport>,
}

pub fn peer_setups(cfg: &ScenarioConfig) -> Vec<PeerSetup> {
    cfg.players
        .iter()
        .map(|p| {
            let endpoint = match &p.endpoint {
                Some(e) => EndpointProfile::preset(e).expect("validated at parse"),
                None => cfg.endpoint.clone(),
            };
            PeerSetup::with_coverage(
                p.label(),
                relay_link(p.site, p.access, cfg.cross_isp),
                endpoint,
                cfg.coverage,
            )
            .with_redundancy(cfg.redundancy)
        })
        .collect()
}

pub fn ensemble_config(cfg: &ScenarioConfig, matrix: &[Vec<f64>], seed: u64) -> EnsembleConfig {
    EnsembleConfig {
        names: cfg.players.iter().map(|p| p.label()).collect(),
        players: cfg
            .players
            .iter()
            .map(|p| {
                let mut m = MusicianParams::rock_default(p.role);
                m.latency_compensation = p.compensation.unwrap_or(cfg.compensation);
                if p.follower {
                    m = m.as_follower();
                }
                m
            })
            .collect(),
        latency_ms: matrix.to_vec(),
        bpm: cfg.bpm,
        bars: cfg.bars,
        seed,
        thresholds: Thresholds::default(),
    }
}

pub fn run(cfg: &ScenarioConfig) -> ScenarioReport {
    let pipeline = pipeline::run(&PipelineConfig {
        peers: peer_setups(cfg),
        duration_ms: cfg.duration_ms,
        seed: 1,
        topology: cfg.topology,
    });
    let matrix = pipeline.latency_matrix_p50();
    let runs: Vec<EnsembleReport> = (0..cfg.seeds.max(1))
        .map(|seed| simulate(&ensemble_config(cfg, &matrix, seed)))
        .collect();
    let r = runs.len() as f64;
    let frac =
        |pred: fn(&EnsembleReport) -> bool| runs.iter().filter(|x| pred(x)).count() as f64 / r;
    ScenarioReport {
        evidence_level: "Simulation",
        disclaimer: "All network, device, and musician parameters are ASSUMED planning values. This is not a measurement and makes no latency claim about real networks.",
        name: cfg.name.clone(),
        players: cfg.players.clone(),
        endpoint: cfg.endpoint.clone(),
        cross_isp: cfg.cross_isp,
        bpm: cfg.bpm,
        bars: cfg.bars,
        latency_matrix_ms: matrix,
        ensemble_runs: runs.len() as u64,
        tight_fraction: frac(|x| x.verdict == Verdict::Tight),
        playable_or_better_fraction: frac(|x| x.verdict != Verdict::Struggling),
        mean_tempo_drift_pct: runs.iter().map(|x| x.tempo_drift_pct).sum::<f64>() / r,
        mean_worst_heard_rms_ms: runs.iter().map(|x| x.worst_heard_rms_ms).sum::<f64>() / r,
        coverage: cfg.coverage,
        redundancy: cfg.redundancy,
        worst_audible_dropouts_per_min: pipeline
            .pairs
            .iter()
            .map(|p| p.audible_dropouts_per_min)
            .fold(0.0, f64::max),
        worst_dropouts_per_min: pipeline
            .pairs
            .iter()
            .map(|p| p.dropouts_per_min)
            .fold(0.0, f64::max),
        worst_concealed_pct: pipeline
            .pairs
            .iter()
            .map(|p| p.concealed_pct)
            .fold(0.0, f64::max),
        thresholds: Thresholds::default(),
        example_run: runs.first().cloned().expect("at least one run"),
        runs,
        pipeline,
    }
}

/// Audible-dropout tolerance (hypotheses, per minute on the worst path).
pub const CLEAN_DROPOUTS_PER_MIN: f64 = 1.0;
pub const NOTICEABLE_DROPOUTS_PER_MIN: f64 = 6.0;

/// Raw PCM stream rate on the wire per copy: (19 B header + 256 B PCM +
/// 28 B IP/UDP) x 375 packets/s.
pub const STREAM_KBPS: f64 = 909.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StudyOutcome {
    /// Whole band playable, dropouts at most noticeable.
    FullBand,
    /// With the click played early, the band hears the mobile player on the
    /// beat; the mobile player hears the band about a round trip late and
    /// must play to the click rather than to the band.
    OnTimeViaClick,
    /// Wired players are fine; the mobile player keeps up as a follower but
    /// sounds late to them.
    CoreOkMobileBehind,
    NotUsable,
}

/// How the band organises around the player on the slow link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Arrangement {
    /// Everyone listens to everyone.
    Normal,
    /// The band ignores the mobile player's timing; the mobile player follows.
    Follower,
    /// Follower, plus a shared click track that the app plays early for the
    /// mobile player by their measured round trip, so they can play ahead of
    /// what they hear and arrive on time at the band. Requires clock-synced
    /// clicks; whether players can do this comfortably is unverified.
    FollowerClickAhead,
}

impl Arrangement {
    pub const ALL: [Arrangement; 3] = [
        Arrangement::Normal,
        Arrangement::Follower,
        Arrangement::FollowerClickAhead,
    ];
}

/// One remedy combination for a band with one player on a mobile link.
#[derive(Debug, Clone, Serialize)]
pub struct MobileStudyRow {
    pub mobile_role: Role,
    pub access: Access,
    pub endpoint: String,
    pub coverage: f64,
    pub redundancy: usize,
    pub arrangement: Arrangement,
    /// Worst p50 mouth-to-ear to or from the mobile player.
    pub mobile_path_ms: f64,
    /// Worst p50 among the wired players.
    pub core_path_ms: f64,
    /// Worst audible dropouts/min on any path touching the mobile player.
    pub mobile_audible_dropouts_per_min: f64,
    pub mean_tempo_drift_pct: f64,
    /// Whole band playable (every listener).
    pub band_playable_fraction: f64,
    /// Wired core playable (drift and heard RMS among wired players only).
    pub core_playable_fraction: f64,
    /// How late the mobile player sounds to the wired players, on average.
    pub mobile_sounds_late_ms: f64,
    /// How late the band sounds to the mobile player, on average.
    pub band_sounds_late_to_mobile_ms: f64,
    pub mobile_uplink_kbps: f64,
    pub mobile_downlink_kbps: f64,
    pub outcome: StudyOutcome,
}

pub const MOBILE_VARIANTS: [(Access, &str); 9] = [
    (Access::FiberWired, "native-interface"),
    (Access::Mobile4g, "phone-app"),
    (Access::Hotspot4gWifi, "native-interface"),
    (Access::Hotspot4gUsb, "native-interface"),
    (Access::Mobile5g, "phone-app"),
    (Access::Hotspot5gWifi, "native-interface"),
    (Access::Hotspot5gUsb, "native-interface"),
    (Access::Mobile5gSa, "phone-app"),
    (Access::Mobile5gSa, "native-interface"),
];

pub const COVERAGES: [f64; 3] = [0.99, 0.95, 0.9];

/// Wired 4-piece where `mobile_role` is on a mobile variant, across buffer
/// coverage, redundancy, and band arrangement.
pub fn mobile_study(mobile_role: Role, seeds: u64, duration_ms: f64) -> Vec<MobileStudyRow> {
    let base = preset("taiwan-rock-4-wired").expect("preset");
    let th = Thresholds::default();
    let mut rows = Vec::new();
    for (access, endpoint) in MOBILE_VARIANTS {
        for redundancy in [1, 2] {
            for coverage in COVERAGES {
                for arrangement in Arrangement::ALL {
                    let players: Vec<PlayerSpec> = base
                        .iter()
                        .cloned()
                        .map(|mut p| {
                            if p.role == mobile_role {
                                p.access = access;
                                p.endpoint = Some(endpoint.to_string());
                                p.follower = arrangement != Arrangement::Normal;
                                if arrangement == Arrangement::FollowerClickAhead {
                                    // Aim ahead of what is heard by incoming +
                                    // outgoing latency (≈ 2 × incoming).
                                    p.compensation = Some(2.0);
                                }
                            }
                            p
                        })
                        .collect();
                    let n = players.len();
                    let mobile = players
                        .iter()
                        .position(|p| p.role == mobile_role)
                        .expect("role in band");
                    let cfg = ScenarioConfig {
                        name: "mobile-study".into(),
                        players,
                        endpoint: EndpointProfile::preset("native-interface").expect("preset"),
                        cross_isp: false,
                        topology: Topology::Forward,
                        bpm: 120.0,
                        bars: 32,
                        seeds,
                        duration_ms,
                        compensation: 0.0,
                        coverage,
                        redundancy,
                    };
                    let r = run(&cfg);
                    let m = &r.latency_matrix_ms;
                    let mut mobile_path = 0.0f64;
                    let mut core_path = 0.0f64;
                    for (i, row) in m.iter().enumerate() {
                        for (j, lat) in row.iter().enumerate().filter(|(j, _)| *j != i) {
                            if i == mobile || j == mobile {
                                mobile_path = mobile_path.max(*lat);
                            } else {
                                core_path = core_path.max(*lat);
                            }
                        }
                    }
                    let mobile_name = &r.pipeline.peers[mobile].name;
                    let mobile_audible = r
                        .pipeline
                        .pairs
                        .iter()
                        .filter(|p| &p.from == mobile_name || &p.to == mobile_name)
                        .map(|p| p.audible_dropouts_per_min)
                        .fold(0.0, f64::max);
                    let runs = r.runs.len().max(1) as f64;
                    let core_ok = |x: &EnsembleReport| {
                        !x.causality_warning
                            && x.tempo_drift_pct.abs() <= th.playable_drift_pct
                            && x.worst_heard_rms_excluding(&[mobile]) <= th.playable_heard_rms_ms
                    };
                    let core_playable = r.runs.iter().filter(|x| core_ok(x)).count() as f64 / runs;
                    let mobile_late = r
                        .runs
                        .iter()
                        .map(|x| {
                            (0..n)
                                .filter(|t| *t != mobile)
                                .map(|t| x.heard_offset_ms[mobile][t])
                                .sum::<f64>()
                                / (n - 1) as f64
                        })
                        .sum::<f64>()
                        / runs;
                    let band_late = r
                        .runs
                        .iter()
                        .map(|x| {
                            (0..n)
                                .filter(|f| *f != mobile)
                                .map(|f| x.heard_offset_ms[f][mobile])
                                .sum::<f64>()
                                / (n - 1) as f64
                        })
                        .sum::<f64>()
                        / runs;
                    let dropouts_ok = mobile_audible <= NOTICEABLE_DROPOUTS_PER_MIN;
                    let outcome = if r.playable_or_better_fraction >= 0.5 && dropouts_ok {
                        StudyOutcome::FullBand
                    } else if arrangement == Arrangement::FollowerClickAhead
                        && core_playable >= 0.5
                        && mobile_late.abs() <= th.tight_heard_rms_ms
                        && dropouts_ok
                    {
                        StudyOutcome::OnTimeViaClick
                    } else if core_playable >= 0.5 && dropouts_ok {
                        StudyOutcome::CoreOkMobileBehind
                    } else {
                        StudyOutcome::NotUsable
                    };
                    rows.push(MobileStudyRow {
                        mobile_role,
                        access,
                        endpoint: endpoint.to_string(),
                        coverage,
                        redundancy,
                        arrangement,
                        mobile_path_ms: mobile_path,
                        core_path_ms: core_path,
                        mobile_audible_dropouts_per_min: mobile_audible,
                        mean_tempo_drift_pct: r.mean_tempo_drift_pct,
                        band_playable_fraction: r.playable_or_better_fraction,
                        core_playable_fraction: core_playable,
                        mobile_sounds_late_ms: mobile_late,
                        band_sounds_late_to_mobile_ms: band_late,
                        mobile_uplink_kbps: STREAM_KBPS * redundancy as f64,
                        mobile_downlink_kbps: STREAM_KBPS * redundancy as f64 * (n - 1) as f64,
                        outcome,
                    });
                }
            }
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_parse() {
        for (name, _) in PRESETS {
            assert!(preset(name).is_some(), "{name}");
        }
    }

    #[test]
    fn rejects_bad_specs() {
        assert!(PlayerSpec::parse("drums").is_err());
        assert!(PlayerSpec::parse("kazoo@taipei/wifi").is_err());
        assert!(parse_players("drums@taipei/wifi").is_err());
    }

    #[test]
    fn parses_endpoint_and_follower() {
        let p = PlayerSpec::parse("vocals@kaohsiung/mobile-4g/phone-app!follow").unwrap();
        assert_eq!(p.access, Access::Mobile4g);
        assert_eq!(p.endpoint.as_deref(), Some("phone-app"));
        assert!(p.follower);
        assert_eq!(p.compensation, None);
        assert_eq!(PlayerSpec::parse(&p.label()).unwrap(), p);
        let a = PlayerSpec::parse("drums@taipei/hotspot-4g-usb!ahead").unwrap();
        assert!(a.follower);
        assert_eq!(a.compensation, Some(2.0));
        assert_eq!(PlayerSpec::parse(&a.label()).unwrap(), a);
        assert!(PlayerSpec::parse("drums@taipei/wifi/no-such-device").is_err());
    }
}
