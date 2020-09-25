use gtk::prelude::*;
use gtk::Inhibit;
use libhandy::HeaderBarExt;
use relm::{execute, EventStream, Relm, Widget};
use relm_derive::{widget, Msg};

use std::time::Duration;

mod interfaces;
mod proxy;
mod widgets;

use proxy::{Msg as BusMsg, Proxy};
use widgets::config_panel::ConfigPanel;
use widgets::fan_speed::Fans;

pub struct Model {
    proxy: EventStream<BusMsg>,
}

#[derive(Msg)]
pub enum Msg {
    Quit,
}

const CSS: &'static [u8] = include_bytes!("styles/main.css");

#[widget]
impl Widget for Win {
    type Root = libhandy::Window;

    fn init_view(&mut self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(CSS).expect("Failed to load CSS");
        gtk::StyleContext::add_provider_for_screen(
            &gdk::Screen::get_default().expect("Error initializing GTK CSS provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        /*let switcher = ViewSwitcherBuilder::new()
        .stack(&self.widgets.stack)
        .visible(true)
        .build();
        self.widgets.header.set_custom_title(Some(&switcher));*/
        self.widgets.header.set_title(Some("Fancy"));
    }

    fn model(_: &Relm<Self>, _params: ()) -> Model {
        let proxy = execute::<Proxy>(());
        Model { proxy }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
        }
    }

    view! {
        libhandy::Window {
            delete_event(_,_) => (Msg::Quit, Inhibit(false)),
            gtk::Box {
                orientation: gtk::Orientation::Vertical,

                #[name="header"]
                libhandy::HeaderBar {
                    show_close_button: true,
                },

                #[name="main"]
                gtk::Box {
                    border_width: 25,
                    spacing: 64,
                    orientation: gtk::Orientation::Vertical,

                    #[name="config"]
                    ConfigPanel((self.model.proxy.stream().clone(),)) {
                    },

                    #[name="fans"]
                    Fans((self.model.proxy.stream().clone(), Duration::from_secs(1))) {
                    }
                },
            }
        }
    }
}

fn main() {
    Win::run(()).unwrap();
}
