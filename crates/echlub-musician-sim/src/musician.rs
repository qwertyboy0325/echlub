//! Musician roles and timing parameters.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Drums,
    Bass,
    Guitar,
    Vocals,
}

impl Role {
    pub const ROCK_FOUR_PIECE: [Role; 4] = [Role::Drums, Role::Bass, Role::Guitar, Role::Vocals];

    pub fn name(self) -> &'static str {
        match self {
            Role::Drums => "drums",
            Role::Bass => "bass",
            Role::Guitar => "guitar",
            Role::Vocals => "vocals",
        }
    }

    pub fn parse(s: &str) -> Option<Role> {
        Role::ROCK_FOUR_PIECE.into_iter().find(|r| r.name() == s)
    }

    /// How strongly other players lock to this role when listening.
    pub fn listen_weight(self) -> f64 {
        match self {
            Role::Drums => 2.0,
            Role::Bass => 1.3,
            Role::Guitar => 1.0,
            Role::Vocals => 0.6,
        }
    }
}

/// Linear phase/period correction timekeeper (Vorberg-Wing style).
///
/// Per beat: `t[k+1] = t[k] + T[k] - alpha*E[k] + noise` and
/// `T[k+1] = T[k] - beta*E[k] + gamma*(T0 - T[k])`, where `E[k]` is the
/// player's own onset minus the weighted mean of the onsets it heard from
/// others for beat `k`, plus its habitual anticipation (players tend to aim
/// slightly ahead of what they hear), and `T0` is the intended tempo.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MusicianParams {
    pub role: Role,
    /// Phase correction gain (0 = ignore others, 1 = fully re-align each beat).
    pub alpha: f64,
    /// Period (tempo) correction gain.
    pub beta: f64,
    /// Pull of the period back toward the intended tempo (tempo memory).
    pub gamma: f64,
    /// Habitual lead over what is heard, in ms (negative mean asynchrony).
    pub anticipation_ms: f64,
    /// Central timekeeper noise, SD in ms.
    pub timekeeper_sd_ms: f64,
    /// Motor implementation noise, SD in ms.
    pub motor_sd_ms: f64,
    /// Fraction of known incoming latency the player consciously discounts
    /// (0 = naive, 1 = perfect "play ahead of what you hear").
    pub latency_compensation: f64,
}

impl MusicianParams {
    /// Assumed defaults for a rock band where the drummer anchors time.
    pub fn rock_default(role: Role) -> Self {
        let (alpha, beta, gamma, antic, tk, motor) = match role {
            Role::Drums => (0.15, 0.02, 0.10, 8.0, 6.0, 2.0),
            Role::Bass => (0.40, 0.03, 0.08, 10.0, 8.0, 3.0),
            Role::Guitar => (0.35, 0.03, 0.08, 10.0, 9.0, 3.0),
            Role::Vocals => (0.25, 0.02, 0.06, 10.0, 12.0, 4.0),
        };
        Self {
            role,
            alpha,
            beta,
            gamma,
            anticipation_ms: antic,
            timekeeper_sd_ms: tk,
            motor_sd_ms: motor,
            latency_compensation: 0.0,
        }
    }
}
