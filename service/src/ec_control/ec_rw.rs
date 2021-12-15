use async_std::fs::File;
use async_std::io::{Read, Result, Seek, SeekFrom, Write};
use async_std::io::prelude::*;
use async_std::prelude::*;
use async_trait::async_trait;

use crate::RawPort;

pub(crate) enum EcAccess<R: RW> {
    RawPort(RawPort<R>),
    AcpiEc(File),
    EcSys(File),
}


impl<R: RW> EcWrite for EcAccess<R> {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()> {
        match self {
            EcAccess::RawPort(p) => p.write_bytes(offset, bytes),
            EcAccess::AcpiEc(f) => f.write_bytes(offset, bytes),
            EcAccess::EcSys(f) => f.write_bytes(offset, bytes)
        }
    }
}

impl<R: RW> EcRead for EcAccess<R> {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()> {
        match self {
            EcAccess::RawPort(p) => p.read_bytes(offset, bytes),
            EcAccess::AcpiEc(f) => f.read_bytes(offset, bytes),
            EcAccess::EcSys(f) => f.read_bytes(offset, bytes)
        }
    }
}

#[async_trait]
pub(crate) trait EcRead {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()>;
}

#[async_trait]
pub(crate) trait EcWrite {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &[u8]) -> Result<()>;
}

#[async_trait]
pub(crate) trait EcRW: EcRead + EcWrite {}

#[async_trait]
impl EcRead for File {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()> {
        self.seek(offset).await?;
        self.read_exact(bytes).await
    }
}

#[async_trait]
impl EcWrite for File {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &[u8]) -> Result<()> {
        self.seek(offset).await?;
        self.write_all(bytes).await
    }
}

pub(crate) trait RW: Read + Write + Seek + Unpin + std::fmt::Debug {}

impl<T: Read + Write + Seek + Unpin + std::fmt::Debug> RW for T {}

#[async_trait]
impl<T: RW> EcRead for RawPort<T> {
    async fn read_bytes(&mut self, offset: SeekFrom, bytes: &mut [u8]) -> Result<()> {
        let SeekFrom::Start(offset) = offset;
        self.pos = offset as u8;

        for byte in bytes.iter_mut() {
            //TODO: Reading from /dev/port can sometimes be capricious
            *byte = self.ec_read_byte(self.pos).await?;
            self.pos += 1;
        }

        Ok(())
    }
}

#[async_trait]
impl<T: RW> EcWrite for RawPort<T> {
    async fn write_bytes(&mut self, offset: SeekFrom, bytes: &[u8]) -> Result<()> {
        let SeekFrom::Start(offset) = offset;
        self.pos = offset as u8;

        for byte in bytes {
            //TODO: Writing to /dev/port can sometimes be capricious
            self.ec_write_byte(self.pos, *byte).await?;
            self.pos += 1;
        }

        Ok(())
    }
}
