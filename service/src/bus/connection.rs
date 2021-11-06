/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dbus::blocking::LocalConnection;
use dbus_tree::{DataType, Factory, MethodErr};
use snafu::{ResultExt, Snafu};

use super::interfaces::*;
use crate::config::nbfc_control::test_load_control_config;
use crate::constants::{BUS_NAME_STR, OBJ_PATH_STR};
use crate::State;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Copy, Clone, Default, Debug)]
struct TData;

impl DataType for TData {
    type Tree = Rc<State>;
    type ObjectPath = ();
    type Interface = ();
    type Property = ();
    type Method = ();
    type Signal = ();
}

#[derive(Debug, Snafu)]
pub(crate) enum DBusError {
    #[snafu(display("An error occured with D-Bus: {}", source))]
    DBus { source: dbus::Error },
}

impl From<DBusError> for MethodErr {
    fn from(e: DBusError) -> Self {
        MethodErr::failed(&e)
    }
}

type IFaceResult<T> = Result<T, MethodErr>;

impl ComMusikidFancy for State {
    fn fans_speeds(&self) -> Result<Vec<f64>, MethodErr> {
        Ok(self.fans_speeds.borrow().to_owned())
    }
    fn target_fans_speeds(&self) -> Result<Vec<f64>, MethodErr> {
        Ok(self.target_fans_speeds.borrow().to_owned())
    }
    fn set_target_fans_speeds(&self, value: Vec<f64>) -> Result<(), MethodErr> {
        let mut target_fans_speeds = self.target_fans_speeds.borrow_mut();
        let len = self.fans_speeds.borrow().len();
        if value.len() != len {
            return Err(MethodErr::invalid_arg(
                "The number of values is not equal to the number of fans.",
            ));
        }
        if value.iter().any(|&v| v > 100. || v < 0.) {
            return Err(MethodErr::invalid_arg("One of the values is out of bounds"));
        }

        *target_fans_speeds = value;
        Ok(())
    }
    fn set_target_fan_speed(&self, index: u8, speed: f64) -> Result<(), MethodErr> {
        let mut target_fans_speeds = self.target_fans_speeds.borrow_mut();
        if index as usize >= target_fans_speeds.len() {
            return Err(MethodErr::invalid_arg(&format!(
                "{} is not a valid index.",
                index
            )));
        }
        if !(0f64..=100f64).contains(&speed) {
            return Err(MethodErr::invalid_arg("The speed is out of bounds"));
        }
        target_fans_speeds[index as usize] = speed;
        *self.manual_set_target_speeds.borrow_mut() = true;
        Ok(())
    }
    fn config(&self) -> Result<String, MethodErr> {
        Ok(self.config.borrow().to_owned())
    }
    fn set_config(&self, value: String) -> Result<(), MethodErr> {
        match test_load_control_config(&value, *self.check_control_config.borrow()) {
            Ok(_) => {
                *self.config.borrow_mut() = value;
                Ok(())
            }
            Err(e) => Err(MethodErr::failed(&e.to_string())),
        }
    }
    fn critical(&self) -> Result<bool, MethodErr> {
        Ok(*self.critical.borrow())
    }
    fn auto(&self) -> Result<bool, MethodErr> {
        Ok(*self.auto.borrow())
    }
    fn set_auto(&self, value: bool) -> Result<(), MethodErr> {
        *self.auto.borrow_mut() = value;
        Ok(())
    }
    fn temperatures(&self) -> Result<HashMap<String, f64>, MethodErr> {
        Ok(self.temps.borrow().to_owned())
    }
    fn poll_interval(&self) -> IFaceResult<u64> {
        Ok(*self.poll_interval.borrow())
    }
    fn fans_names(&self) -> Result<Vec<String>, dbus_tree::MethodErr> {
        Ok(self.fans_names.borrow().to_owned())
    }
}

/// Create the D-Bus connection to listen incoming requests.
pub(crate) fn create_dbus_conn(data: Rc<State>) -> Result<LocalConnection, DBusError> {
    let fac = Factory::new_fn::<TData>();
    let tree = fac
        .tree(Rc::clone(&data))
        .add(
            fac.object_path(OBJ_PATH_STR, ())
                .introspectable()
                .add(com_musikid_fancy_server(&fac, (), |m| {
                    let d: &State = Rc::borrow(m.tree.get_data());
                    d
                })),
        )
        // This path is for debugging
        .add(fac.object_path("/", ()).introspectable());

    let c = LocalConnection::new_system().context(DBus {})?;
    c.request_name(BUS_NAME_STR, true, true, false)
        .context(DBus {})?;
    tree.start_receive(&c);

    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn getters() {
        let dummy_fans_speeds = vec![50., 35., 23.];
        let dummy_target_fans_speeds = vec![90., 85., 75.];
        let dummy_temps: HashMap<String, f64> = vec![("CPU".to_owned(), 26.)].into_iter().collect();
        let dummy_config = String::from("Dummy config");
        let state = State {
            ec_access_mode: RefCell::new(crate::config::service::ECAccessMode::Either),
            fans_speeds: RefCell::new(dummy_fans_speeds.clone()),
            target_fans_speeds: RefCell::new(dummy_target_fans_speeds.clone()),
            auto: RefCell::new(true),
            critical: RefCell::new(false),
            config: RefCell::new(dummy_config),
            temps: RefCell::new(dummy_temps.clone()),
            poll_interval: RefCell::new(0),
            fans_names: RefCell::new(vec!["dummy".to_string()]),
            check_control_config: RefCell::new(false),
            ..Default::default()
        };

        assert_eq!(state.fans_speeds().unwrap(), dummy_fans_speeds);
        assert_eq!(
            state.target_fans_speeds().unwrap(),
            dummy_target_fans_speeds
        );
        assert_eq!(&*state.config().unwrap(), "Dummy config");
        assert_eq!(state.critical().unwrap(), false);
        assert_eq!(state.auto().unwrap(), true);
        assert_eq!(state.temperatures().unwrap(), dummy_temps);
        assert_eq!(state.poll_interval().unwrap(), 0);
        assert_eq!(state.fans_names().unwrap(), vec!["dummy".to_string()]);
    }

    #[test]
    fn setters() {
        let dummy_target_fans_speeds = vec![90., 85., 75.];
        // let dummy_config = String::from("Dummy config");
        let state = State {
            target_fans_speeds: RefCell::from(vec![0., 0., 0.]),
            fans_speeds: RefCell::from(vec![0., 0., 0.]),
            ..Default::default()
        };

        assert!(state
            .set_target_fans_speeds(dummy_target_fans_speeds.clone())
            .is_ok());
        assert_eq!(
            &*state.target_fans_speeds.borrow(),
            &dummy_target_fans_speeds
        );

        assert!(state.set_auto(true).is_ok());
        assert_eq!(*state.auto.borrow(), true);
    }

    #[test]
    fn set_target_fans_speeds() {
        let state = State {
            target_fans_speeds: RefCell::from(vec![0., 0., 0.]),
            fans_speeds: RefCell::from(vec![0., 0., 0.]),
            ..Default::default()
        };
        let dummy_target_speeds = vec![3., 2., 10.];

        assert!(state
            .set_target_fans_speeds(dummy_target_speeds.clone())
            .is_ok());
        assert!(*state.target_fans_speeds.borrow() == dummy_target_speeds);

        let invalid_target_speeds = vec![102., 1023., 1244.];
        assert!(state.set_target_fans_speeds(invalid_target_speeds).is_err());

        let invalid_number_speeds = vec![0.];
        assert!(state.set_target_fans_speeds(invalid_number_speeds).is_err());
    }

    #[test]
    fn out_of_bounds_target_speeds() {
        let state = State {
            ..Default::default()
        };
        let dummy_target_speeds = vec![-35., 0., 100., 24.54, 26.12];

        assert!(state.set_target_fans_speeds(dummy_target_speeds).is_err());
    }

    //   #[test]
    //   fn connecting() {
    //       use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
    //       use std::sync::Rc;
    //       use std::thread::spawn;
    //
    //       let state = Rc::from(State {
    //           target_fans_speeds: RefCell::new([42.0].to_vec()),
    //           ..Default::default()
    //       });
    //       let _conn = create_dbus_conn(Rc::clone(&state)).unwrap();
    //
    //       let t = spawn(|| {
    //           let client_conn = dbus::blocking::LocalConnection::new_system().unwrap();
    //           let _proxy = client_conn.with_proxy(
    //               "com.musikid.fancy",
    //               "/com/musikid/fancy",
    //               std::time::Duration::from_millis(1),
    //           );
    //       });
    //
    //       t.join().unwrap();
    //   }
    //
    //   #[test]
    //   fn test_client() {
    //       use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
    //       use std::sync::Rc;
    //       use std::thread::spawn;
    //       use std::time::Duration;
    //
    //       let state = Rc::from(State {
    //           target_fans_speeds: RefCell::new([42.0].to_vec()),
    //           ..Default::default()
    //       });
    //       let mut conn = create_dbus_conn(Rc::clone(&state)).unwrap();
    //
    //       let res = spawn(|| {
    //           let client_conn = dbus::blocking::LocalConnection::new_system().unwrap();
    //           let proxy = client_conn.with_proxy(
    //               "com.musikid.fancy",
    //               "/com/musikid/fancy",
    //               std::time::Duration::from_millis(500),
    //           );
    //
    //           let (vec,): (Vec<f64>,) = proxy.get("com.musikid.fancy", "TargetFansSpeeds").unwrap();
    //           vec
    //       });
    //       assert!(conn.process(Duration::from_millis(15)).unwrap());
    //       let vec = res.join().unwrap();
    //       assert_eq!(vec, vec![42.0]);
    //   }
}
