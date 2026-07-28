//! Multiplayer match protocol and host-authoritative room logic.
//!
//! Transport-agnostic: TCP/WS/loopback all speak the same messages.

#![forbid(unsafe_code)]

pub mod loopback;
pub mod msg;
pub mod room;

pub use msg::{ClientMsg, Role, ServerMsg, PROTOCOL_VERSION};
pub use room::{normalize_code, Phase, RoomError, RoomEvent, TttRoom};
