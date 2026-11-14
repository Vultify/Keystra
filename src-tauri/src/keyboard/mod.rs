//! Everything that talks to the keyboard: the protocol, reply parsing, USB transport and the worker
//! thread that owns the device.

mod conflicts;
mod error;
mod protocol;
mod settings;
mod transport;
mod worker;

pub use error::KeyboardError;
pub use settings::{Lighting, MAX_LEVEL, POLLING_RATES, is_known_lighting_mode};
pub use worker::{KeyboardHandle, KeyboardStatus, Snapshot};
#[cfg(test)]
pub use settings::lighting_modes;
#[cfg(test)]
pub use worker::STATUS_EVENT;
