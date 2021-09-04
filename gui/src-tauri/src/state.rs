use std::collections::HashMap;

#[derive(Debug)]
pub enum Msg {
  GetTemps,
  Temps(HashMap<String, f64>),
  GetSpeeds,
  Speeds(Vec<f64>),
  GetPollInterval,
  PollInterval(u64),
  GetTargetSpeeds,
  TargetSpeeds(Vec<f64>),
  SetTargetSpeed(u8, f64),
  GetCritical,
  Critical(bool),
  GetNames,
  Names(Vec<String>),
  GetAuto,
  Auto(bool),
  SetAuto(bool),
}

#[derive(Debug)]
pub struct State {
  pub config: String,
  pub poll_interval: u64,
  pub msg_sender: flume::Sender<Msg>,
  pub msg_receiver: flume::Receiver<Msg>,
}

impl State {
  pub fn new(msg_sender: flume::Sender<Msg>, msg_receiver: flume::Receiver<Msg>) -> Self {
    State {
      msg_receiver,
      msg_sender,
      poll_interval: 0,
      config: String::new(),
    }
  }
}
