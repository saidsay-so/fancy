use gtk::{prelude::*, Adjustment, Box as BoxWidget, FlowBox, Frame, FrameExt, Label, Scale};
use relm::{connect, connect_stream, Relm, Update, Widget};
use relm_derive::Msg;

use std::{collections::HashMap, time::Duration};

use crate::proxy::{Msg as BusMsg, ProxyBus};

#[derive(Msg)]
pub enum FanMsg {
    // This event is dispatched by the parent
    UpdateSpeed(f64),
    UpdateAuto(bool),
    SetTargetFanSpeed,
}

pub struct FanModel {
    name: String,
    index: u8,
    proxy: ProxyBus,
}

pub struct Fan {
    root: Frame,
    model: FanModel,
    speed: Label,
    scale: Scale,
    value: Adjustment,
}

impl Update for Fan {
    type Model = FanModel;
    type ModelParam = (String, u8, ProxyBus);
    type Msg = FanMsg;

    fn model(_relm: &Relm<Self>, params: Self::ModelParam) -> Self::Model {
        FanModel {
            name: params.0,
            index: params.1,
            proxy: params.2,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        let proxy = self.model.proxy.stream();
        proxy.emit(BusMsg::ReqAuto);
        connect_stream!(proxy@BusMsg::Auto(a), relm, FanMsg::UpdateAuto(a));
    }

    fn update(&mut self, msg: Self::Msg) {
        match msg {
            FanMsg::UpdateSpeed(s) => {
                self.speed.set_text(&format!("{:.1}%", s));
            }
            FanMsg::UpdateAuto(a) => {
                self.scale.set_visible(!a);
            }
            FanMsg::SetTargetFanSpeed => {
                self.model.proxy.emit(BusMsg::SetTargetFanSpeed(
                    self.model.index,
                    self.value.get_value(),
                ));
            }
        }
    }
}

impl Widget for Fan {
    type Root = Frame;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let root = Frame::new(Some(&model.name));
        root.set_shadow_type(gtk::ShadowType::None);
        root.get_style_context().add_class("fan");
        root.get_label_widget()
            .unwrap()
            .get_style_context()
            .add_class("name");
        root.set_label_align(0.5, 0.5);

        let box_w = BoxWidget::new(gtk::Orientation::Vertical, 4);
        box_w.set_border_width(8);
        box_w.set_baseline_position(gtk::BaselinePosition::Center);
        root.add(&box_w);

        let value = Adjustment::new(f64::NAN, 0., 100., 1., 10., 0.);
        let scale = Scale::new(gtk::Orientation::Horizontal, Some(&value));
        connect!(
            scale,
            connect_value_changed(_),
            relm,
            FanMsg::SetTargetFanSpeed
        );

        box_w.pack_start(&scale, false, false, 16);

        let speed = Label::new(None);
        speed.get_style_context().add_class("speed");
        box_w.pack_end(&speed, true, true, 16);
        root.show_all();

        Fan {
            root,
            model,
            speed,
            scale,
            value,
        }
    }
}

#[derive(Msg)]
pub enum FansMsg {
    BusSpeeds(HashMap<String, f64>),
    ConfigChange,
}

pub struct FansModel {
    proxy: ProxyBus,
    is_config_changed: bool,
    poll_interval: Duration,
}

pub struct Fans {
    model: FansModel,
    widgets: HashMap<String, relm::Component<Fan>>,
    list_box: FlowBox,
    root: gtk::Box,
}

impl Update for Fans {
    type Model = FansModel;
    type ModelParam = (ProxyBus, Duration);
    type Msg = FansMsg;

    fn model(_relm: &Relm<Self>, params: Self::ModelParam) -> Self::Model {
        FansModel {
            proxy: params.0,
            is_config_changed: true,
            poll_interval: params.1,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        let proxy = self.model.proxy.clone();
        connect_stream!(proxy@BusMsg::FansSpeeds(ref speeds), relm, FansMsg::BusSpeeds(speeds.clone()));

        let interval = self.model.poll_interval.as_millis() as u32;
        glib::timeout_add_local(interval, move || {
            proxy.emit(BusMsg::ReqFansSpeeds);
            glib::Continue(true)
        });
    }

    fn update(&mut self, msg: Self::Msg) {
        match msg {
            FansMsg::BusSpeeds(speeds) => {
                if self.model.is_config_changed {
                    for widget in self.widgets.values() {
                        self.list_box.remove(widget.widget());
                    }

                    let mut i = 0;
                    self.widgets = speeds
                        .iter()
                        .map(|(name, _)| {
                            let widget =
                                relm::init::<Fan>((name.clone(), i, self.model.proxy.clone()))
                                    .unwrap();
                            i += 1;
                            self.list_box.add(widget.widget());
                            (name.clone(), widget)
                        })
                        .collect();

                    self.model.is_config_changed = false;
                }

                for (name, speed) in speeds {
                    if let Some(widget) = self.widgets.get(&name) {
                        widget.emit(FanMsg::UpdateSpeed(speed));
                    }
                }
            }

            FansMsg::ConfigChange => {
                self.model.is_config_changed = true;
            }
        }
    }
}

impl Widget for Fans {
    type Root = gtk::Box;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(_relm: &Relm<Self>, model: Self::Model) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 4);

        let list_box = FlowBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::None);
        root.add(&list_box);
        root.show_all();

        Fans {
            model,
            root,
            widgets: Default::default(),
            list_box,
        }
    }
}
