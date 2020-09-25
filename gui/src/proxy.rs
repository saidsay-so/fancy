use flume::{unbounded, Receiver, Sender};
use futures::{
    future::{join, FutureExt},
    stream::StreamExt,
};
use relm::{Channel, Relm, Sender as RelmSender, StreamHandle, Update, UpdateNew};
use relm_derive::Msg;
use zbus::azync::Connection;
use zbus::fdo::*;

use std::thread::{self, JoinHandle};
use std::{collections::HashMap, convert::TryInto};

use super::interfaces::*;

pub type ProxyBus = StreamHandle<Msg>;

pub struct Model {
    _channel: Channel<Msg>,
    _thread: JoinHandle<()>,
    sender: Sender<Msg>,
}

#[derive(Debug, Msg)]
pub enum Msg {
    // Events emitted by the widgets.
    SetConfig(String),
    SetTargetFanSpeed(u8, f64),
    SetAuto(bool),
    ReqFansSpeeds,
    ReqAuto,
    // Events emitted for the widgets.
    Config(String),
    FansSpeeds(HashMap<String, f64>),
    Auto(bool),
    Critical(bool),
}

pub struct Proxy {
    model: Model,
}

impl Update for Proxy {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(relm: &Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        let stream = relm.stream().clone();

        // This channel send events from the worker to the UI thread.
        let (_channel, worker_sender) = Channel::new(move |event| {
            stream.emit(event);
        });

        // And vice-versa.
        let (relm_sender, receiver) = unbounded();

        let _thread = thread::spawn(|| smol::block_on(main_loop(worker_sender, receiver)));

        Model {
            _channel,
            _thread,
            sender: relm_sender,
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        match msg {
            Msg::SetConfig(c) => self.model.sender.send(Msg::SetConfig(c)).unwrap(),
            Msg::SetTargetFanSpeed(i, s) => self
                .model
                .sender
                .send(Msg::SetTargetFanSpeed(i, s))
                .unwrap(),
            Msg::SetAuto(a) => self.model.sender.send(Msg::SetAuto(a)).unwrap(),
            Msg::ReqFansSpeeds => self.model.sender.send(Msg::ReqFansSpeeds).unwrap(),
            Msg::ReqAuto => self.model.sender.send(Msg::ReqAuto).unwrap(),
            _ => {}
        }
    }
}

impl UpdateNew for Proxy {
    fn new(_relm: &Relm<Self>, model: Model) -> Self {
        Proxy { model }
    }
}

/*macro_rules! closure_try {
    ($x: expr) => {
        match $x {
            Ok($x) => $x,
            Err(e) => return Box::pin(async { Err(FdoError::Failed(e.to_string())) }),
        }
    };
}*/

async fn main_loop(sender: RelmSender<Msg>, receiver: Receiver<Msg>) {
    //FIXME: Reading two proxies from the same connection blocks.
    let props_conn = Connection::new_system().await.unwrap();
    let props_proxy =
        AsyncPropertiesProxy::new_for(&props_conn, "com.musikid.fancy", "/com/musikid/fancy")
            .unwrap();

    let fancy_conn = Connection::new_system().await.unwrap();
    let fancy_proxy = AsyncFancyProxy::new(&fancy_conn).unwrap();

    {
        let sender = sender.clone();
        props_proxy
            .connect_properties_changed(move |_iface, props, _invalid_props| {
                if let Some(config) = props.get("Config") {
                    let config: String = config.try_into().unwrap();
                    sender.send(Msg::Config(config)).unwrap();
                }

                if let Some(auto) = props.get("Auto") {
                    let auto: bool = auto.try_into().unwrap();
                    sender.send(Msg::Auto(auto)).unwrap();
                }
                async { Ok(()) }.boxed()
            })
            .await
            .unwrap();
    }

    let config = fancy_proxy.config().await.unwrap();
    sender.send(Msg::Config(config)).unwrap();

    let critical = fancy_proxy.critical().await.unwrap();
    sender.send(Msg::Critical(critical)).unwrap();

    let auto = fancy_proxy.auto().await.unwrap();
    sender.send(Msg::Auto(auto)).unwrap();

    let signal_handler = async { while props_proxy.next_signal().await.unwrap().is_none() {} };

    let msg_handler = async {
        let mut receiver = receiver.into_stream();
        while let Some(msg) = receiver.next().await {
            match msg {
                Msg::SetConfig(c) => fancy_proxy.set_config(&c).await.unwrap(),
                Msg::SetTargetFanSpeed(i, s) => {
                    fancy_proxy.set_target_fan_speed(i, s).await.unwrap()
                }
                Msg::SetAuto(a) => fancy_proxy.set_auto(a).await.unwrap(),
                Msg::ReqFansSpeeds => {
                    let speeds = fancy_proxy.fans_speeds().await.unwrap();
                    sender.send(Msg::FansSpeeds(speeds)).unwrap()
                }
                Msg::ReqAuto => {
                    let auto = fancy_proxy.auto().await.unwrap();
                    sender.send(Msg::Auto(auto)).unwrap()
                }
                _ => {}
            }
        }
    };

    join(msg_handler, signal_handler).await;
}
