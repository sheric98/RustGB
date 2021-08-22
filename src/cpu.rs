use crate::common::RegBytes;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use maplit::hashmap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Reg {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

enum RegOrder {
    First,
    Second,
    Both,
}

struct RegPair {
    pair: [u8; 2],
}

impl RegPair {
    fn new() -> Self {
        Self {
            pair: [0, 0],
        }
    }

    fn write_8(&mut self, first: bool, byte: RegBytes) {
        if first {
            self.pair[0] = byte.get_single();
        }
        else {
            self.pair[1] = byte.get_single();
        }
    }

    fn read_8(&self, first: bool) -> RegBytes {
        if first {
            RegBytes::new_single(self.pair[0])
        }
        else {
            RegBytes::new_single(self.pair[1])
        }
    }

    fn write_16(&mut self, bytes: RegBytes) {
        self.pair = bytes.get_double().to_le_bytes();
    }

    fn read_16(&self) -> RegBytes {
        RegBytes::new_double(u16::from_le_bytes(self.pair))
    }
}

#[derive(Clone, Copy)]
pub enum Flag {
    Z = 1 << 7,
    N = 1 << 6,
    H = 1 << 5,
    C = 1 << 4,
}

fn set_flag(flag: Flag, byte: &mut RegBytes) {
    unsafe {
        byte.single |= flag as u8;
    }
}

fn unset_flag(flag: Flag, byte: &mut RegBytes) {
    unsafe {
        byte.single &= !(flag as u8);
    }
}

fn check_flag(flag: Flag, byte: &RegBytes) -> bool {
    unsafe {
        byte.single & (flag as u8) != 0
    }
}

pub struct CPU {
    pub sp: u16,
    pub pc: u16,

    reg_map: HashMap<Reg, (Rc<RefCell<RegPair>>, RegOrder)>,
}

impl CPU {
    pub fn new() -> Self {
        let af = Rc::new(RefCell::new(RegPair::new()));
        let bc = Rc::new(RefCell::new(RegPair::new()));
        let de = Rc::new(RefCell::new(RegPair::new()));
        let hl = Rc::new(RefCell::new(RegPair::new()));

        Self {
            reg_map: hashmap!{
                Reg::A => (af.clone(), RegOrder::First),
                Reg::B => (bc.clone(), RegOrder::First),
                Reg::C => (bc.clone(), RegOrder::Second),
                Reg::D => (de.clone(), RegOrder::First),
                Reg::E => (de.clone(), RegOrder::Second),
                Reg::F => (af.clone(), RegOrder::Second),
                Reg::H => (hl.clone(), RegOrder::First),
                Reg::L => (hl.clone(), RegOrder::Second),
                Reg::AF => (af.clone(), RegOrder::Both),
                Reg::BC => (bc.clone(), RegOrder::Both),
                Reg::DE => (de.clone(), RegOrder::Both),
                Reg::HL => (hl.clone(), RegOrder::Both),
            },

            sp: 0,
            pc: 0,
        }
    }

    pub fn read_reg(&self, reg: Reg) -> RegBytes {
        match reg {
            Reg::SP => RegBytes::new_double(self.sp),
            Reg::PC => RegBytes::new_double(self.pc),
            _ => {
                let (pair_ref, order) = self.reg_map.get(&reg).unwrap();
                let pair = pair_ref.borrow();
                
                match order {
                    RegOrder::First => pair.read_8(true),
                    RegOrder::Second => pair.read_8(false),
                    RegOrder::Both => pair.read_16(),
                }
            }
        }
    }

    pub fn write_reg(&mut self, reg: Reg, bytes: RegBytes) {
        match reg {
            Reg::SP => self.sp = bytes.get_double(),
            Reg::PC => self.pc = bytes.get_double(),
            _ => {
                let (pair_ref, order) = self.reg_map.get(&reg).unwrap();
                let mut pair = pair_ref.borrow_mut();
        
                match order {
                    RegOrder::First => pair.write_8(true, bytes),
                    RegOrder::Second => pair.write_8(false, bytes),
                    RegOrder::Both => pair.write_16(bytes),
                }
            }
        }
    }

    fn set_flag(&mut self, flag: Flag) {
        let mut byte = self.read_reg(Reg::F);
        set_flag(flag, &mut byte);
        self.write_reg(Reg::F, byte);
    }

    fn unset_flag(&mut self, flag: Flag) {
        let mut byte = self.read_reg(Reg::F);
        unset_flag(flag, &mut byte);
        self.write_reg(Reg::F, byte);
    }

    pub fn check_flag(&mut self, flag: Flag) -> bool {
        let byte = self.read_reg(Reg::F);
        check_flag(flag, &byte)
    }
}
