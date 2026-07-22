use crate::live_clock::canonical_metrics_from_valid_local_probe;
use crate::live_schema::LiveMetricValue;
use serde_json::Value;

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

fn mad(values: &[f64], med: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let deviations: Vec<f64> = values.iter().map(|v| (v - med).abs()).collect();
    median(&deviations)
}

fn metric_observed(value: f64) -> LiveMetricValue {
    LiveMetricValue::ObservedNumber { value }
}

fn metric_unavailable(reason: &str) -> LiveMetricValue {
    LiveMetricValue::Unavailable {
        reason: reason.to_string(),
    }
}

pub fn compute_live_endpoint_derived(endpoint: &Value) -> Value {
    let stats = endpoint
        .get("statsSamples")
        .or_else(|| endpoint.get("stats_samples"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let clock = endpoint
        .get("clockProbes")
        .or_else(|| endpoint.get("clock_probes"))
        .and_then(|v| v.get("samples"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let peer_role = endpoint
        .get("peerRole")
        .or_else(|| endpoint.get("peer_role"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut rtts = Vec::new();
    let mut offsets = Vec::new();
    for probe in &clock {
        if let Some(metrics) = canonical_metrics_from_valid_local_probe(probe, peer_role) {
            rtts.push(metrics.rtt_ms);
            offsets.push(metrics.offset_ms);
        }
    }

    let mut pair_rtts = Vec::new();
    let mut jb_delays = Vec::new();
    let mut jb_targets = Vec::new();
    let mut jb_mins = Vec::new();
    let mut loss_deltas = Vec::new();
    let mut recv_bitrates = Vec::new();
    let mut send_bitrates = Vec::new();
    let mut conceal_ratios = Vec::new();
    let mut inserted_ratios = Vec::new();
    let mut removed_ratios = Vec::new();

    let mut prev: Option<&Value> = None;
    for sample in &stats {
        if let Some(interval) = sample.get("intervalMetrics") {
            push_if_observed(interval.get("candidatePairRttMs"), &mut pair_rtts);
            push_if_observed(interval.get("jitterBufferDelayMs"), &mut jb_delays);
            push_if_observed(interval.get("jitterBufferTargetDelayMs"), &mut jb_targets);
            push_if_observed(interval.get("jitterBufferMinimumDelayMs"), &mut jb_mins);
            push_if_observed(interval.get("packetLossDelta"), &mut loss_deltas);
            push_if_observed(interval.get("receiveBitrateBps"), &mut recv_bitrates);
            push_if_observed(interval.get("sendBitrateBps"), &mut send_bitrates);
            push_if_observed(interval.get("concealmentRatio"), &mut conceal_ratios);
            push_if_observed(interval.get("insertedSampleRatio"), &mut inserted_ratios);
            push_if_observed(interval.get("removedSampleRatio"), &mut removed_ratios);
        } else if let Some(p) = prev {
            derive_interval_from_samples(
                p,
                sample,
                &mut pair_rtts,
                &mut recv_bitrates,
                &mut send_bitrates,
            );
        }
        prev = Some(sample);
    }

    let rtt_med = median(&pair_rtts);
    let clock_rtt_med = median(&rtts);
    let offset_med = median(&offsets);

    serde_json::json!({
        "medianCandidatePairRttMs": rtt_med.map(metric_observed).unwrap_or_else(|| metric_unavailable("insufficient samples")),
        "candidatePairRttMadMs": rtt_med.and_then(|m| mad(&pair_rtts, m).map(metric_observed)).unwrap_or_else(|| metric_unavailable("insufficient samples")),
        "medianClockProbeRttMs": clock_rtt_med.map(metric_observed).unwrap_or_else(|| metric_unavailable("insufficient probes")),
        "clockProbeRttMadMs": clock_rtt_med.and_then(|m| mad(&rtts, m).map(metric_observed)).unwrap_or_else(|| metric_unavailable("insufficient probes")),
        "medianClockOffsetMs": offset_med.map(metric_observed).unwrap_or_else(|| metric_unavailable("insufficient probes")),
        "clockOffsetMadMs": offset_med.and_then(|m| mad(&offsets, m).map(metric_observed)).unwrap_or_else(|| metric_unavailable("insufficient probes")),
        "intervalJitterBufferDelayAvgMs": avg_metric(&jb_delays),
        "intervalJitterBufferTargetDelayAvgMs": avg_metric(&jb_targets),
        "intervalJitterBufferMinimumDelayAvgMs": avg_metric(&jb_mins),
        "intervalPacketLossDeltaAvg": avg_metric(&loss_deltas),
        "intervalReceiveBitrateAvgBps": avg_metric(&recv_bitrates),
        "intervalSendBitrateAvgBps": avg_metric(&send_bitrates),
        "concealmentRatioAvg": avg_metric(&conceal_ratios),
        "insertedSampleRatioAvg": avg_metric(&inserted_ratios),
        "removedSampleRatioAvg": avg_metric(&removed_ratios),
    })
}

fn push_if_observed(val: Option<&Value>, out: &mut Vec<f64>) {
    if let Some(v) = val.and_then(|v| v.get("value")).and_then(|v| v.as_f64()) {
        out.push(v);
    } else if let Some(v) = val.and_then(|v| v.as_f64()) {
        out.push(v);
    }
}

fn avg_metric(values: &[f64]) -> LiveMetricValue {
    if values.is_empty() {
        metric_unavailable("insufficient interval samples")
    } else {
        metric_observed(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn derive_interval_from_samples(
    prev: &Value,
    curr: &Value,
    pair_rtts: &mut Vec<f64>,
    recv_bitrates: &mut Vec<f64>,
    send_bitrates: &mut Vec<f64>,
) {
    let dt = curr
        .get("offsetMs")
        .and_then(|v| v.as_f64())
        .unwrap_or(1000.0)
        - prev.get("offsetMs").and_then(|v| v.as_f64()).unwrap_or(0.0);
    if dt <= 0.0 {
        return;
    }
    if let (Some(prev_bytes), Some(curr_bytes)) = (
        prev.pointer("/inboundAudio/bytesReceived/value")
            .or(prev.pointer("/inboundAudio/bytesReceived"))
            .and_then(|v| v.as_f64()),
        curr.pointer("/inboundAudio/bytesReceived/value")
            .or(curr.pointer("/inboundAudio/bytesReceived"))
            .and_then(|v| v.as_f64()),
    ) {
        let delta = curr_bytes - prev_bytes;
        if delta >= 0.0 {
            recv_bitrates.push((delta * 8.0 * 1000.0) / dt);
        }
    }
    if let (Some(prev_bytes), Some(curr_bytes)) = (
        prev.pointer("/outboundAudio/bytesSent/value")
            .or(prev.pointer("/outboundAudio/bytesSent"))
            .and_then(|v| v.as_f64()),
        curr.pointer("/outboundAudio/bytesSent/value")
            .or(curr.pointer("/outboundAudio/bytesSent"))
            .and_then(|v| v.as_f64()),
    ) {
        let delta = curr_bytes - prev_bytes;
        if delta >= 0.0 {
            send_bitrates.push((delta * 8.0 * 1000.0) / dt);
        }
    }
    if let Some(rtt) = curr
        .pointer("/candidatePair/currentRoundTripTime/value")
        .or(curr.pointer("/candidatePair/currentRoundTripTime"))
        .and_then(|v| v.as_f64())
    {
        pair_rtts.push(rtt * 1000.0);
    }
}
