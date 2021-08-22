pub union RegBytes {
    pub single: u8,
    pub double: u16,
}

impl RegBytes {
    pub fn new_single(single: u8) -> Self {
        Self {
            single,
        }
    }

    pub fn new_double(double: u16) -> Self {
        Self {
            double,
        }
    }

    pub fn get_single(&self) -> u8 {
        unsafe {
            self.single
        }
    }

    pub fn get_double(&self) -> u16 {
        unsafe {
            self.double
        }
    }
}

pub enum ByteSize {
    Single,
    Double,
}
