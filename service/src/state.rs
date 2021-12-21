use crate::ec_control::EcAccessMode;

pub(crate) struct State {
    ec_access_mode: EcAccessMode,
    config: String,
    selected_sensors: Vec<String>,
}

