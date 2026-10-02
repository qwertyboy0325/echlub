use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

use echlub_jam::budget::{star_path, Topology};
use echlub_jam::jitter::recommended_depth;
use echlub_jam::netsim::LinkProfile;
use echlub_jam::profiles::{relay_link, Access, EndpointProfile};
use echlub_jam::wav::encode_wav;
use echlub_jam::{frame_ms, SAMPLE_RATE};
use echlub_jam_lab::bots::{run_band, BandConfig, BotMode};
use echlub_jam_lab::report::{
    band_markdown, budget_markdown, mobile_study_markdown, scenario_markdown, sweep_markdown,
};
use echlub_jam_lab::scenario::{self, parse_players, peer_setups, ScenarioConfig, PRESETS};
use echlub_musician_sim::ensemble::sweep_uniform_latency;
use echlub_musician_sim::synth::render_listener_mix;
use echlub_musician_sim::Role;
use serde::Serialize;

fn usage() -> ! {
    let presets: Vec<_> = PRESETS.iter().map(|(n, _)| *n).collect();
    eprintln!(
        "Usage: jam-lab <command> [options]
Commands:
  scenario  [--preset NAME | --players LIST] [--endpoint E] [--topology mix|forward] [--cross-isp]
            [--bpm 120] [--bars 32] [--seeds 20] [--compensation 0]
            [--coverage 0.99] [--redundancy 1] [--out DIR] [--wav]
  budget    [--preset NAME | --players LIST] [--endpoint E] [--topology mix|forward] [--cross-isp]
  sweep     [--max 80] [--step 5] [--bpm 120] [--bars 32] [--runs 30]
            [--compensation 0] [--out DIR]
  bots      [--relay HOST:PORT] [--players LIST | --count N] [--mode probe|music]
            [--topology mix|forward] [--seconds 10] [--impair] [--coverage 0.99] [--bpm 120] [--relay-depth 2] [--out DIR] [--wav]

  mobile-study [--role vocals] [--seeds 10] [--out DIR]

LIST is comma-separated role@site/access[/endpoint][!follow|!ahead], e.g.
drums@taipei/fiber-wired or vocals@kaohsiung/mobile-4g/phone-app!follow.
Roles: drums bass guitar vocals. Sites: taipei taichung tainan kaohsiung hualien.
Access: {}
Endpoints: {}
Presets: {}
All profiles are ASSUMED planning values; outputs are simulations.",
        Access::ALL.map(|a| a.name()).join(" "),
        EndpointProfile::PRESETS.join(" "),
        presets.join(" ")
    );
    process::exit(2);
}

struct Args {
    flags: HashMap<String, String>,
}

impl Args {
    fn parse(raw: &[String]) -> Args {
        let mut flags = HashMap::new();
        let mut i = 0;
        while i < raw.len() {
            let key = raw[i]
                .strip_prefix("--")
                .unwrap_or_else(|| usage())
                .to_string();
            match raw.get(i + 1).filter(|v| !v.starts_with("--")) {
                Some(v) => {
                    flags.insert(key, v.clone());
                    i += 2;
                }
                None => {
                    flags.insert(key, "true".into());
                    i += 1;
                }
            }
        }
        Args { flags }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.flags.get(key).map(String::as_str)
    }

    fn has(&self, key: &str) -> bool {
        self.flags.contains_key(key)
    }

    fn num<T: std::str::FromStr>(&self, key: &str, default: T) -> T {
        match self.get(key) {
            Some(v) => v
                .parse()
                .unwrap_or_else(|_| fail(&format!("bad --{key}: {v}"))),
            None => default,
        }
    }
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    process::exit(1);
}

fn main() {
    let raw: Vec<String> = env::args().collect();
    let Some(cmd) = raw.get(1) else { usage() };
    let args = Args::parse(&raw[2..]);
    match cmd.as_str() {
        "scenario" => cmd_scenario(&args),
        "budget" => cmd_budget(&args),
        "sweep" => cmd_sweep(&args),
        "bots" => cmd_bots(&args),
        "mobile-study" => cmd_mobile_study(&args),
        _ => usage(),
    }
}

fn scenario_config(args: &Args) -> ScenarioConfig {
    let (name, players) = match (args.get("preset"), args.get("players")) {
        (_, Some(list)) => (
            "custom".to_string(),
            parse_players(list).unwrap_or_else(|e| fail(&e)),
        ),
        (preset, None) => {
            let name = preset.unwrap_or("taiwan-rock-4");
            let players =
                scenario::preset(name).unwrap_or_else(|| fail(&format!("unknown preset {name}")));
            (name.to_string(), players)
        }
    };
    let endpoint_name = args.get("endpoint").unwrap_or("native-interface");
    ScenarioConfig {
        name,
        players,
        endpoint: EndpointProfile::preset(endpoint_name)
            .unwrap_or_else(|| fail(&format!("unknown endpoint {endpoint_name}"))),
        cross_isp: args.has("cross-isp"),
        topology: topology(args),
        bpm: args.num("bpm", 120.0),
        bars: args.num("bars", 32),
        seeds: args.num("seeds", 20),
        duration_ms: 20_000.0,
        compensation: args.num("compensation", 0.0),
        coverage: args.num("coverage", 0.99),
        redundancy: args.num("redundancy", 1),
    }
}

fn topology(args: &Args) -> Topology {
    let name = args.get("topology").unwrap_or("forward");
    Topology::parse(name).unwrap_or_else(|| fail(&format!("unknown topology {name}")))
}

fn write_out<T: Serialize>(dir: &Path, stem: &str, json: &T, markdown: &str) {
    fs::create_dir_all(dir).unwrap_or_else(|e| fail(&format!("create {}: {e}", dir.display())));
    let json_path = dir.join(format!("{stem}.json"));
    let md_path = dir.join(format!("{stem}.md"));
    fs::write(
        &json_path,
        serde_json::to_string_pretty(json).expect("serialize"),
    )
    .unwrap_or_else(|e| fail(&format!("write {}: {e}", json_path.display())));
    fs::write(&md_path, markdown)
        .unwrap_or_else(|e| fail(&format!("write {}: {e}", md_path.display())));
    eprintln!("wrote {} and {}", json_path.display(), md_path.display());
}

fn cmd_scenario(args: &Args) {
    let cfg = scenario_config(args);
    let report = scenario::run(&cfg);
    let md = scenario_markdown(&report);
    print!("{md}");
    if let Some(out) = args.get("out") {
        let dir = PathBuf::from(out);
        write_out(&dir, "scenario", &report, &md);
        if args.has("wav") {
            let roles: Vec<_> = cfg.players.iter().map(|p| p.role).collect();
            for (i, p) in cfg.players.iter().enumerate() {
                let samples = render_listener_mix(
                    &roles,
                    &report.example_run.onsets_ms,
                    &report.latency_matrix_ms,
                    i,
                    1_500.0,
                );
                let path = dir.join(format!("heard-by-{}.wav", p.role.name()));
                fs::write(&path, encode_wav(&samples, SAMPLE_RATE))
                    .unwrap_or_else(|e| fail(&format!("write {}: {e}", path.display())));
                eprintln!("wrote {}", path.display());
            }
        }
    }
}

fn cmd_budget(args: &Args) {
    let cfg = scenario_config(args);
    let peers = peer_setups(&cfg);
    let budgets: Vec<_> = peers
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            peers
                .iter()
                .enumerate()
                .filter(move |(j, _)| *j != i)
                .map(move |(_, b)| star_path(a, b, cfg.topology))
        })
        .collect();
    print!("{}", budget_markdown(&budgets));
}

fn cmd_sweep(args: &Args) {
    let max: f64 = args.num("max", 80.0);
    let step: f64 = args.num("step", 5.0);
    let bpm = args.num("bpm", 120.0);
    let compensation = args.num("compensation", 0.0);
    if step <= 0.0 {
        fail("--step must be positive");
    }
    let lats: Vec<f64> = (0..)
        .map(|k| k as f64 * step)
        .take_while(|l| *l <= max + 1e-9)
        .collect();
    let rows = sweep_uniform_latency(
        &lats,
        bpm,
        args.num("bars", 32),
        args.num("runs", 30),
        compensation,
    );
    let md = sweep_markdown(&rows, bpm, compensation);
    print!("{md}");
    if let Some(out) = args.get("out") {
        write_out(Path::new(out), "sweep", &rows, &md);
    }
}

fn cmd_mobile_study(args: &Args) {
    let role_name = args.get("role").unwrap_or("vocals");
    let role = Role::parse(role_name).unwrap_or_else(|| fail(&format!("unknown role {role_name}")));
    let rows = scenario::mobile_study(role, args.num("seeds", 10), 30_000.0);
    let md = mobile_study_markdown(&rows);
    print!("{md}");
    if let Some(out) = args.get("out") {
        write_out(Path::new(out), "mobile-study", &rows, &md);
    }
}

fn cmd_bots(args: &Args) {
    let players = match args.get("players") {
        Some(list) => parse_players(list).unwrap_or_else(|e| fail(&e)),
        None => {
            let n: usize = args.num("count", 4);
            let preset = scenario::preset("taiwan-rock-4").expect("preset");
            if !(2..=4).contains(&n) {
                fail("--count must be 2-4");
            }
            preset.into_iter().take(n).collect()
        }
    };
    let impair = args.has("impair");
    let links: Vec<LinkProfile> = players
        .iter()
        .map(|p| {
            if impair {
                relay_link(p.site, p.access, false)
            } else {
                LinkProfile::ideal()
            }
        })
        .collect();
    let forward = topology(args) == Topology::Forward;
    // Mix: a client buffers one stream that crossed its own downlink.
    // Forward: each source's stream crosses that source's uplink and this
    // downlink, so size each per-source buffer for the combined path.
    let coverage: f64 = args.num("coverage", 0.99);
    let depth = |jitter: f64| recommended_depth(jitter, frame_ms()).max(2);
    let depths: Vec<Vec<usize>> = links
        .iter()
        .map(|own| {
            links
                .iter()
                .map(|src| {
                    if forward {
                        depth(src.then(own).jitter_quantile_ms(coverage))
                    } else {
                        depth(own.jitter_quantile_ms(coverage))
                    }
                })
                .collect()
        })
        .collect();
    let bpm = args.num("bpm", 120.0);
    let mode = args.get("mode").unwrap_or("probe");
    let modes: Vec<BotMode> = players
        .iter()
        .map(|p| match mode {
            "probe" => BotMode::Probe,
            "music" => BotMode::Music { role: p.role, bpm },
            other => fail(&format!("unknown mode {other}")),
        })
        .collect();
    let relay = args.get("relay").map(|r| {
        r.parse()
            .unwrap_or_else(|_| fail(&format!("bad --relay {r}")))
    });
    let record = args.has("wav");
    let result = run_band(BandConfig {
        relay,
        forward,
        relay_depth: args.num("relay-depth", 2),
        links,
        jitter_depths: depths,
        modes,
        duration: Duration::from_secs_f64(args.num("seconds", 10.0)),
        record,
    })
    .unwrap_or_else(|e| fail(&format!("bots: {e}")));
    let label = format!(
        "{} bots, mode {mode}, impairment {}",
        players.len(),
        if impair { "assumed-taiwan" } else { "none" }
    );
    let md = band_markdown(&result, players.len(), &label);
    print!("{md}");
    if let Some(out) = args.get("out") {
        let dir = PathBuf::from(out);
        write_out(&dir, "bots", &result.bots, &md);
        if record {
            for (b, p) in result.bots.iter().zip(&players) {
                let path = dir.join(format!("bot-heard-by-{}.wav", p.role.name()));
                fs::write(&path, encode_wav(&b.recording, SAMPLE_RATE))
                    .unwrap_or_else(|e| fail(&format!("write {}: {e}", path.display())));
                eprintln!("wrote {}", path.display());
            }
        }
    }
}
