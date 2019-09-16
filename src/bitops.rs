pub const ARCHITECTURE_SIZE: usize = std::mem::size_of::<usize>()*8;

pub fn lsb_bitmask(bit_count: usize) -> usize {
    match 1usize.checked_shl(bit_count as u32) {
        Some(shifted) => shifted.wrapping_sub(1usize),
        None => -1isize as usize
    }
}

pub struct BitwiseRead<'a> {
    src: &'a [usize],
    length: isize,
    shift: usize,
    index: usize
}

impl BitwiseRead<'_> {
    pub fn new<'a>(src: &'a [usize], length: usize, offset: usize) -> BitwiseRead<'a> {
        let result: BitwiseRead<'a> = BitwiseRead {
            src: src,
            length: length as isize,
            shift: offset%ARCHITECTURE_SIZE,
            index: offset/ARCHITECTURE_SIZE
        };
        result
    }
}

impl Iterator for BitwiseRead<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length <= 0 {
            return None;
        }
        let mut dst = self.src[self.index]>>self.shift;
        if (self.length as usize) > ARCHITECTURE_SIZE-self.shift && self.shift > 0 {
            dst |= self.src[self.index+1]<<(ARCHITECTURE_SIZE-self.shift);
        }
        if (self.length as usize) < ARCHITECTURE_SIZE {
            dst &= lsb_bitmask(self.length as usize);
        }
        self.length -= ARCHITECTURE_SIZE as isize;
        self.index += 1;
        Some(dst)
    }
}

pub struct BitwiseWrite<'a> {
    dst: &'a mut [usize],
    length: isize,
    shift: usize,
    index: usize
}

impl BitwiseWrite<'_> {
    pub fn new<'a>(dst: &'a mut [usize], length: usize, offset: usize) -> BitwiseWrite<'a> {
        let result: BitwiseWrite<'a> = BitwiseWrite {
            dst: dst,
            length: length as isize,
            shift: offset%ARCHITECTURE_SIZE,
            index: offset/ARCHITECTURE_SIZE
        };
        result
    }

    pub fn more(&self) -> bool {
        self.length > 0
    }

    pub fn next(&mut self, mut src: usize) {
        let mask = lsb_bitmask(self.length as usize);
        src &= mask;
        self.dst[self.index] &= !(mask<<self.shift);
        self.dst[self.index] |= src<<self.shift;
        if (self.length as usize) > ARCHITECTURE_SIZE-self.shift && self.shift > 0 {
            self.dst[self.index+1] &= !(mask>>(ARCHITECTURE_SIZE-self.shift));
            self.dst[self.index+1] |= src>>(ARCHITECTURE_SIZE-self.shift);
        }
        self.length -= ARCHITECTURE_SIZE as isize;
        self.index += 1;
    }
}

pub fn bitwise_copy(dst: &mut[usize], src: &[usize], dst_offset: usize, src_offset: usize, length: usize) {
    let bitwise_read = BitwiseRead::new(src, length, src_offset);
    let mut bitwise_write = BitwiseWrite::new(dst, length, dst_offset);
    for element in bitwise_read {
        bitwise_write.next(element);
    }
}
