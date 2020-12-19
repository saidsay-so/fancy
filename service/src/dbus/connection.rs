/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use dbus_crate::blocking::LocalConnection;
use dbus_crate::tree::{DataType, Factory, MethodErr};
use snafu::{ResultExt, Snafu};

use super::interfaces::*;
use crate::{config::nbfc_control::test_load_control_config, constants::OBJ_PATH_STR, State};

use std::collections::HashMap;
use std::{borrow::Borrow, rc::Rc};

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
    DBus { source: dbus_crate::Error },
}

impl From<DBusError> for MethodErr {
    fn from(e: DBusError) -> Self {
        MethodErr::failed(&e)
    }
}

impl ComMusikidFancy for State {
    fn fans_speeds(&self) -> Result<HashMap<String, f64>, MethodErr> {
        Ok(self.fans_speeds.borrow().to_owned())
    }
    fn target_fans_speeds(&self) -> Result<Vec<f64>, MethodErr> {
        Ok(self.target_fans_speeds.borrow().to_owned())
    }
    fn set_target_fans_speeds(&self, mut value: Vec<f64>) -> Result<(), MethodErr> {
        // From https://rust-num.github.io/num/src/num_traits/lib.rs.html#329-338
        #[inline]
        fn clamp<T: PartialOrd>(input: T, min: T, max: T) -> T {
            assert!(min <= max, "min must be less than or equal to max");
            if input < min {
                min
            } else if input > max {
                max
            } else {
                input
            }
        }

        value.iter_mut().for_each(|v| *v = clamp(*v, 0., 100.));
        *self.target_fans_speeds.borrow_mut() = value;
        Ok(())
    }
    fn config(&self) -> Result<String, MethodErr> {
        Ok(self.config.borrow().to_owned())
    }
    fn critical(&self) -> Result<bool, MethodErr> {
        Ok(*self.critical.borrow())
    }
    fn set_config(&self, value: String) -> Result<(), MethodErr> {
        if test_load_control_config(&value).is_ok() {
            *self.config.borrow_mut() = value;
            Ok(())
        } else {
            Err(MethodErr::failed(&format!(
                "`{}` is not in configs path",
                value
            )))
        }
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
    c.request_name("com.musikid.fancy", true, true, false)
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
        let dummy_fans_speeds: HashMap<String, f64> = vec![
            ("Dummy0".to_owned(), 50.),
            ("Dummy1".to_owned(), 35.),
            ("Dummy2".to_owned(), 23.),
        ]
        .into_iter()
        .collect();
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
    }

    #[test]
    fn setters() {
        let dummy_target_fans_speeds = vec![90., 85., 75.];
        // let dummy_config = String::from("Dummy config");
        let state = State {
            ..Default::default()
        };

        assert!(state
            .set_target_fans_speeds(dummy_target_fans_speeds.clone())
            .is_ok());
        // assert!(state.set_config(dummy_config.clone()).is_ok());
        assert!(state.set_auto(true).is_ok());

        assert_eq!(
            state.target_fans_speeds.borrow().clone(),
            dummy_target_fans_speeds
        );
        // assert_eq!(
        //     block_on(async { state.config.read().clone() }),
        //     dummy_config
        // );
        assert_eq!(*state.auto.borrow(), true);
    }

    #[test]
    fn set_target_fans_speeds() {
        let state = State {
            ..Default::default()
        };
        let dummy_target_speeds = vec![3., 2., 10., 24., 26.];
        assert!(state
            .set_target_fans_speeds(dummy_target_speeds.clone())
            .is_ok());

        assert!(*state.target_fans_speeds.borrow() == dummy_target_speeds);
    }

    #[test]
    fn out_of_bounds_target_speeds() {
        let state = State {
            ..Default::default()
        };
        let dummy_target_speeds = vec![-35., 0., 100., 24.54, 26.12];

        assert!(state
            .set_target_fans_speeds(dummy_target_speeds.clone())
            .is_ok());

        assert!(state
            .target_fans_speeds
            .borrow()
            .iter()
            .all(|&el| 0.0 <= el && el <= 100.0));
    }

    //   #[test]
    //   fn connecting() {
    //       use dbus_crate::blocking::stdintf::org_freedesktop_dbus::Properties;
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
    //           let client_conn = dbus_crate::blocking::LocalConnection::new_system().unwrap();
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
    //       use dbus_crate::blocking::stdintf::org_freedesktop_dbus::Properties;
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
    //           let client_conn = dbus_crate::blocking::LocalConnection::new_system().unwrap();
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
