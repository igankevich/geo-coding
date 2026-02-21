pub trait Write {
    fn write_u32(&mut self, value: u32) -> std::io::Result<()> {
        self.write_bytes(&value.to_le_bytes()[..])
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()>;

    fn write_sign_magnitude(
        &mut self,
        values: impl IntoIterator<Item = i64> + Clone,
    ) -> std::io::Result<()>;

    fn write_magnitude_monotonic(
        &mut self,
        values: impl IntoIterator<Item = u32> + Clone,
    ) -> std::io::Result<()>;

    fn write_magnitude(
        &mut self,
        values: impl IntoIterator<Item = u32> + Clone,
    ) -> std::io::Result<()>;
}

impl<W: std::io::Write> Write for W {
    fn write_bytes(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.write_all(bytes)
    }

    fn write_sign_magnitude(
        &mut self,
        values: impl IntoIterator<Item = i64> + Clone,
    ) -> std::io::Result<()> {
        let mut count = 0;
        let num_zeros = {
            let mut prev = 0_i64;
            let mut min_clz = u64::BITS;
            for v in values.clone().into_iter() {
                let magnitude = v.wrapping_sub(prev).unsigned_abs();
                let clz = magnitude.leading_zeros();
                if clz < min_clz {
                    min_clz = clz;
                }
                prev = v;
                count += 1;
            }
            min_clz
        };
        if count == 0 {
            return Ok(());
        }
        let num_bytes = (u64::BITS - num_zeros).div_ceil(u8::BITS);
        self.write_all(&[num_bytes as u8])?;
        // Write magnitudes.
        {
            let mut prev = 0_i64;
            for v in values.clone().into_iter() {
                let magnitude = v.wrapping_sub(prev).unsigned_abs();
                self.write_all(&magnitude.to_le_bytes()[..num_bytes as usize])?;
                prev = v;
            }
        }
        // Write signs.
        {
            let mut prev = 0_i64;
            for v in values.into_iter() {
                let sign = if v.wrapping_sub(prev) < 0 { 1_u8 } else { 0_u8 };
                self.write_all(&[sign])?;
                prev = v;
            }
        }
        Ok(())
    }

    fn write_magnitude_monotonic(
        &mut self,
        values: impl IntoIterator<Item = u32> + Clone,
    ) -> std::io::Result<()> {
        let mut count = 0;
        let num_zeros = {
            let mut prev = 0_u32;
            let mut min_clz = u32::BITS;
            for v in values.clone().into_iter() {
                let magnitude = v.wrapping_sub(prev);
                let clz = magnitude.leading_zeros();
                if clz < min_clz {
                    min_clz = clz;
                }
                prev = v;
                count += 1;
            }
            min_clz
        };
        if count == 0 {
            return Ok(());
        }
        let num_bytes = (u32::BITS - num_zeros).div_ceil(u8::BITS);
        self.write_all(&[num_bytes as u8])?;
        if num_bytes == 0 {
            return Ok(());
        }
        // Write magnitudes.
        {
            let mut prev = 0_u32;
            for v in values.clone().into_iter() {
                let magnitude = v.wrapping_sub(prev);
                self.write_all(&magnitude.to_le_bytes()[..num_bytes as usize])?;
                prev = v;
            }
        }
        Ok(())
    }

    fn write_magnitude(
        &mut self,
        values: impl IntoIterator<Item = u32> + Clone,
    ) -> std::io::Result<()> {
        let mut count = 0;
        let num_zeros = {
            let mut min_clz = u32::BITS;
            for v in values.clone().into_iter() {
                let magnitude = v;
                let clz = magnitude.leading_zeros();
                if clz < min_clz {
                    min_clz = clz;
                }
                count += 1;
            }
            min_clz
        };
        if count == 0 {
            return Ok(());
        }
        let num_bytes = (u32::BITS - num_zeros).div_ceil(u8::BITS);
        self.write_all(&[num_bytes as u8])?;
        if num_bytes == 0 {
            return Ok(());
        }
        // Write magnitudes.
        {
            for v in values.clone().into_iter() {
                let magnitude = v;
                self.write_all(&magnitude.to_le_bytes()[..num_bytes as usize])?;
            }
        }
        Ok(())
    }
}
