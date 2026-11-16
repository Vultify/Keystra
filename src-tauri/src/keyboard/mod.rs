//! Everything that talks to the keyboard: the protocol, reply parsing, USB transport and the worker
//! thread that owns the device.

mod conflicts;
mod device_events;
mod error;
mod lighting;
mod pattern;
mod protocol;
mod session;
mod settings;
mod transport;
mod worker;

pub use error::KeyboardError;
pub use lighting::LightingSettings;
pub use pattern::{LIGHT_COUNT, Pattern};
pub use protocol::PATTERN_SLOTS;
pub use protocol::ONBOARD_SLOTS;
pub use settings::POLLING_RATES;
#[cfg(test)]
pub use settings::GERMAN_DEVICE_ID;
pub use session::{ApplyRequest, Snapshot};
pub use worker::{KeyboardHandle, KeyboardStatus, STATUS_EVENT};
#[cfg(test)]
pub use lighting::{Lighting, MAX_LEVEL, lighting_modes};
