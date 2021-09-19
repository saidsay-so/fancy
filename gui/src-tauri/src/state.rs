use crate::interface::*;

#[derive(Debug)]
pub struct State<'a> {
  pub config: String,
  pub poll_interval: u64,
  pub proxy: Option<AsyncFancyProxy<'a>>,
  pub proxy_state: ProxyState,
}

#[derive(Debug)]
pub enum ProxyState {
  Uninitialized,
  Initialized,
  Error(zbus::Error),
}

impl<'a> State<'a> {
  pub fn new() -> Self {
    State {
      proxy: None,
      poll_interval: 0,
      config: String::new(),
      proxy_state: ProxyState::Uninitialized,
    }
  }

  pub fn set_proxy(&mut self, proxy: AsyncFancyProxy<'a>) {
    self.proxy = Some(proxy);
    self.proxy_state = ProxyState::Initialized;
  }

  pub fn set_connection_error(&mut self, proxy_err: zbus::Error) {
    self.proxy_state = ProxyState::Error(proxy_err);
  }
}
