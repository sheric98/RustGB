use crate::common::{RegBytes, ByteSize};
use crate::cpu::{Flag, Reg};
use crate::motherboard::Motherboard;

#[derive(Clone, Copy)]
pub enum RegExt {
    Reg(Reg),
    N,
    NN,
    NFlag(Flag),
    Flag(Flag),
    B(u8),
    H(u8),
}

impl RegExt {
    pub fn size(&self) -> ByteSize {
        match self {
            RegExt::Reg(Reg::A) => ByteSize::Single,
            RegExt::Reg(Reg::B) => ByteSize::Single,
            RegExt::Reg(Reg::C) => ByteSize::Single,
            RegExt::Reg(Reg::D) => ByteSize::Single,
            RegExt::Reg(Reg::E) => ByteSize::Single,
            RegExt::Reg(Reg::F) => ByteSize::Single,
            RegExt::Reg(Reg::H) => ByteSize::Single,
            RegExt::Reg(Reg::L) => ByteSize::Single,
            RegExt::NFlag(_) => ByteSize::Single,
            RegExt::Flag(_) => ByteSize::Single,
            RegExt::B(_) => ByteSize::Single,
            _ => ByteSize::Double
        }
    }
}

#[derive(Clone)]
pub struct CmdInp {
    re: RegExt,
    mem: bool,
    change: u16,
}

impl CmdInp {
    pub fn new(re: RegExt, mem: bool, change: u16) -> Self {
        Self {
            re,
            mem,
            change,
        }
    }

    pub fn size(&self) -> ByteSize {
        if self.mem {
            ByteSize::Single
        }
        else {
            self.re.size()
        }
    }
}

const CMD_INP_A: CmdInp = CmdInp {
    re: RegExt::Reg(Reg::A), 
    mem: false,
    change: 0,
};

const CMD_INP_PC: CmdInp = CmdInp {
    re: RegExt::Reg(Reg::PC),
    mem: false,
    change: 0,
};

fn get_reg_ext_val(
    mother: &Motherboard,
    arg: &CmdInp,
) -> RegBytes {
    let val;
    
    match &arg.re {
        RegExt::Reg(reg) => val = mother.cpu.read_reg(*reg),
        RegExt::N => val = mother.get_immediate_val(true),
        RegExt::NN => val = mother.get_immediate_val(false),
        _ => panic!("Get value of Flag or bit position")
    }

    if arg.mem {
        RegBytes::new_single(mother.get_mem_at(val.get_double() + arg.change))
    }
    else {
        val
    }
}

fn put_reg_ext_val(
    mother: &mut Motherboard,
    arg: &CmdInp,
    val: RegBytes,
) {
    if arg.mem {
        let new_arg = CmdInp::new(arg.re, false, 0);
        let loc = get_reg_ext_val(mother, &new_arg);
        mother.put_mem_at(loc.get_double() + arg.change, val.get_single())
    }
    else {
        match arg.re {
            RegExt::Reg(reg) => mother.cpu.write_reg(reg, val),
            _ => panic!("putting val into non reg"),
        }
    }
}

fn get_reg_ext_flag_val(
    mother: &mut Motherboard,
    arg: &CmdInp,
) -> bool {
    match &arg.re {
        RegExt::Flag(flag) => mother.cpu.check_flag(*flag),
        RegExt::NFlag(flag) => !mother.cpu.check_flag(*flag),
        _ => panic!("Getting flag value from nonflag"),
    }
}

fn get_reg_ext_byte_val(
    mother: &mut Motherboard,
    arg: &CmdInp,
) -> u8 {
    match &arg.re {
        RegExt::B(b) => *b,
        RegExt::H(h) => *h,
        _ => panic!("Getting byte value from non-byte"),
    }
}

fn get_flag_val(
    mother: &mut Motherboard,
    flag: Flag,
) -> u8 {
    mother.cpu.check_flag(flag) as u8
}

// booleans to indicate references to memory
pub fn ld(
    mother: &mut Motherboard,
    dst: CmdInp,
    src: CmdInp,
) {
    let src_val = get_reg_ext_val(mother, &src);
    put_reg_ext_val(mother, &dst, src_val);
}

// note we only decrement HL in ldd
fn change_hl(mother: &mut Motherboard, reg_ext: RegExt, inc: bool) {
    match reg_ext {
        RegExt::Reg(Reg::HL) => {
            let mut val: u16 = mother.cpu.read_reg(Reg::HL).get_double();
            if inc {
                val += 1;
            }
            else {
                val -= 1;
            }
            mother.cpu.write_reg(Reg::HL, RegBytes::new_double(val));
        }
        _ => panic!("unexpected reg_ext in ldd"),
    }
} 

fn ld_change(
    mother: &mut Motherboard,
    dst: CmdInp,
    src: CmdInp,
    inc: bool,
) {
    ld(mother, dst.clone(), src.clone());
    if dst.mem {
        change_hl(mother, dst.re, inc);
    }
    else if src.mem {
        change_hl(mother, src.re, inc);
    }
    else {
        panic!("non ref in ldd");
    }
}

pub fn ldd(
    mother: &mut Motherboard,
    dst: CmdInp,
    src: CmdInp,
) {
    ld_change(mother, dst, src, false);
}

pub fn ldi(
    mother: &mut Motherboard,
    dst: CmdInp,
    src: CmdInp,
) {
    ld_change(mother, dst, src, true);
}

pub fn ldhl(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let v = get_reg_ext_val(mother, &arg2).get_single() as u16;
    let new_arg1 = CmdInp::new(RegExt::Reg(Reg::HL), false, 0);
    let new_arg2 = CmdInp::new(arg1.re, arg1.mem, arg1.change + v);
    ld(mother, new_arg1, new_arg2);
}

pub fn push(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_double();
    mother.push(val);
}

pub fn pop(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = RegBytes::new_double(mother.pop());
    put_reg_ext_val(mother, &arg, val);
}

pub fn add(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let size1 = arg1.size();
    let v1 = get_reg_ext_val(mother, &arg1);
    let v2 = get_reg_ext_val(mother, &arg2);
    let mut val;
    match size1 {
        ByteSize::Single => {
            val = RegBytes::new_single(v1.get_single() + v2.get_single());
        },
        ByteSize::Double => {
            let size2 = arg2.size();
            let val1 = v1.get_double();
            let mut val2;
            match size2 {
                ByteSize::Single => {
                    val2 = v2.get_single() as u16;
                },
                ByteSize::Double => {
                    val2 = v2.get_double();
                }
            }
            val = RegBytes::new_double(val1 + val2);
        },
    }
    put_reg_ext_val(mother, &arg1, val);
}

pub fn adc(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let v1 = get_reg_ext_val(mother, &arg1).get_single();
    let v2 = get_reg_ext_val(mother, &arg2).get_single();
    let carry = get_flag_val(mother, Flag::C);
    let new_val = RegBytes::new_single(v1 + v2 + carry);
    put_reg_ext_val(mother, &arg1, new_val);
}

pub fn sub(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let a_val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_val = RegBytes::new_single(a_val - val);
    put_reg_ext_val(mother, &CMD_INP_A, new_val);
}

pub fn sbc(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let v1 = get_reg_ext_val(mother, &arg1).get_single();
    let v2 = get_reg_ext_val(mother, &arg2).get_single();
    let carry = get_flag_val(mother, Flag::C);
    let new_val = RegBytes::new_single(v1 - v2 - carry);
    put_reg_ext_val(mother, &arg1, new_val);
}

pub fn and(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let a_val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let v = get_reg_ext_val(mother, &arg).get_single();
    let new_v = RegBytes::new_single(a_val & v);
    put_reg_ext_val(mother, &CMD_INP_A, new_v);
}

pub fn or(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let a_val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let v = get_reg_ext_val(mother, &arg).get_single();
    let new_v = RegBytes::new_single(a_val | v);
    put_reg_ext_val(mother, &CMD_INP_A, new_v);
}

pub fn xor(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let a_val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let v = get_reg_ext_val(mother, &arg).get_single();
    let new_v = RegBytes::new_single(a_val ^ v);
    put_reg_ext_val(mother, &CMD_INP_A, new_v);
}

pub fn cp(
    mother: &Motherboard,
    arg: CmdInp,
) {
    let a_val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let v = get_reg_ext_val(mother, &arg).get_single();
    let out = a_val - v;
}

pub fn inc(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    match arg.size() {
        ByteSize::Single => {
            let v = get_reg_ext_val(mother, &arg).get_single();
            let new_v = RegBytes::new_single(v + 1);
            put_reg_ext_val(mother, &arg, new_v);
        }
        ByteSize::Double => {
            let v = get_reg_ext_val(mother, &arg).get_double();
            let new_v = RegBytes::new_double(v + 1);
            put_reg_ext_val(mother, &arg, new_v);
        }
    }
}

pub fn dec(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    match arg.size() {
        ByteSize::Single => {
            let v = get_reg_ext_val(mother, &arg).get_single();
            let new_v = RegBytes::new_single(v - 1);
            put_reg_ext_val(mother, &arg, new_v);
        }
        ByteSize::Double => {
            let v = get_reg_ext_val(mother, &arg).get_double();
            let new_v = RegBytes::new_double(v - 1);
            put_reg_ext_val(mother, &arg, new_v);
        }
    }
}

pub fn swap(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_v = (val >> 4) + (val << 4);
    let bytes = RegBytes::new_single(new_v);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn daa(
    mother: &mut Motherboard,
) {
    let val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let new_val: u8;
    let mut corr: u8 = 0;
    if mother.cpu.check_flag(Flag::H) {
        corr += 0x6;
    }
    if mother.cpu.check_flag(Flag::C) {
        corr += 0x60;
    }
    if mother.cpu.check_flag(Flag::N) {
        new_val = val - corr;
    } else {
        if (val & 0xf) > 0x9 {
            corr |= 0x6;
        }
        if val > 0x99 {
            corr |= 0x60;
        }
        new_val = val + corr;
    }
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &CMD_INP_A, bytes);
}

pub fn cpl(
    mother: &mut Motherboard,
) {
    let val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let new_v = !val;
    let bytes = RegBytes::new_single(new_v);
    put_reg_ext_val(mother, &CMD_INP_A, bytes);
}

pub fn ccf(
    mother: &Motherboard,
) {
    ()
}

pub fn scf(
    mother: &Motherboard,
) {
    ()
}

pub fn nop(
    mother: &Motherboard,
) {
    ()
}

pub fn halt(
    mother: &Motherboard,
) {
    ()
}

pub fn stop(
    mother: &Motherboard,
) {
    ()
}

pub fn di(
    mother: &Motherboard,
) {
    ()
}

pub fn ei(
    mother: &Motherboard,
) {
    ()
}

pub fn rlca(
    mother: &mut Motherboard,
) {
    let val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let new_val = (val << 1) + (val >> 7);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &CMD_INP_A, bytes);
}

pub fn rla(
    mother: &mut Motherboard,
) {
    let val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let flag_val = get_flag_val(mother, Flag::C);
    let new_val = flag_val + val;
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &CMD_INP_A, bytes);
}

pub fn rrca(
    mother: &mut Motherboard,
) {
    let val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let new_val = (val >> 1) + (val << 7);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &CMD_INP_A, bytes);
}

pub fn rra(
    mother: &mut Motherboard,
) {
    let val = get_reg_ext_val(mother, &CMD_INP_A).get_single();
    let flag = get_flag_val(mother, Flag::C);
    let new_val = (val >> 1) + (flag << 7);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &CMD_INP_A, bytes);
}

pub fn rlc(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_val = (val << 1) + (val >> 7);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn rl(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let flag = get_flag_val(mother, Flag::C);
    let new_val = (val << 1) + flag;
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn rrc(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_val = (val >> 1) + (val << 7);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn rr(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let flag = get_flag_val(mother, Flag::C);
    let new_val = (val >> 1) + (flag << 7);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn sla(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_val = val << 1;
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn sra(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_val = (val >> 1) | (val & 0x80);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn srl(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_val = val >> 1;
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg, bytes);
}

pub fn bit(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let pos = get_reg_ext_byte_val(mother, &arg1);
    let val = get_reg_ext_val(mother, &arg2).get_single();
    let test = (val & (1 << pos)) == 0;
}

pub fn set(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let pos = get_reg_ext_byte_val(mother, &arg1);
    let val = get_reg_ext_val(mother, &arg2).get_single();
    let new_val = val | (1 << pos);
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg2, bytes);
}

pub fn res(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let pos = get_reg_ext_byte_val(mother, &arg1);
    let val = get_reg_ext_val(mother, &arg2).get_single();
    let new_val = val & (!(1 << pos));
    let bytes = RegBytes::new_single(new_val);
    put_reg_ext_val(mother, &arg2, bytes);
}

pub fn jp(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let val = get_reg_ext_val(mother, &arg);
    put_reg_ext_val(mother, &CMD_INP_PC, val);
}

pub fn jp_flag(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let test = get_reg_ext_flag_val(mother, &arg1);
    if test {
        jp(mother, arg2);
    }
}

pub fn jr(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let curr = get_reg_ext_val(mother, &CMD_INP_PC).get_double();
    let val = get_reg_ext_val(mother, &arg).get_single();
    let new_val = curr + (val as u16);
    let bytes = RegBytes::new_double(new_val);
    put_reg_ext_val(mother, &CMD_INP_PC, bytes);
}

pub fn jr_flag(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let test = get_reg_ext_flag_val(mother, &arg1);
    if test {
        jr(mother, arg2);
    }
}

pub fn call(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let cur = get_reg_ext_val(mother, &CMD_INP_PC).get_double();
    mother.push(cur + 3);
    let val = get_reg_ext_val(mother, &arg);
    put_reg_ext_val(mother, &CMD_INP_PC, val);
}

pub fn call_flag(
    mother: &mut Motherboard,
    arg1: CmdInp,
    arg2: CmdInp,
) {
    let test = get_reg_ext_flag_val(mother, &arg1);
    if test {
        call(mother, arg2);
    }
}

pub fn rst(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let cur = get_reg_ext_val(mother, &CMD_INP_PC).get_double();
    mother.push(cur);
    let new_addr = get_reg_ext_byte_val(mother, &arg) as u16;
    let bytes = RegBytes::new_double(new_addr);
    put_reg_ext_val(mother, &CMD_INP_PC, bytes);
}

pub fn ret(
    mother: &mut Motherboard,
) {
    let addr = mother.pop();
    let bytes = RegBytes::new_double(addr);
    put_reg_ext_val(mother, &CMD_INP_PC, bytes);
}

pub fn ret_flag(
    mother: &mut Motherboard,
    arg: CmdInp,
) {
    let test = get_reg_ext_flag_val(mother, &arg);
    if test {
        ret(mother);
    }
}

pub fn reti(
    mother: &Motherboard,
) {
    ()
}
