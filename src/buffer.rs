use std::convert::TryInto;

pub struct Buffer {
    buf: Vec<u8>,
    write_index: usize,
    read_index: usize,
}

macro_rules! to_ne_bytes {
    ($name:ident, $t:ty) => {
        pub fn $name(&mut self, elem: $t) {
            // const N: usize = std::mem::size_of::<$t>();
            let bytes = elem.to_be_bytes();
            // dbg!(bytes);
            self.append(&bytes);
        }
    };
}

macro_rules! from_ne_bytes {
    ($name:ident, $t:ty) => {
        pub fn $name(&mut self, is_consume: bool) -> Option<$t> {
            const N: usize = std::mem::size_of::<$t>();
            if self.readable_bytes() < N {
                return None;
            }
            let val = <$t>::from_be_bytes(self.as_slice()[0..N].try_into().unwrap());
            if is_consume {
                self.consume(N);
            }
            Some(val)
        }
    };
}

impl Buffer {
    pub fn new(cap: usize) -> Self {
        Buffer {
            buf: Vec::with_capacity(cap),
            write_index: 0,
            read_index: 0,
        }
    }

    pub fn readable_bytes(&self) -> usize {
        self.write_index - self.read_index
    }

    pub fn writeable_bytes(&self) -> usize {
        self.buf.capacity() - self.write_index
    }

    pub fn prependable_bytes(&self) -> usize {
        self.read_index
    }

    pub fn consume(&mut self, len: usize) {
        assert!(len <= self.readable_bytes());
        self.read_index += len;
        if self.read_index == self.write_index {
            self.consume_all();
        }
    }

    pub fn consume_all(&mut self) {
        self.read_index = 0;
        self.write_index = 0;
    }

    pub fn push(&mut self, elem: u8) {
        self.buf.push(elem);
        self.write_index += 1;
    }

    pub fn append(&mut self, data: &[u8]) {
        self.ensure_space(data.len());
        unsafe {
            let dest = self.buf.as_mut_ptr().add(self.write_index);
            std::ptr::copy(data.as_ptr(), dest, data.len());
            self.write_index += data.len();
            self.buf.set_len(self.write_index);
        };
    }

    fn make_space(&mut self, len: usize) {
        if self.writeable_bytes() + self.prependable_bytes() < len {
            self.buf.resize(self.write_index + len, 0);
        // dbg!(self.write_index + len, self.buf.capacity());
        } else {
            let readn = self.readable_bytes();
            let src: *const u8 = self.buf.as_mut_ptr();
            let src = unsafe { src.add(self.read_index) };
            unsafe {
                std::ptr::copy(src, self.buf.as_mut_ptr(), readn);
                self.buf.set_len(readn);
            }
            self.read_index = 0;
            self.write_index = self.read_index + readn;
        }
    }

    fn ensure_space(&mut self, len: usize) {
        if self.writeable_bytes() < len {
            self.make_space(len);
        }
        assert!(self.writeable_bytes() >= len);
    }

    pub fn retrieve_tovec(&mut self, len: usize) -> Vec<u8> {
        let len = std::cmp::min(len, self.readable_bytes());
        if len <= 0 {
            return vec![];
        }

        let mut ret = Vec::with_capacity(len);
        unsafe {
            std::ptr::copy(
                self.buf.as_ptr().add(self.read_index),
                ret.as_mut_ptr(),
                len,
            );
            ret.set_len(len);
        }
        self.read_index += len;
        ret
    }

    pub fn get_slice(&self, len: usize) -> &[u8] {
        let len = std::cmp::min(len, self.readable_bytes());
        &self.as_slice()[0..len]
    }

    pub fn as_slice(&self) -> &[u8] {
        // dbg!(
        //     self.read_index,
        //     self.write_index,
        //     self.buf.capacity(),
        //     self.buf.len()
        // );
        // dbg!(&self.buf);
        &self.buf.as_slice()[self.read_index..self.write_index]
    }

    to_ne_bytes!(append_u8, u8);
    to_ne_bytes!(append_i8, i8);
    to_ne_bytes!(append_u16, u16);
    to_ne_bytes!(append_i16, i16);
    to_ne_bytes!(append_u32, u32);
    to_ne_bytes!(append_i32, i32);

    to_ne_bytes!(append_f32, f32);
    to_ne_bytes!(append_f64, f64);

    from_ne_bytes!(read_u8, u8);
    from_ne_bytes!(read_u16, u16);
    from_ne_bytes!(read_u32, u32);
    from_ne_bytes!(read_i8, i8);
    from_ne_bytes!(read_i16, i16);
    from_ne_bytes!(read_i32, i32);

    from_ne_bytes!(read_f32, f32);
    from_ne_bytes!(read_f64, f64);
}

impl Extend<u8> for Buffer {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        for elem in iter {
            self.push(elem);
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn buffer_basic() {
        let mut buf = Buffer::new(10);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn buffer_make_space() {
        let mut buf = Buffer::new(1);
        buf.push(1);
        assert_eq!(buf.writeable_bytes(), 0);
        assert_eq!(buf.readable_bytes(), 1);
        buf.append(&[2, 3, 4, 5, 6]);
        assert_eq!(buf.readable_bytes(), 6);
        assert_eq!(buf.as_slice(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn buffer_consume() {
        let mut buf = Buffer::new(1);
        buf.append(&[2, 3, 4, 5, 6]);
        buf.consume(1);
        assert_eq!(buf.readable_bytes(), 4);
        assert_eq!(buf.as_slice(), &[3, 4, 5, 6]);
        buf.consume_all();
        assert_eq!(buf.readable_bytes(), 0);
        assert_eq!(buf.as_slice(), &[]);
    }

    #[test]
    fn buffer_move_space() {
        let mut buf = Buffer::new(10);
        buf.append(&[1, 2, 3, 4, 5, 6, 7]);
        buf.consume(2);
        buf.append(&[8, 9, 10, 11]);
        assert_eq!(buf.readable_bytes(), 9);
        assert_eq!(buf.writeable_bytes(), 1);
        assert_eq!(buf.prependable_bytes(), 0);
        assert_eq!(buf.as_slice(), &[3, 4, 5, 6, 7, 8, 9, 10, 11]);
    }

    #[test]
    fn buffer_append_int_size_test() {
        let mut buf = Buffer::new(10);
        buf.append_u8(0x12u8);
        buf.append_u16(0x3456u16);
        buf.append_u32(0x78123456u32);
        assert_eq!(buf.as_slice(), &[0x12, 0x34, 0x56, 0x78, 0x12, 0x34, 0x56]);
    }

    #[test]
    fn buffer_append_float_size_test() {
        let mut buf = Buffer::new(10);
        let val = 1.24f32;
        buf.append_f32(val);
        assert_eq!(buf.as_slice(), &val.to_be_bytes());
        buf.consume_all();

        let val = 1.244f64;
        buf.append_f64(val);
        assert_eq!(buf.as_slice(), &val.to_be_bytes());
    }

    #[test]
    fn buffer_read_size_test() {
        let mut buf = Buffer::new(10);
        buf.append_u8(0x12u8);
        buf.append_u16(0x3456u16);
        buf.append_u32(0x78123456u32);
        buf.append_f32(1.24f32);
        buf.append_f64(1.223f64);

        assert_eq!(buf.read_u8(false).unwrap(), 0x12u8);
        assert_eq!(buf.readable_bytes(), 19);

        assert_eq!(buf.read_u8(true).unwrap(), 0x12u8);
        assert_eq!(buf.read_u16(true).unwrap(), 0x3456u16);
        assert_eq!(buf.read_u32(true).unwrap(), 0x78123456u32);
        assert_eq!(buf.read_f32(true).unwrap(), 1.24f32);
        assert_eq!(buf.read_f64(true).unwrap(), 1.223f64);
    }

    #[test]
    fn buffer_retrieve_test() {
        let mut buf = Buffer::new(10);
        assert_eq!(buf.retrieve_tovec(10), vec![]);
        assert_eq!(buf.retrieve_tovec(0), vec![]);

        buf.append(&[1, 2, 3, 4]);
        assert_eq!(buf.retrieve_tovec(20), vec![1, 2, 3, 4]);
        assert_eq!(buf.readable_bytes(), 0);
    }

    #[test]
    fn buffer_get_slice_test() {
        let mut buf = Buffer::new(10);
        buf.append(&[1, 2, 3, 4]);
        assert_eq!(buf.get_slice(0), &[]);
        assert_eq!(buf.get_slice(2), &[1, 2]);
        assert_eq!(buf.get_slice(10), &[1, 2, 3, 4]);
    }
}
