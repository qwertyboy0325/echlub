//! Mix-minus: every peer hears the sum of everyone except itself.

/// `inputs[i]` is peer `i`'s frame for this tick (silence if absent).
/// Returns one mix per input, each excluding that input.
pub fn mix_minus(inputs: &[Vec<i16>], frame_len: usize) -> Vec<Vec<i16>> {
    let mut total = vec![0i32; frame_len];
    for frame in inputs {
        for (acc, s) in total.iter_mut().zip(frame) {
            *acc += *s as i32;
        }
    }
    inputs
        .iter()
        .map(|own| {
            total
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let v = t - own.get(i).copied().unwrap_or(0) as i32;
                    v.clamp(i16::MIN as i32, i16::MAX as i32) as i16
                })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_own_signal() {
        let mixes = mix_minus(&[vec![1, 1], vec![10, 10], vec![100, 100]], 2);
        assert_eq!(mixes[0], vec![110, 110]);
        assert_eq!(mixes[1], vec![101, 101]);
        assert_eq!(mixes[2], vec![11, 11]);
    }

    #[test]
    fn saturates() {
        let mixes = mix_minus(&[vec![0], vec![30_000], vec![30_000]], 1);
        assert_eq!(mixes[0], vec![i16::MAX]);
    }
}
