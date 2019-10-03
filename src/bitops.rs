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

pub fn bitwise_copy_nonoverlapping(dst: &mut[usize], src: &[usize], dst_offset: usize, src_offset: usize, length: usize) {
    if length == 0 {
        return;
    }
    if dst_offset%ARCHITECTURE_SIZE == 0 && src_offset%ARCHITECTURE_SIZE == 0 {
        let mut last_index = (length+ARCHITECTURE_SIZE-1)/ARCHITECTURE_SIZE;
        if length%ARCHITECTURE_SIZE > 0 {
            last_index -= 1;
            let last_in_dst = &mut dst[dst_offset/ARCHITECTURE_SIZE+last_index];
            let mask = lsb_bitmask(length%ARCHITECTURE_SIZE);
            *last_in_dst = (*last_in_dst&!mask)|(src[src_offset/ARCHITECTURE_SIZE+last_index]&mask);
        }
        for index in 0..last_index {
            dst[dst_offset/ARCHITECTURE_SIZE+index] = src[src_offset/ARCHITECTURE_SIZE+index];
        }
    } else if dst_offset%8 == 0 && src_offset%8 == 0 {
        let dst_bytes: *mut u8 = unsafe { (dst.as_mut_ptr() as *mut u8).offset((dst_offset/8) as isize) };
        let src_bytes: *const u8 = unsafe { (src.as_ptr() as *const u8).offset((src_offset/8) as isize) };
        let mut last_index = (length+7)/8;
        if length%8 > 0 {
            last_index -= 1;
            let last_in_dst = unsafe { dst_bytes.offset(last_index as isize) };
            let mask = lsb_bitmask(length%8) as u8;
            unsafe { *last_in_dst = ((*last_in_dst)&(!mask))|((*src_bytes.offset(last_index as isize))&mask) };
        }
        // for index in 0..last_index {
        //     unsafe { *dst_bytes.offset(index as isize) = *src_bytes.offset(index as isize); }
        // }
        unsafe { std::ptr::copy_nonoverlapping(src_bytes, dst_bytes, last_index); }
    } else {
        let bitwise_read = BitwiseRead::new(src, length, src_offset);
        let mut bitwise_write = BitwiseWrite::new(dst, length, dst_offset);
        for element in bitwise_read {
            bitwise_write.next(element);
        }
    }
}
