use alloc::vec::Vec;
use core::mem::size_of;

pub trait Read {
    fn read_u32(&mut self) -> std::io::Result<u32> {
        let mut buf = [0; 4];
        self.read_bytes(&mut buf[..])?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_bytes(&mut self, buf: &mut [u8]) -> std::io::Result<()>;
    fn read_sign_magnitude(&mut self, count: usize) -> std::io::Result<Vec<i64>>;
    fn read_magnitude_monotonic(&mut self, count: usize) -> std::io::Result<Vec<u32>>;
    fn read_magnitude(&mut self, count: usize) -> std::io::Result<Vec<u32>>;
}

impl<R: std::io::Read> Read for R {
    fn read_bytes(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.read_exact(buf)
    }

    fn read_sign_magnitude(&mut self, count: usize) -> std::io::Result<Vec<i64>> {
        const MAX_BYTES: usize = size_of::<u64>();
        let mut values = Vec::with_capacity(count);
        if count == 0 {
            return Ok(values);
        }
        let mut buf = [0_u8; MAX_BYTES];
        self.read_exact(&mut buf[..1])?;
        let num_bytes = buf[0];
        if num_bytes == 0 || usize::from(num_bytes) > MAX_BYTES {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        // Read magnitudes.
        for _ in 0..count {
            self.read_exact(&mut buf[..usize::from(num_bytes)])?;
            let magnitude = u64::from_le_bytes(buf);
            values.push(magnitude as i64);
        }
        // Read signs.
        let mut prev = 0_i64;
        for magnitude in values.iter_mut() {
            self.read_exact(&mut buf[..1])?;
            let sign = buf[0];
            match sign {
                0 => {}
                1 if *magnitude == i64::MIN => {}
                1 => *magnitude = -*magnitude,
                _ => return Err(std::io::ErrorKind::InvalidData.into()),
            }
            *magnitude = magnitude.wrapping_add(prev);
            prev = *magnitude;
        }
        Ok(values)
    }

    fn read_magnitude_monotonic(&mut self, count: usize) -> std::io::Result<Vec<u32>> {
        const MAX_BYTES: usize = size_of::<u32>();
        let mut values = Vec::with_capacity(count);
        if count == 0 {
            return Ok(values);
        }
        let mut buf = [0_u8; MAX_BYTES];
        self.read_exact(&mut buf[..1])?;
        let num_bytes = buf[0];
        if usize::from(num_bytes) > MAX_BYTES {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        if num_bytes == 0 {
            values.resize(count, 0_u32);
        }
        // Read magnitudes.
        let mut prev = 0_u32;
        for _ in 0..count {
            self.read_exact(&mut buf[..usize::from(num_bytes)])?;
            let magnitude = u32::from_le_bytes(buf).wrapping_add(prev);
            values.push(magnitude);
            prev = magnitude;
        }
        Ok(values)
    }

    fn read_magnitude(&mut self, count: usize) -> std::io::Result<Vec<u32>> {
        const MAX_BYTES: usize = size_of::<u32>();
        let mut values = Vec::with_capacity(count);
        if count == 0 {
            return Ok(values);
        }
        let mut buf = [0_u8; MAX_BYTES];
        self.read_exact(&mut buf[..1])?;
        let num_bytes = buf[0];
        if usize::from(num_bytes) > MAX_BYTES {
            return Err(std::io::ErrorKind::InvalidData.into());
        }
        if num_bytes == 0 {
            values.resize(count, 0_u32);
        }
        // Read magnitudes.
        for _ in 0..count {
            self.read_exact(&mut buf[..usize::from(num_bytes)])?;
            let magnitude = u32::from_le_bytes(buf);
            values.push(magnitude);
        }
        Ok(values)
    }
}
