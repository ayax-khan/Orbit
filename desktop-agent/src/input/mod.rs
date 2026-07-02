//! Input simulation: applies remote mouse/keyboard events to the host.

pub mod events;
pub mod keyboard;
pub mod mouse;

pub use events::InputProcessor;

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("failed to initialize input backend: {0}")]
    Init(String),
    #[error("input simulation failed: {0}")]
    Simulation(String),
}
