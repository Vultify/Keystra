//! Everything that talks to the keyboard: the protocol, reply parsing, USB transport and the worker
//! thread that owns the device.

mod conflicts;
mod error;
mod protocol;
mod settings;
mod transport;
mod worker;

pub use error::KeyboardError;
pub use worker::{KeyboardHandle, KeyboardStatus, Snapshot};
