/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
pub mod ec_manager;
mod read;
mod write;
use std::io::{Read, Seek, Write};

type RcWrapper<T> = std::rc::Rc<std::cell::RefCell<T>>;

pub(crate) trait RW: Read + Write + Seek + std::fmt::Debug {}
impl<T: Read + Write + Seek + std::fmt::Debug> RW for T {}

// For the cleaner.
pub(crate) use write::ECWriter;
