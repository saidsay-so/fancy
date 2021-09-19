use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::state::ProxyState;

#[derive(Debug, Error)]
pub enum Error {
  //TODO: add the zbus error
  #[error("connection to d-bus service has been refused: {0}")]
  ConnectionRefused(String),
  #[error("A DBus error occured: {0}")]
  DBusError(#[from] zbus::Error),
  #[error("connection has not been established yet")]
  UninitializedConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsError {
  message: String,
  critical: bool,
}

impl JsError {
  pub fn new<S: Into<String>>(message: S, critical: bool) -> Self {
    JsError {
      message: message.into(),
      critical,
    }
  }
}

pub(super) fn generate_proxy_err(proxy_state: &ProxyState) -> JsError {
  match proxy_state {
    ProxyState::Uninitialized => JsError::new(Error::UninitializedConnection.to_string(), false),
    //TODO: use the error
    ProxyState::Error(e) => JsError::new(Error::ConnectionRefused(e.to_string()).to_string(), true),
    ProxyState::Initialized => unreachable!(),
  }
}
