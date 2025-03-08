use std::io::{Read, Write};

pub(crate) trait ReadExt {
    fn read_sized(&mut self, size: usize) -> std::io::Result<Vec<u8>>;
    fn read_u8(&mut self) -> std::io::Result<u8>;
    fn read_u16(&mut self) -> std::io::Result<u16>;
    fn read_i16(&mut self) -> std::io::Result<i16>;
    fn read_u32(&mut self) -> std::io::Result<u32>;
    fn read_string(&mut self, size: usize) -> std::io::Result<String>;
}

impl<T: ?Sized + Read> ReadExt for T {
    fn read_sized(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; size];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_u8(&mut self) -> std::io::Result<u8> {
        let buf = self.read_sized(1)?;
        Ok(u8::from_le_bytes(buf.try_into().unwrap()))
    }

    fn read_u16(&mut self) -> std::io::Result<u16> {
        let buf = self.read_sized(2)?;
        Ok(u16::from_le_bytes(buf.try_into().unwrap()))
    }

    fn read_i16(&mut self) -> std::io::Result<i16> {
        let buf = self.read_sized(2)?;
        Ok(i16::from_le_bytes(buf.try_into().unwrap()))
    }

    fn read_u32(&mut self) -> std::io::Result<u32> {
        let buf = self.read_sized(4)?;
        Ok(u32::from_le_bytes(buf.try_into().unwrap()))
    }

    fn read_string(&mut self, size: usize) -> std::io::Result<String> {
        let buf = self.read_sized(size)?;
        Ok(core::str::from_utf8(&buf).unwrap().to_string())
    }
}

pub(crate) trait WriteExt {
    #[allow(dead_code)]
    fn write_u8(&mut self, value: u8) -> std::io::Result<()>;
    fn write_u16(&mut self, value: u16) -> std::io::Result<()>;
    fn write_i16(&mut self, value: i16) -> std::io::Result<()>;
    fn write_u32(&mut self, value: u32) -> std::io::Result<()>;
    fn write_string(&mut self, s: &str) -> std::io::Result<()>;
}

impl<T: Write> WriteExt for T {
    fn write_u8(&mut self, value: u8) -> std::io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_u16(&mut self, value: u16) -> std::io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_i16(&mut self, value: i16) -> std::io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_u32(&mut self, value: u32) -> std::io::Result<()> {
        self.write_all(&value.to_le_bytes())
    }

    fn write_string(&mut self, s: &str) -> std::io::Result<()> {
        self.write_all(s.as_bytes())
    }
}
