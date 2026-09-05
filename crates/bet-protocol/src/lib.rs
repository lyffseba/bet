//! Multiplayer match protocol and host-authoritative room logic.
//!
//! Transport-agnostic: TCP/WS/loopback all speak the same messages.

#![forbid(unsafe_code)]

pub mod hangman_room;
pub mod host;
pub mod loopback;
pub mod msg;
pub mod room;
pub mod table;

pub use hangman_room::HangmanRoom;
pub use host::HostRoom;
pub use msg::{ClientMsg, GameKind, Role, ServerMsg, PROTOCOL_VERSION};
pub use room::{normalize_code, Phase, RoomError, RoomEvent, TttRoom};
pub use table::Table;
