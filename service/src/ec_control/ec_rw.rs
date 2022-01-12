use async_std::fs::{File, OpenOptions};
use async_std::io::prelude::*;
use async_std::io::{Read, Result, Seek, SeekFrom, Write};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use std::fmt::Debug;
use std::path::Path;

use crate::RawPort;

const EC_SYS_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/sys/kernel/debug/ec/ec0/io"));
const ACPI_EC_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/dev/ec"));
const PORT_DEV_PATH: Lazy<&Path> = Lazy::new(|| Path::new("/dev/port"));

pub trait RW: Read + Write + Send + Sync + Seek + Unpin + Debug {}
impl<T: Read + Write + Seek + Send + Sync + Unpin + Debug> RW for T {}

//TODO: Use integers for offsets

#[async_trait]
pub trait EcRead {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()>;
}

#[async_trait]
pub trait EcWrite {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &[u8]) -> Result<()>;
}

#[async_trait]
pub trait EcRW: EcRead + EcWrite {}
impl<T: EcRead + EcWrite> EcRW for T {}

#[async_trait]
impl<T: RW> EcRead for T {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()> {
        self.seek(offset).await?;
        self.read_exact(bytes).await
    }
}

#[async_trait]
impl<T: RW> EcWrite for T {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &[u8]) -> Result<()> {
        self.seek(offset).await?;
        self.write_all(bytes).await
    }
}

#[async_trait]
impl<T: RW> EcRead for RawPort<T> {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()> {
        if let SeekFrom::Start(offset) = offset {
            self.pos = offset as u8;

            for byte in bytes.iter_mut() {
                //TODO: Reading from /dev/port can sometimes be capricious
                *byte = self.ec_read_byte(self.pos).await?;
                self.pos += 1;
            }

            Ok(())
        } else {
            Err(async_std::io::Error::from(
                async_std::io::ErrorKind::InvalidInput,
            ))
        }
    }
}

#[async_trait]
impl<T: RW> EcWrite for RawPort<T> {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &[u8]) -> Result<()> {
        if let SeekFrom::Start(offset) = offset {
            self.pos = offset as u8;

            for byte in bytes {
                //TODO: Writing to /dev/port can sometimes be capricious
                self.ec_write_byte(self.pos, *byte).await?;
                self.pos += 1;
            }

            Ok(())
        } else {
            Err(async_std::io::Error::from(
                async_std::io::ErrorKind::InvalidInput,
            ))
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
/// Describe the way to access to the EC.
pub enum EcAccessMode {
    /// Access to the EC using the `/dev/port` file.
    RawPort,
    /// Access to the EC using the module `acpi_ec` (`/dev/ec`).
    AcpiEC,
    /// Access to the EC using the module `ec_sys` with `write_support=1`.
    ECSys,
}

//TODO: Does it make sense?
impl Default for EcAccessMode {
    fn default() -> Self {
        EcAccessMode::AcpiEC
    }
}

impl EcAccessMode {
    /// Get path corresponding to the access mode.
    pub fn path(&self) -> &'static Path {
        match self {
            EcAccessMode::RawPort => *PORT_DEV_PATH,
            EcAccessMode::AcpiEC => *ACPI_EC_DEV_PATH,
            EcAccessMode::ECSys => *EC_SYS_DEV_PATH,
        }
    }
}

#[derive(Debug)]
pub enum EcAccess<R: RW, E: EcRW> {
    RawPort(RawPort<R>, EcAccessMode),
    Direct(E, EcAccessMode),
}

impl<R: RW, E: EcRW> EcAccess<R, E> {
    pub fn mode(&self) -> EcAccessMode {
        match self {
            EcAccess::Direct(_, mode) | EcAccess::RawPort(_, mode) => *mode,
        }
    }
}

#[async_trait]
impl<R: RW + Send, E: EcRW + Send> EcWrite for EcAccess<R, E> {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &[u8]) -> Result<()> {
        (match self {
            EcAccess::RawPort(p, ..) => p.write_bytes(offset, bytes),
            EcAccess::Direct(f, ..) => f.write_bytes(offset, bytes),
        })
        .await
    }
}

#[async_trait]
impl<R: RW + Send, E: EcRW + Send> EcRead for EcAccess<R, E> {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()> {
        (match self {
            EcAccess::RawPort(p, ..) => p.read_bytes(offset, bytes),
            EcAccess::Direct(f, ..) => f.read_bytes(offset, bytes),
        })
        .await
    }
}

impl EcAccess<File, File> {
    pub async fn from_mode(mode: EcAccessMode) -> Result<EcAccess<File, File>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(mode.path())
            .await?;

        Ok(match mode {
            EcAccessMode::AcpiEC | EcAccessMode::ECSys => EcAccess::Direct(file, mode),
            EcAccessMode::RawPort => EcAccess::RawPort(RawPort::from(file), mode),
        })
    }

    pub async fn try_default() -> Result<EcAccess<File, File>> {
        const MODES: &[EcAccessMode] = &[
            EcAccessMode::AcpiEC,
            EcAccessMode::ECSys,
            EcAccessMode::RawPort,
        ];

        for mode in MODES {
            let mode = EcAccess::from_mode(*mode).await;
            if mode.is_ok() {
                return mode;
            }
        }

        Err(async_std::io::Error::from(
            async_std::io::ErrorKind::NotFound,
        ))
    }
}
