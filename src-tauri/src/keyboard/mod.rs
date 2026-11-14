//! Everything that talks to the keyboard: the protocol, reply parsing, USB transport and the worker
//! thread that owns the device.

mod conflicts;
mod error;
mod lighting;
mod protocol;
mod session;
mod settings;
mod transport;
mod worker;

pub use error::KeyboardError;
pub use lighting::LightingSettings;
pub use protocol::ONBOARD_SLOTS;
pub use settings::POLLING_RATES;
pub use session::{ApplyRequest, Snapshot};
pub use worker::{KeyboardHandle, KeyboardStatus, STATUS_EVENT};
#[cfg(test)]
pub use lighting::{Lighting, MAX_LEVEL, lighting_modes};
