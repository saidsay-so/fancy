mod dbus;
mod misc;

use crate::error::JsError;

pub type CmdResult<T> = std::result::Result<T, JsError>;

pub use dbus::*;
pub use misc::*;
