use std::time::Duration;

use echlub_jam::netsim::LinkProfile;
use echlub_jam_lab::bots::{run_band, BandConfig, BotMode};
use echlub_musician_sim::Role;

#[test]
fn probe_bots_measure_latency_through_real_relay() {
    let result = run_band(BandConfig {
        relay: None,
        forward: false,
        relay_depth: 2,
        links: vec![LinkProfile::ideal(); 2],
        jitter_depths: vec![vec![2; 2]; 2],
        modes: vec![BotMode::Probe; 2],
        duration: Duration::from_secs(4),
        record: false,
    })
    .expect("band runs");
    let relay = result.relay.expect("in-process relay");
    assert_eq!(relay.decode_errors, 0);
    for bot in &result.bots {
        assert!(bot.mixes_received > 100, "{bot:?}");
        assert!(
            bot.detections.len() >= 2,
            "bot {} detected {:?}",
            bot.peer_id,
            bot.detections
        );
        let mut lat: Vec<f64> = bot.detections.iter().map(|(_, l)| *l).collect();
        lat.sort_by(f64::total_cmp);
        let median = lat[lat.len() / 2];
        // Localhost: a handful of frames of buffering. Generous for slow CI.
        assert!(median > 0.0 && median < 120.0, "median {median}");
    }
}

#[test]
fn forward_relay_delivers_every_peer() {
    let result = run_band(BandConfig {
        relay: None,
        forward: true,
        relay_depth: 2,
        links: vec![LinkProfile::ideal(); 3],
        jitter_depths: vec![vec![2; 3]; 3],
        modes: vec![BotMode::Probe; 3],
        duration: Duration::from_secs(3),
        record: false,
    })
    .expect("band runs");
    for bot in &result.bots {
        let mut sources: Vec<usize> = bot.detections.iter().map(|(s, _)| *s).collect();
        sources.sort();
        sources.dedup();
        assert_eq!(
            sources.len(),
            2,
            "bot {} heard {:?}",
            bot.peer_id,
            bot.detections
        );
    }
}

#[test]
fn music_bots_record_what_they_hear() {
    let roles = [Role::Drums, Role::Bass, Role::Guitar, Role::Vocals];
    let result = run_band(BandConfig {
        relay: None,
        forward: true,
        relay_depth: 2,
        links: vec![LinkProfile::new("lan", 0.5, 0.2, 0.0); 4],
        jitter_depths: vec![vec![2; 4]; 4],
        modes: roles
            .iter()
            .map(|r| BotMode::Music {
                role: *r,
                bpm: 120.0,
            })
            .collect(),
        duration: Duration::from_secs(2),
        record: true,
    })
    .expect("band runs");
    for bot in &result.bots {
        assert!(!bot.recording.is_empty());
        let loud = bot
            .recording
            .iter()
            .filter(|s| s.unsigned_abs() > 500)
            .count();
        assert!(loud > 1_000, "bot {} heard too little", bot.peer_id);
    }
}
