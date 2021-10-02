use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::state::ProxyState;

use strum::{AsRefStr, Display};

#[derive(Debug, Error, AsRefStr)]
pub enum Error {
  #[error("connection to d-bus service has been refused: {0}")]
  ConnectionRefused(String),
  #[error("A DBus error occured for the command '{1}': {0}")]
  CmdDBusError(zbus::Error, String),
  #[error("A DBus error occured while listening to changes: {0}")]
  ChangesDBusError(zbus::Error),
  #[error("An I/O error occured: {0}")]
  IoError(#[from] std::io::Error),
  #[error("An error occured while parsing configuration \"{1}\": {0}")]
  InvalidConfiguration(quick_xml::DeError, String),
  #[error("connection has not been established yet")]
  UninitializedConnection,
}

#[derive(Display, AsRefStr, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum ErrorEvent {
  ConnectionError,
  ProxyError,
  DeserializeError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsError {
  message: String,
  name: String,
  critical: bool,
}

impl JsError {
  pub fn new<S: Into<String>>(message: S, name: S, critical: bool) -> Self {
    JsError {
      message: message.into(),
      name: name.into(),
      critical,
    }
  }
}

pub(super) fn generate_proxy_err(proxy_state: &ProxyState) -> JsError {
  match proxy_state {
    ProxyState::Uninitialized => JsError::new(
      Error::UninitializedConnection.to_string(),
      Error::UninitializedConnection.as_ref().to_string(),
      false,
    ),
    //TODO: use the error
    ProxyState::Error(e) => {
      let e = Error::ConnectionRefused(e.to_string());
      JsError::new(e.to_string(), e.as_ref().to_string(), true)
    }
    ProxyState::Initialized => unreachable!(),
  }
}
