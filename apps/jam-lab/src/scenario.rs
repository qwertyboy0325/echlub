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
}

impl PlayerSpec {
    /// Parse `role@site/access`, e.g. `drums@taipei/fiber-wired`.
    pub fn parse(s: &str) -> Result<PlayerSpec, String> {
        let (role, rest) = s
            .split_once('@')
            .ok_or(format!("expected role@site/access: {s}"))?;
        let (site, access) = rest
            .split_once('/')
            .ok_or(format!("expected role@site/access: {s}"))?;
        Ok(PlayerSpec {
            role: Role::parse(role).ok_or(format!("unknown role {role}"))?,
            site: Site::parse(site).ok_or(format!("unknown site {site}"))?,
            access: Access::parse(access).ok_or(format!("unknown access {access}"))?,
        })
    }

    pub fn label(&self) -> String {
        format!(
            "{}@{}/{}",
            self.role.name(),
            self.site.name(),
            self.access.name()
        )
    }
}

pub const PRESETS: [(&str, &str); 5] = [
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
    pub thresholds: Thresholds,
    /// First seed's run, used for audio rendering.
    pub example_run: EnsembleReport,
}

pub fn peer_setups(cfg: &ScenarioConfig) -> Vec<PeerSetup> {
    cfg.players
        .iter()
        .map(|p| {
            PeerSetup::auto(
                p.label(),
                relay_link(p.site, p.access, cfg.cross_isp),
                cfg.endpoint.clone(),
            )
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
                m.latency_compensation = cfg.compensation;
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
        thresholds: Thresholds::default(),
        example_run: runs.into_iter().next().expect("at least one run"),
        pipeline,
    }
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
}
