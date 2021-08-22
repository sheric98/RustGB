use crate::common::RegBytes;
use crate::cpu::CPU;

pub struct Motherboard {
    pub cpu: CPU,
}

impl Motherboard {
    pub fn new() -> Self {
        Self {
            cpu: CPU::new(),
        }
    }

    pub fn get_mem_at(&self, addr: u16) -> u8 {
        0
    }

    pub fn put_mem_at(&self, addr: u16, val: u8) {
        ()
    }

    // true for byte, false for two bytes
    pub fn get_immediate_val(&self, single: bool) -> RegBytes {
        if single {
            let byte = self.get_mem_at(self.cpu.pc + 1);
            RegBytes::new_single(byte)
        }
        else {
            let byte1 = self.get_mem_at(self.cpu.pc + 1);
            let byte2 = self.get_mem_at(self.cpu.pc + 2);
            let bytes = u16::from_le_bytes([byte1, byte2]);
            RegBytes::new_double(bytes)
        }
    }

    pub fn push(&mut self, val: u16) {
        let bytes = val.to_be_bytes();
        self.put_mem_at(self.cpu.sp - 2, bytes[0]);
        self.put_mem_at(self.cpu.sp - 1, bytes[1]);
        self.cpu.sp -= 2;
    }

    pub fn pop(&mut self) -> u16 {
        let byte1 = self.get_mem_at(self.cpu.sp);
        let byte2 = self.get_mem_at(self.cpu.sp + 1);
        let ret = u16::from_be_bytes([byte2, byte1]);
        self.cpu.sp += 2;
        ret
    }
}