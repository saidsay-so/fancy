/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
mod ec_manager;
mod ec_rw;
mod raw_port;
mod read;
mod write;

type ArcWrapper<T> = std::sync::Arc<async_std::sync::Mutex<T>>;

pub(crate) use ec_manager::{ECManager, EcManagerError, Event, EventSender, ExternalEvent};

pub(crate) use ec_rw::*;
pub(crate) use raw_port::RawPort;
