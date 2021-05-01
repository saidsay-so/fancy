/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use super::RW;

use std::io::{Error, Read, Seek, SeekFrom, Write};

#[derive(Copy, Clone, PartialEq)]
enum BufferEmpty {
    OutputBuffer = 0x01,
    InputBuffer = 0x02,
}

const EC_COMMAND_READ: u8 = 0x80;
const EC_COMMAND_WRITE: u8 = 0x81;

const RAW_PORT_TIMEOUT: u16 = 2000;

const COMMAND_PORT: SeekFrom = SeekFrom::Start(0x66);
const DATA_PORT: SeekFrom = SeekFrom::Start(0x62);

type Result<T = ()> = std::result::Result<T, Error>;

/// Wraps reads and writes to `/dev/port`.
/// `/dev/port` is mapped to I/O ports, and thus we need an abstraction layer for reading/writing from/to the EC.
#[derive(Debug)]
pub(crate) struct RawPort<T: RW> {
    inner: T,
    pos: u8,
}

impl<T: RW> From<T> for RawPort<T> {
    fn from(inner: T) -> Self {
        RawPort { inner, pos: 0 }
    }
}

impl<T: RW> RawPort<T> {
    /// Low-level wait function before reading/writing to `/dev/port`.
    ///
    /// It waits for input/output buffer to be empty and return an error on timeout.
    fn raw_port_wait(&mut self, buffer_type: BufferEmpty) -> Result {
        let mut retries = RAW_PORT_TIMEOUT;
        while retries > 0 {
            retries -= 1;

            let mut value = [0u8; 1];
            self.inner.seek(COMMAND_PORT)?;
            self.inner.read(&mut value)?;

            let mut value = value[0];
            // Invert the value for output buffer.
            if buffer_type == BufferEmpty::OutputBuffer {
                value = !value;
            }

            if (value & (buffer_type as u8)) == 0 {
                return Ok(());
            }
        }

        Err(Error::from(std::io::ErrorKind::TimedOut))
    }

    /// Read variant of [`raw_port_wait`](#method.raw_port_wait).
    fn raw_port_wait_read(&mut self) -> Result {
        self.raw_port_wait(BufferEmpty::OutputBuffer)
    }

    /// Write variant of [`raw_port_wait`](#method.raw_port_wait).
    fn raw_port_wait_write(&mut self) -> Result {
        self.raw_port_wait(BufferEmpty::InputBuffer)
    }

    /// Write data (a query) to a port.
    fn raw_port_query(&mut self, port: SeekFrom, query: u8) -> Result {
        self.raw_port_wait_write()?;
        self.inner.seek(port)?;
        self.inner.write_all(&[query])
    }

    /// Read a byte from the EC at `offset`.
    fn ec_read_byte(&mut self, offset: u8) -> Result<u8> {
        self.raw_port_query(COMMAND_PORT, EC_COMMAND_READ)?;
        self.raw_port_query(DATA_PORT, offset)?;

        self.raw_port_wait_read()?;
        self.raw_port_wait_write()?;

        let mut byte = [0u8; 1];
        self.inner.seek(DATA_PORT)?;
        self.inner.read_exact(&mut byte)?;

        Ok(byte[0])
    }

    /// Write a byte to the EC at `offset`.
    fn ec_write_byte(&mut self, offset: u8, byte: u8) -> Result {
        self.raw_port_query(COMMAND_PORT, EC_COMMAND_WRITE)?;
        self.raw_port_query(DATA_PORT, offset)?;

        self.raw_port_query(DATA_PORT, byte)
    }
}

impl<T: RW> Write for RawPort<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        for byte in buf {
            //TODO: Writing to /dev/port can sometimes be capricious
            self.ec_write_byte(self.pos, *byte)?;
            self.pos += 1;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: RW> Read for RawPort<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        for byte in buf.iter_mut() {
            //TODO: Reading from /dev/port can sometimes be capricious
            *byte = self.ec_read_byte(self.pos)?;
            self.pos += 1;
        }
        Ok(buf.len())
    }
}

impl<T: RW> Seek for RawPort<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match pos {
            SeekFrom::Start(pos) => {
                self.pos = pos as u8;
            }
            _ => {
                return Err(Error::from(std::io::ErrorKind::InvalidInput));
            }
        }

        Ok(self.pos.into())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        io::{Read, Seek, Write},
    };

    use super::*;

    const COMMAND_PORT_UINT: u8 = 0x66;
    const DATA_PORT_UINT: u8 = 0x62;

    /// Emulates EC access through `/dev/port`.
    #[derive(Debug)]
    struct BufferTest {
        pub reads: Vec<(u8, u8)>,
        pub writes: Vec<(u8, u8)>,
        pub registers: HashMap<u8, u8>,
        pub register: Option<u8>,
        pub full_input: bool,
        pub full_output: bool,
        pos: u8,
    }

    impl BufferTest {
        fn new() -> BufferTest {
            BufferTest {
                writes: Vec::new(),
                reads: Vec::new(),
                registers: HashMap::new(),
                register: None,
                full_input: false,
                full_output: false,
                pos: 0,
            }
        }
    }

    impl Write for BufferTest {
        fn write(&mut self, buf: &[u8]) -> Result<usize> {
            assert!(buf.len() == 1);
            if self.pos == DATA_PORT_UINT && self.register.is_none() {
                self.register = Some(buf[0]);
            } else if let Some(register) = self.register {
                self.registers.insert(register, buf[0]);
                self.register = None;
            }
            self.writes.push((self.pos, buf[0]));
            Ok(1)
        }

        fn flush(&mut self) -> Result<()> {
            Ok(())
        }
    }

    impl Read for BufferTest {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            assert!(buf.len() == 1);
            buf[0] = if self.pos == COMMAND_PORT_UINT {
                let input_status = if self.full_input {
                    BufferEmpty::InputBuffer as u8
                } else {
                    !(BufferEmpty::InputBuffer as u8)
                };
                let output_status = if self.full_output {
                    !(BufferEmpty::OutputBuffer as u8)
                } else {
                    BufferEmpty::OutputBuffer as u8
                };

                input_status | output_status
            } else if let Some(register) = self.register {
                *self.registers.get(&register).unwrap()
            } else {
                unreachable!()
            };
            self.reads.push((self.pos, buf[0]));
            Ok(1)
        }
    }

    impl Seek for BufferTest {
        fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
            if let SeekFrom::Start(pos) = pos {
                self.pos = pos as u8;
                return Ok(self.pos as u64);
            }

            unreachable!()
        }
    }

    #[test]
    fn wait_before_write() {
        let mut buffer = BufferTest::new();
        buffer.full_input = true;

        let mut raw_port = RawPort::from(&mut buffer);

        let result = raw_port.write(&[0]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::TimedOut);

        assert_eq!(buffer.reads.len(), RAW_PORT_TIMEOUT as usize);
        assert!(buffer.reads.iter().all(|e| e.0 == COMMAND_PORT_UINT as u8));
    }

    #[test]
    fn wait_before_read() {
        let mut buffer = BufferTest::new();
        buffer.full_output = true;

        let mut raw_port = RawPort::from(&mut buffer);

        let result = raw_port.read(&mut [0]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::TimedOut);

        assert_eq!(buffer.reads.len(), RAW_PORT_TIMEOUT as usize);
        assert!(buffer.reads.iter().all(|e| e.0 == COMMAND_PORT_UINT as u8));
    }

    #[test]
    fn write_offset_value() {
        let mut buffer = BufferTest::new();

        let mut raw_port = RawPort::from(&mut buffer);

        let excepted_register = 23;
        let excepted_value = 200;
        raw_port
            .seek(SeekFrom::Start(excepted_register as u64))
            .unwrap();
        raw_port.write(&[excepted_value]).unwrap();

        assert_eq!(
            *buffer.registers.get(&excepted_register).unwrap(),
            excepted_value
        );

        assert!(buffer.reads.len() == 3);

        assert!(buffer.reads.iter().all(|e| e.0 == COMMAND_PORT_UINT));

        assert_eq!(buffer.writes[0].0, COMMAND_PORT_UINT);
        assert_eq!(buffer.writes[0].1, EC_COMMAND_WRITE);

        assert_eq!(buffer.writes[1].0, DATA_PORT_UINT);
        assert_eq!(buffer.writes[1].1, excepted_register);

        assert_eq!(buffer.writes[2].0, DATA_PORT_UINT);
        assert_eq!(buffer.writes[2].1, excepted_value);
    }

    #[test]
    fn read_offset_value() {
        let mut buffer = BufferTest::new();
        let excepted_value = 50;
        let excepted_register = 37;
        buffer.registers.insert(excepted_register, excepted_value);

        let mut raw_port = RawPort::from(&mut buffer);

        let mut value = [0u8; 1];
        raw_port
            .seek(SeekFrom::Start(excepted_register as u64))
            .unwrap();
        raw_port.read(&mut value).unwrap();
        let value = value[0];
        assert_eq!(value, excepted_value);

        //TODO: Check the reads/writes
        assert!(buffer.reads.len() == 5);

        assert_eq!(
            *buffer.writes.iter().skip(1).next().unwrap(),
            (DATA_PORT_UINT, excepted_register)
        );
        assert_eq!(
            *buffer.reads.last().unwrap(),
            (DATA_PORT_UINT, excepted_value)
        );
    }
}
