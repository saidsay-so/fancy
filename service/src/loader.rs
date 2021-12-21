use std::collections::HashSet;
use std::sync::Arc;

use async_std::prelude::*;
use async_std::sync::RwLock;
use async_std::{
    fs::{self, read_dir},
    path::PathBuf,
};
use nbfc_config::{FanControlConfigV2, XmlFanControlConfigV2};
use phf::{phf_ordered_map, OrderedMap};
use smol::unblock;
use snafu::{ensure, ResultExt, Snafu};
use zbus::dbus_interface;

use crate::ec_control::{Event, EventSender, ExternalEvent};

const DEFAULT_PATH: &str = "/etc/fancy/configs/";

#[derive(Debug, Snafu)]
pub(crate) enum LoaderError {
    #[snafu(display("The file could not be loaded: {}", source))]
    FileLoading { source: async_std::io::Error },

    #[snafu(display("Could not refresh list of available configs: {}", source))]
    Refresh { source: async_std::io::Error },

    #[snafu(display("Config '{}' is unavailable", config))]
    UnavailableConfig { config: String },

    #[snafu(display("Invalid config: {}", source))]
    InvalidConfig { source: Box<LoaderError> },

    #[snafu(display("Invalid XML data: {}", source))]
    InvalidXml { source: quick_xml::de::DeError },

    #[snafu(display("Invalid JSON data: {}", source))]
    InvalidJson { source: serde_json::Error },

    #[snafu(display("'{}' cannot be read: {}", config, source))]
    ReadString {
        config: String,
        source: async_std::io::Error,
    },
}

impl From<LoaderError> for zbus::fdo::Error {
    fn from(e: LoaderError) -> Self {
        let msg = e.to_string();
        match e {
            LoaderError::FileLoading { .. }
            | LoaderError::Refresh { .. }
            | LoaderError::ReadString { .. } => zbus::fdo::Error::IOError(msg),
            LoaderError::UnavailableConfig { .. } => zbus::fdo::Error::FileNotFound(msg),
            _ => zbus::fdo::Error::Failed(msg),
        }
    }
}

type Result<T> = std::result::Result<T, LoaderError>;

type FormatDeserializer = fn(&str) -> Result<FanControlConfigV2>;

fn xml_loader(s: &str) -> Result<FanControlConfigV2> {
    let raw_xml_config =
        quick_xml::de::from_str::<XmlFanControlConfigV2>(s).context(InvalidXml {})?;

    Ok(raw_xml_config.into())
}

fn json_loader(s: &str) -> Result<FanControlConfigV2> {
    serde_json::de::from_str(s).context(InvalidJson {})
}

const SUPPORTED_EXTENSIONS: OrderedMap<&str, FormatDeserializer> = phf_ordered_map! {
    "xml" => xml_loader,
    "json" => json_loader,
};

pub(crate) struct Loader {
    configs: HashSet<String>,
    pub current_config: Option<(String, FanControlConfigV2)>,
    //TODO: Replace sending event to using a stream or something else
    ev_sender: EventSender,
}

impl Loader {
    pub async fn new(ev_sender: EventSender) -> Self {
        Self {
            configs: (HashSet::new()),
            current_config: (None),
            ev_sender,
        }
    }

    async fn refresh_available(&mut self) -> Result<()> {
        self.configs = read_dir(DEFAULT_PATH)
            .await
            .context(Refresh {})?
            .filter_map(|entry| entry.ok())
            .filter_map(|file| {
                file.path()
                    .file_stem()
                    .map(|f| f.to_string_lossy().to_string())
            })
            .collect::<HashSet<String>>()
            .await;

        Ok(())
    }

    pub async fn load(&mut self, config_name: &str) -> Result<()> {
        if !self.configs.contains(config_name) {
            self.refresh_available().await?;
            ensure!(
                self.configs.contains(config_name),
                UnavailableConfig {
                    config: config_name.to_owned()
                }
            );
        }

        let mut path = PathBuf::from(DEFAULT_PATH);
        path.push(&config_name);

        for (extension, loader) in SUPPORTED_EXTENSIONS.entries() {
            path.set_extension(extension);

            if !path.is_file().await {
                continue;
            }

            let buf = fs::read_to_string(path).await.context(ReadString {
                config: config_name.clone(),
            })?;

            let config_name = config_name.to_string();
            let control_config = unblock(move || loader(&buf)).await?;
            self.current_config.replace((config_name, control_config));
            return Ok(());
        }

        Err(LoaderError::UnavailableConfig {
            config: config_name.to_owned(),
        })
    }
}

#[dbus_interface(name = "com.musikid.fancy.Loader")]
impl Loader {
    async fn configs(&mut self) -> zbus::fdo::Result<Vec<String>> {
        self.refresh_available().await?;
        Ok(self.configs.iter().cloned().collect())
    }

    #[dbus_interface(property, name = "CurrentConfig")]
    fn current_config(&self) -> String {
        self.current_config
            .as_ref()
            .map(|t| t.0.clone())
            .unwrap_or_default()
    }

    #[dbus_interface(property, name = "CurrentConfig")]
    async fn set_current_config(&mut self, config: String) -> zbus::fdo::Result<()> {
        self.load(&config)
            .await
            .map_err(|_| zbus::Error::Unsupported)?;

        let control_config = self.current_config.as_ref().unwrap().1.clone();

        self.ev_sender
            .send_event(Event::External(ExternalEvent::RefreshConfig(
                control_config,
            )))
            .await;

        Ok(())
    }
}
