use gio::{
    prelude::*, Cancellable, FileEnumeratorExt, FileExt, FileInfo, FileMonitorFlags,
    FileQueryInfoFlags, FILE_ATTRIBUTE_STANDARD_NAME,
};
use gtk::{prelude::*, Box as BoxWidget, ComboBoxText, ContainerExt, ListBox, Switch};
use libhandy::prelude::*;
use libhandy::ActionRow;
use relm::{connect, connect_async, connect_stream, Relm, Update, Widget};
use relm_derive::Msg;

use crate::proxy::{Msg as BusMsg, ProxyBus};

const CONFIGS_DIR: &str = "/etc/fancy/configs/";

pub struct PanelModel {
    proxy: ProxyBus,
}

#[derive(Msg)]
pub enum ChooserMsg {
    InputChange,
    BusConfigChange(String),
    ConfigsLoaded(Vec<FileInfo>),
}

#[derive(Msg)]
pub enum PanelMsg {}

pub struct ConfigPanel {
    _model: PanelModel,
    root: BoxWidget,
    _auto: relm::Component<Auto>,
    _chooser: relm::Component<Chooser>,
}

pub struct ChooserModel {
    current_config: Option<String>,
    configs: Vec<String>,
    proxy: ProxyBus,
}

pub struct Chooser {
    model: ChooserModel,
    root: ActionRow,
    combo_box: ComboBoxText,
}

impl Widget for Chooser {
    type Root = ActionRow;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let root = ActionRow::new();
        root.set_title(Some("Model"));
        root.set_subtitle(Some("Set the computer model"));

        let combo_box = ComboBoxText::new();
        connect!(relm, combo_box, connect_changed(_), ChooserMsg::InputChange);
        root.add(&combo_box);

        Chooser {
            combo_box,
            model,
            root,
        }
    }
}

impl Update for Chooser {
    type Model = ChooserModel;
    type ModelParam = (ProxyBus,);
    type Msg = ChooserMsg;

    fn model(_relm: &Relm<Self>, params: Self::ModelParam) -> Self::Model {
        ChooserModel {
            current_config: None,
            configs: vec![],
            proxy: params.0,
        }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            ChooserMsg::ConfigsLoaded(configs_info) => {
                self.model.configs = configs_info
                    .into_iter()
                    .filter_map(|info| info.get_name())
                    .filter_map(|name| name.file_stem().map(|s| s.to_string_lossy().to_string()))
                    .collect();
                self.model.configs.reverse();

                self.combo_box.remove_all();
                for c in &self.model.configs {
                    self.combo_box.append_text(&c);
                }

                if let Some(current_config) = self.model.current_config.as_ref() {
                    let index = self.model.configs.iter().position(|c| c == current_config);
                    self.combo_box.set_active(index.map(|i| i as u32));
                }
            }
            ChooserMsg::InputChange => {
                self.model.current_config = self.combo_box.get_active_text().map(|c| c.to_owned());
                if let Some(c) = self.model.current_config.as_ref() {
                    self.model.proxy.emit(BusMsg::SetConfig(c.clone()));
                }
            }
            ChooserMsg::BusConfigChange(conf) => {
                let index = self.model.configs.iter().position(|c| *c == conf);
                self.combo_box.set_active(index.map(|i| i as u32));
                self.model.current_config = Some(conf.clone());
            }
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        let proxy = self.model.proxy.stream();
        connect_stream!(proxy@BusMsg::Config(ref c), relm, ChooserMsg::BusConfigChange(c.clone()));

        let configs_dir = gio::File::new_for_path(CONFIGS_DIR);
        let monitor = configs_dir
            .monitor_directory::<Cancellable>(FileMonitorFlags::NONE, None)
            .unwrap();

        // We can refresh the configs if there is a change in the folder
        {
            let relm = relm.clone();
            monitor.connect_changed(move |_m, file, _other, _event| {
                let configs_enumerator = file
                    .enumerate_children::<Cancellable>(
                        *FILE_ATTRIBUTE_STANDARD_NAME,
                        FileQueryInfoFlags::NONE,
                        None,
                    )
                    .unwrap();

                connect_async!(
                    configs_enumerator,
                    next_files_async(1000, glib::PRIORITY_DEFAULT),
                    relm,
                    ChooserMsg::ConfigsLoaded
                );
            });
        }

        monitor.emit_event(&configs_dir, &configs_dir, gio::FileMonitorEvent::Changed);
    }
}

impl Update for ConfigPanel {
    type Model = PanelModel;
    type ModelParam = (ProxyBus,);
    type Msg = PanelMsg;

    fn model(_relm: &Relm<Self>, params: Self::ModelParam) -> Self::Model {
        PanelModel { proxy: params.0 }
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            _ => {}
        }
    }
}

impl Widget for ConfigPanel {
    type Root = BoxWidget;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(_relm: &Relm<Self>, model: Self::Model) -> Self {
        let root = BoxWidget::new(gtk::Orientation::Vertical, 16);
        let list_box = ListBox::new();
        list_box.set_vexpand(true);

        let chooser = relm::init::<Chooser>((model.proxy.stream(),)).unwrap();
        list_box.add(chooser.widget());

        let auto = relm::init::<Auto>((model.proxy.stream(),)).unwrap();
        list_box.add(auto.widget());

        root.add(&list_box);
        root.show_all();

        ConfigPanel {
            _model: model,
            root,
            _auto: auto,
            _chooser: chooser,
        }
    }
}

#[derive(Msg)]
pub enum AutoMsg {
    // Triggered by the user.
    Switch(bool),
    // Update from the proxy.
    UpdateAuto(bool),
}

pub struct AutoModel {
    auto: bool,
    proxy: ProxyBus,
}

pub struct Auto {
    model: AutoModel,
    switch: Switch,
    root: ActionRow,
}

impl Update for Auto {
    type Model = AutoModel;
    type ModelParam = (ProxyBus,);
    type Msg = AutoMsg;

    fn model(_relm: &Relm<Self>, params: Self::ModelParam) -> Self::Model {
        AutoModel {
            auto: false,
            proxy: params.0,
        }
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
        let proxy = self.model.proxy.stream();
        proxy.emit(BusMsg::ReqAuto);
        connect_stream!(proxy@BusMsg::Auto(a), relm, AutoMsg::UpdateAuto(a));
    }

    fn update(&mut self, msg: Self::Msg) {
        match msg {
            AutoMsg::Switch(a) => {
                self.model.auto = a;
                self.model.proxy.emit(BusMsg::SetAuto(a))
            }
            AutoMsg::UpdateAuto(a) => {
                self.model.auto = a;
                self.switch.set_state(a)
            }
        }
    }
}

impl Widget for Auto {
    type Root = ActionRow;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let switch = Switch::new();

        connect!(
            relm,
            switch,
            connect_state_set(_, a),
            return (AutoMsg::Switch(a), gtk::Inhibit(false))
        );

        let root = ActionRow::new();
        root.set_title(Some("Auto"));
        root.set_subtitle(Some(
            "Select a custom speed or let Fancy automatically choose it",
        ));
        root.add(&switch);

        Auto {
            model,
            root,
            switch,
        }
    }
}
