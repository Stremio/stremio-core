use std::io::Write;
use std::io::Read;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

#[derive(Debug, Clone)]
pub struct BitField8 {
    pub length: usize,
    values: Vec<u8>,
}
impl BitField8 {
    pub fn from_size(len: usize) -> BitField8 {
        let length = (len as f64 / 8.0).ceil() as usize;
        BitField8 {
            length,
            values: vec![0; length],
        }
    }
    pub fn from_packed(compressed: Vec<u8>, len: Option<usize>) -> Result<BitField8, String> {
        let mut values = vec![];
        let mut z = ZlibDecoder::new(&compressed[..]);
        z.read_to_end(&mut values).map_err(|e| e.to_string())?;

        let length = if let Some(len) = len {
            len
        } else {
            values.len()
        };
        let bytes = (length as f64 / 8.0).ceil() as usize;
        if bytes > values.len() {
            values = [values.clone(), vec![0; bytes - values.len()]].concat();
        }
        Ok(BitField8 { length, values })
    }

    pub fn to_packed(&self) -> Vec<u8> {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::new(6));
        e.write_all(&self.values).ok();
        e.finish().unwrap()
    }

    pub fn get(&self, i: usize) -> bool {
        let index = i / 8;
        let bit = i % 8;

        if index >= self.values.len() {
            false
        } else {
            (self.values[index] >> bit) & 1 != 0
        }
    }

    pub fn set(&mut self, i: usize, val: bool) {
        let index = i / 8;
        let mask = 1 << (i % 8);

        if index >= self.values.len() {
            self.values = [self.values.clone(), vec![0; index - self.values.len() + 1]].concat();
        }
        self.length = self.values.len() * 8;
        if val {
            self.values[index] |= mask;
        } else {
            self.values[index] &= !mask;
        }
    }

    pub fn last_index_of(&self, val: bool) -> Option<usize> {
        for i in (0..self.length - 1).rev() {
            if self.get(i) == val {
                return Some(i);
            }
        }
        None
    }
}
