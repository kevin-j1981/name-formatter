use std::time::{SystemTime, UNIX_EPOCH};

/// Small xorshift64* generator so weighted sampling doesn't need a `rand`
/// dependency. Not suitable for anything security-sensitive, just for
/// picking names.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Seed from the system clock. Good enough for picking random names;
    /// no two runs need to be unpredictable against an adversary.
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15);
        Self::from_seed(nanos)
    }

    /// Seed explicitly, mainly so tests can get a reproducible sequence.
    pub fn from_seed(seed: u64) -> Self {
        // xorshift64* is undefined for a zero state, so nudge it off zero.
        let state = if seed == 0 { 0x2545F4914F6CDD1D } else { seed };
        Rng { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// A float in [0, 1).
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// Draw `count` names from `weighted`, with replacement, where each name's
/// chance of being picked is proportional to how often it occurred in the
/// source list. Returns an empty vector if there's nothing to draw from.
pub fn weighted_sample(weighted: &[(String, usize)], count: usize, rng: &mut Rng) -> Vec<String> {
    let total: usize = weighted.iter().map(|(_, weight)| *weight).sum();
    if total == 0 {
        return Vec::new();
    }

    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let target = ((rng.next_f64() * total as f64) as usize).min(total - 1);
        let mut cumulative = 0usize;
        for (name, weight) in weighted {
            cumulative += weight;
            if target < cumulative {
                out.push(name.clone());
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_no_samples() {
        let mut rng = Rng::from_seed(1);
        assert_eq!(weighted_sample(&[], 5, &mut rng), Vec::<String>::new());
    }

    #[test]
    fn requesting_zero_samples_yields_none() {
        let weighted = vec![("Alice".to_string(), 1)];
        let mut rng = Rng::from_seed(1);
        assert_eq!(weighted_sample(&weighted, 0, &mut rng), Vec::<String>::new());
    }

    #[test]
    fn single_entry_always_wins() {
        let weighted = vec![("Alice".to_string(), 1)];
        let mut rng = Rng::from_seed(42);
        let picks = weighted_sample(&weighted, 10, &mut rng);
        assert_eq!(picks.len(), 10);
        assert!(picks.iter().all(|p| p == "Alice"));
    }

    #[test]
    fn heavier_weight_dominates_over_many_draws() {
        let weighted = vec![("Common".to_string(), 99), ("Rare".to_string(), 1)];
        let mut rng = Rng::from_seed(7);
        let picks = weighted_sample(&weighted, 1000, &mut rng);
        let common_count = picks.iter().filter(|p| p.as_str() == "Common").count();
        // Not asserting an exact count since it's randomized, just that the
        // 99:1 weighting is clearly reflected rather than a coin flip.
        assert!(common_count > 900, "expected most draws to be Common, got {common_count}/1000");
    }

    #[test]
    fn same_seed_produces_same_sequence() {
        let weighted = vec![("Alice".to_string(), 1), ("Bob".to_string(), 1)];
        let mut rng_a = Rng::from_seed(123);
        let mut rng_b = Rng::from_seed(123);
        assert_eq!(
            weighted_sample(&weighted, 20, &mut rng_a),
            weighted_sample(&weighted, 20, &mut rng_b)
        );
    }
}
