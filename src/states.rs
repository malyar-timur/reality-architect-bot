use serde::{Deserialize, Serialize};

/// Состояния конечного автомата (FSM) — теперь только State::Idle (Stateless)
#[allow(dead_code)]
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
}
