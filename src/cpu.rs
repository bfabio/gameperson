use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::memory::Memory;

#[derive(Debug)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug)]
pub enum Register16 {
    BC,
    DE,
    HL,
}

#[derive(Debug)]
pub enum Location {
    Register(Register8),
    DoubleRegister(Register16),
    Address(u16),
}

impl Location {
    fn load8(&self, regs: &Registers, address_space: &Memory) -> u8 {
        match self {
            Self::Register(reg) => match reg {
                Register8::A => regs.a,
                Register8::B => regs.b,
                Register8::C => regs.c,
                Register8::D => regs.d,
                Register8::E => regs.e,
                Register8::H => regs.h,
                Register8::L => regs.l,
            },
            Self::DoubleRegister(_) => {
                panic!("load8() called on a 16bit register");
            }
            Self::Address(address) => address_space.load(*address as usize),
        }
    }

    #[allow(unused)]
    fn load16(&self, regs: &Registers, address_space: &Memory) -> u16 {
        match self {
            Self::Register(_) => {
                panic!("load16() called on a 8bit register");
            }
            Self::DoubleRegister(reg) => match reg {
                Register16::BC => regs.bc(),
                Register16::HL => regs.hl(),
                Register16::DE => regs.de(),
            },
            Self::Address(address) => {
                let (high, low) = (*address as usize, (*address + 1) as usize);
                u16::from(address_space.load(high)) | u16::from(address_space.load(low) << 8)
            }
        }
    }
    fn store8(&self, regs: &mut Registers, address_space: &mut Memory, value: u8) {
        match self {
            Self::Register(reg) => match reg {
                Register8::A => regs.a = value,
                Register8::B => regs.b = value,
                Register8::C => regs.c = value,
                Register8::D => regs.d = value,
                Register8::E => regs.e = value,
                Register8::H => regs.h = value,
                Register8::L => regs.l = value,
            },
            Self::DoubleRegister(_) => {
                panic!("store8() called on a 16bit register");
            }
            Self::Address(address) => {
                address_space.write(*address as usize, value);
            }
        }
    }

    #[allow(unused)]
    fn store16(&self, regs: &mut Registers, address_space: &mut Memory, value: u16) {
        match self {
            Self::Register(_) => {
                panic!("store16() called on a 8bit register");
            }
            Self::DoubleRegister(reg) => match reg {
                Register16::BC => regs.write_bc(value),
                Register16::HL => regs.write_hl(value),
                Register16::DE => regs.write_de(value),
            },
            Self::Address(_) => {
                unimplemented!();
            }
        };
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Register(reg8) => match reg8 {
                Register8::A => write!(f, "A"),
                Register8::B => write!(f, "B"),
                Register8::C => write!(f, "C"),
                Register8::D => write!(f, "D"),
                Register8::E => write!(f, "E"),
                Register8::H => write!(f, "H"),
                Register8::L => write!(f, "L"),
            },
            Self::DoubleRegister(reg16) => match reg16 {
                Register16::BC => write!(f, "BC"),
                Register16::HL => write!(f, "HL"),
                Register16::DE => write!(f, "DE"),
            },
            Self::Address(addr) => write!(f, "({:#06x})", addr),
        }
    }
}

pub struct Cpu {
    regs: Registers,

    // Stack pointer
    sp: u16,

    // Program counter
    pub pc: u16,

    memory: Rc<RefCell<Memory>>,

    pub interrupts_enabled: bool,
    current_op: String,
}

#[derive(Default)]
struct Registers {
    flags: u8,

    // Accumulator
    a: u8,

    b: u8,
    c: u8,

    d: u8,
    e: u8,

    h: u8,
    l: u8,
}

impl Registers {
    pub fn hl(&self) -> u16 {
        (u16::from(self.h) << 8) | u16::from(self.l)
    }

    pub fn bc(&self) -> u16 {
        (u16::from(self.b) << 8) | u16::from(self.c)
    }

    pub fn de(&self) -> u16 {
        (u16::from(self.d) << 8) | u16::from(self.e)
    }

    pub fn write_bc(&mut self, value: u16) {
        self.b = ((value & 0xff00) >> 8) as u8;
        self.c = value as u8;
    }

    pub fn write_de(&mut self, value: u16) {
        self.d = ((value & 0xff00) >> 8) as u8;
        self.e = value as u8;
    }

    pub fn write_hl(&mut self, value: u16) {
        self.h = ((value & 0xff00) >> 8) as u8;
        self.l = value as u8;
    }

    pub fn write_af(&mut self, value: u16) {
        self.a = ((value & 0xff00) >> 8) as u8;

        // Flags are only in the most significant nibble
        self.flags = value as u8 & 0xf0;
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "a: {:#04x}, b: {:#04x}, c: {:#04x}, d: {:#04x}, e: {:#04x}, \
                   h: {:#04x}, l: {:#04x} flags: {:08b} ({:#02x})",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.flags, self.flags
        )
    }
}

const ZERO_FLAG: u8 = 1 << 7;
const SUBTRACT_FLAG: u8 = 1 << 6;
const HALF_CARRY_FLAG: u8 = 1 << 5;
const CARRY_FLAG: u8 = 1 << 4;

impl Cpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Cpu {
        Self {
            regs: Registers::default(),
            sp: 0,
            pc: 0,
            memory,
            interrupts_enabled: false,
            current_op: String::new(),
        }
    }

    pub fn dump_mem(&self) {
        println!("{}", self.memory.borrow());
    }

    pub fn load_immediate8(&self) -> u8 {
        self.memory.borrow().load(self.pc as usize)
    }

    pub fn index(&self, index: u8) -> Location {
        match index {
            0 => Location::Register(Register8::B),
            1 => Location::Register(Register8::C),
            2 => Location::Register(Register8::D),
            3 => Location::Register(Register8::E),
            4 => Location::Register(Register8::H),
            5 => Location::Register(Register8::L),
            6 => Location::Address(self.regs.hl()),
            7 => Location::Register(Register8::A),
            _ => panic!("Invalid index {}", index),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn decode(&mut self) -> u8 {
        let mut memory = self.memory.borrow_mut();

        // REMOVE ME
        // Tetris vblank routine
        /*
        if self.pc == 0x17e {
            panic!();
        }*/

        let opcode = memory.load(self.pc as usize);
        let mut cycles = 1;

        match opcode {
            0x00 => (),
            0x01 | 0x11 | 0x21 | 0x31 => {
                // LD r16,n
                //
                // Store 16 bit immediate value n into 16 bit register
                // (BC, DE, HL, SP).

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let immediate: u16 = u16::from(high) << 8 | u16::from(low);

                let reg_name = match (opcode & 0xF0) >> 4 {
                    0 => {
                        self.regs.write_bc(immediate);
                        "BC"
                    }
                    1 => {
                        self.regs.write_de(immediate);
                        "DE"
                    }
                    2 => {
                        self.regs.write_hl(immediate);
                        "HL"
                    }
                    3 => {
                        self.sp = immediate;
                        "SP"
                    }
                    _ => unreachable!(),
                };

                self.current_op = format!(
                    "{:#04x} {:#04x} LD {},{:#x}",
                    low, high, reg_name, immediate
                );
                cycles = 12;
            }
            0x05 | 0x15 | 0x25 | 0x35 | 0x0d | 0x1d | 0x2d | 0x3d => {
                // DEC r8
                //
                // Decrement 8 bit register r8.
                //
                // flags:
                // Z: Set if result is zero, unset otherwise
                // N: 1
                // H: Set if no borrow from bit 4, unset otherwise
                // C: no change.

                let reg_index = (opcode & 0b0011_1000) >> 3;

                let location = self.index(reg_index);
                let value = location.load8(&self.regs, &memory).wrapping_sub(1);

                location.store8(&mut self.regs, &mut memory, value);

                self.regs.set_flag(ZERO_FLAG, value == 0);
                self.regs.set_flag(SUBTRACT_FLAG, true);
                self.regs.set_flag(HALF_CARRY_FLAG, value & 0x0f == 0x0f);

                cycles = if opcode == 0x35 { 12 } else { 4 };
            }
            0x07 => {
                // RLCA
                //
                // Rotate A left. Old bit 7 to Carry flag.
                //
                //  before:
                //
                //  +---+  +-------------------------------+
                //  |   |  | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
                //  +---+  +-------------------------------+
                //      C                                  A
                //
                //  after
                //
                //  +---+  +---------------------------------+
                //  | 7 |  | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 0x0 |
                //  +---+  +---------------------------------+
                //      C                                  A
                //
                // flags:
                // Z: 0 (GBCPUman.pdf v1.01 is wrong on this one)
                // N: 0
                // H: 0
                // C: Contains old bit 7 data.

                self.regs
                    .set_flag(CARRY_FLAG, self.regs.a & 0b1000_0000 != 0);

                self.regs.a <<= 1;

                self.regs.set_flag(ZERO_FLAG, false);
                self.regs.set_flag(SUBTRACT_FLAG, false);
                self.regs.set_flag(HALF_CARRY_FLAG, false);

                self.current_op = format!("{:10} RLCA", " ");
                cycles = 4;
            }
            0x08 => {
                // LD (a16), SP
                //
                // Put SP value into memory address a16.

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let addr: u16 = u16::from_be_bytes([high, low]);

                memory.write(addr as usize, (self.sp & 0xff) as u8);
                memory.write(addr as usize + 1, ((self.sp & 0xff00) >> 8) as u8);

                self.current_op = format!("{:9} LD ({:#06x}),SP", " ", addr);

                cycles = 20;
            }
            0x17 => {
                // RLA
                // (Note this is different from RL A)
                //
                // 9-bit rotation to the left using the carry flag.
                // A's bits are shifted left, the carry value is put
                // into 0th bit of A and the leaving 7th bit is put
                // into the carry.
                //
                //  before:
                //
                //  +---+  +-------------------------------+
                //  | 8 |  | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
                //  +---+  +-------------------------------+
                //      C                                  A
                //
                //  after
                //
                //  +---+  +-------------------------------+
                //  | 7 |  | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 8 |
                //  +---+  +-------------------------------+
                //      C                                  A
                //
                // flags:
                // Z: 0 (GBCPUman.pdf v1.01 is wrong on this one)
                // N: 0
                // H: 0
                // C: Contains old bit 7 data.

                let carry = (self.regs.flags & CARRY_FLAG) >> 4;

                self.regs
                    .set_flag(CARRY_FLAG, self.regs.a & 0b1000_0000 != 0);

                self.regs.a = self.regs.a << 1 | carry;

                self.regs.set_flag(ZERO_FLAG, false);
                self.regs.set_flag(SUBTRACT_FLAG, false);
                self.regs.set_flag(HALF_CARRY_FLAG, false);

                self.current_op = format!("{:10} RLA", " ");
                cycles = 4;
            }
            0x1f => {
                // RRA
                // (Note this not the same instruction as RL A)

                let carry = (self.regs.flags & CARRY_FLAG) >> 4;

                self.regs
                    .set_flag(CARRY_FLAG, self.regs.a & 0x1 != 0);

                self.regs.a = self.regs.a >> 1 | carry << 7;

                self.regs.set_flag(ZERO_FLAG, false);
                self.regs.set_flag(SUBTRACT_FLAG, false);
                self.regs.set_flag(HALF_CARRY_FLAG, false);

                cycles = 4;
            }
            0x09 | 0x19 | 0x29 | 0x39 => {
                // ADD HL,n
                //
                // Add n to HL.
                //
                // flags:
                // Z: No change
                // N: 0
                // H: Set if carry from bit 11
                // C: Set if carry from bit 15.

                let operand = match opcode {
                    0x09 => self.regs.bc(),
                    0x19 => self.regs.de(),
                    0x29 => self.regs.hl(),
                    0x39 => self.sp,
                    _ => unreachable!(),
                };

                let half_carry = (self.regs.hl() & 0xfff) + (operand & 0xfff) > 0xfff;
                self.regs.set_flag(HALF_CARRY_FLAG, half_carry);

                let (result, carry) = self.regs.hl().overflowing_add(operand);
                self.regs.write_hl(result);

                self.regs.set_flag(SUBTRACT_FLAG, false);
                self.regs.set_flag(CARRY_FLAG, carry);

                cycles = 8;
            }
            0x0a | 0x1a | 0x2a | 0x3a => {
                // LD A,(r16)
                //
                // Store 16 bit value from memory location at register r16 to A.
                // (BC, DE, HL).

                let (value, reg_name) = match (opcode & 0xF0) >> 4 {
                    0 => (memory.load(self.regs.bc() as usize), "BC"),
                    1 => {
                        self.current_op = format!("DE: {:#06x}", self.regs.de());
                        (memory.load(self.regs.de() as usize), "DE")
                    }
                    2 => {
                        let hl = self.regs.hl();
                        self.regs.write_hl(hl + 1);

                        (memory.load(hl as usize), "HL+")
                    }
                    3 => {
                        let hl = self.regs.hl();
                        self.regs.write_hl(hl - 1);

                        (memory.load(hl as usize), "HL-")
                    }
                    _ => unreachable!(),
                };

                self.regs.a = value;

                self.current_op = format!("{:10} LD A,({})", " ", reg_name);
                cycles = 8;
            }
            0x06 | 0x16 | 0x26 | 0x36 | 0x0e | 0x1e | 0x2e | 0x3e => {
                // LD r8,n
                //
                // Store 8 bit immediate value n into 8 bit register

                self.pc += 1;
                let immediate = memory.load(self.pc as usize);

                let reg_index = (opcode & 0b0011_1000) >> 3;

                let location = self.index(reg_index);
                location.store8(&mut self.regs, &mut memory, immediate);

                self.current_op = format!("{1:#04x}      LD {0},{1:#x}", location, immediate);
                cycles = if opcode == 0x36 { 12 } else { 8 };
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                // 16bit INC
                //
                // No flags affected.

                let reg_name = match (opcode & 0xF0) >> 4 {
                    0x0 => {
                        self.regs.write_bc(self.regs.bc().wrapping_add(1));

                        "BC"
                    }
                    0x1 => {
                        self.regs.write_de(self.regs.de().wrapping_add(1));

                        "DE"
                    }
                    0x2 => {
                        self.regs.write_hl(self.regs.hl().wrapping_add(1));

                        "HL"
                    }
                    0x3 => {
                        self.sp = self.sp.wrapping_add(1);

                        "SP"
                    }
                    _ => unreachable!(),
                };

                self.current_op = format!("{:9} INC {}", " ", reg_name);
                cycles = 8;
            }
            0x18 => {
                // JR n
                //
                // Jump to the current address + n (signed).

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                self.current_op = format!("{0:#04x}      JR {0:#04x} {0}", immediate);

                // Relative jump is calculated starting from the next instruction
                self.pc += 1;

                // XXX
                self.pc = if immediate >= 0 {
                    self.pc.saturating_add(immediate as u16)
                } else {
                    let immediate = immediate.abs();
                    self.pc.saturating_sub(immediate as u16)
                };
                cycles = 12;

                return cycles;
            }
            0x28 => {
                // JR Z,n
                //
                // Jump to the current address + n (signed) if Z is 1.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                self.current_op = format!("{0:#04x}      JR Z,{0:#04x} {0}", immediate);

                if (self.regs.flags & ZERO_FLAG) != 0 {
                    // Relative jump is calculated starting from the next instruction
                    self.pc += 1;

                    cycles = 12;
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs();
                        self.pc.saturating_sub(immediate as u16)
                    };

                    return cycles;
                }

                cycles = 8;
            }
            0x38 => {
                // JR C,n
                //
                // Jump to the current address + n (signed) if C is 1.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                self.current_op = format!("{0:#04x}      JR C,{0:#04x} {0}", immediate);

                self.pc += 1;

                cycles = 8;
                if (self.regs.flags & CARRY_FLAG) != 0 {
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs();
                        self.pc.saturating_sub(immediate as u16)
                    };
                    cycles = 12;
                }
                return cycles;
            }
            0x0b | 0x1b | 0x2b | 0x3b => {
                // 16bit DEC
                //
                // No flags affected.

                let reg_name = match (opcode & 0xF0) >> 4 {
                    0x0 => {
                        self.regs.write_bc(self.regs.bc() - 1);

                        "BC"
                    }
                    0x1 => {
                        self.regs.write_de(self.regs.de() - 1);

                        "DE"
                    }
                    0x2 => {
                        self.regs.write_hl(self.regs.hl() - 1);

                        "HL"
                    }
                    0x3 => {
                        self.sp -= 1;

                        "SP"
                    }
                    _ => unreachable!(),
                };

                self.current_op = format!("{:9} DEC {}", " ", reg_name);
                cycles = 8;
            }
            0x04 | 0x14 | 0x24 | 0x34 | 0x0c | 0x1c | 0x2c | 0x3c => {
                // INC r8
                //
                // Increment 8 bit register r8.
                //
                // flags:
                // Z: Set if result is zero, unset otherwise
                // N: 0
                // H: Set if carry from bit 3, unset otherwise
                // C: no change.

                let reg_index = (opcode & 0b0011_1000) >> 3;

                let location = self.index(reg_index);
                let value = location.load8(&self.regs, &memory).wrapping_add(1);

                location.store8(&mut self.regs, &mut memory, value);

                self.regs.set_flag(ZERO_FLAG, value == 0);
                self.regs.set_flag(SUBTRACT_FLAG, false);
                self.regs.set_flag(HALF_CARRY_FLAG, value & 0xf == 0);

                self.current_op = format!("{:9} INC {}", " ", location);
                cycles = if opcode == 0x34 { 12 } else { 4 };
            }
            0x02 => {
                // LD (BC), A

                let address = self.regs.bc() as usize;
                memory.write(address, self.regs.a);

                self.current_op = format!("LD (BC),A");
                cycles = 8;
            }
            0x12 => {
                // LD (DE),A

                let address = self.regs.de() as usize;
                memory.write(address, self.regs.a);

                self.current_op = format!("LD (DE),A");
                cycles = 8;
            }

            0x22 => {
                // LD (HL+),A

                let address = self.regs.hl() as usize;
                memory.write(address, self.regs.a);
                self.regs.write_hl(self.regs.hl() + 1);

                self.current_op = format!("LD (HL+),A");
                cycles = 8;
            }
            0x32 => {
                // LD (HL-),A
                //
                // Copy A into memory address HL and decrement HL.

                let address = self.regs.hl() as usize;
                memory.write(address, self.regs.a);
                self.regs.write_hl(self.regs.hl() - 1);

                self.current_op = format!("LD (HL-),A");
                cycles = 8;
            }
            0x76 => {
                // HALT
                // TODO check if cycles are ok
                cycles = 16;
            }
            (0x40..=0x7f) => {
                // LD r1,r2
                //
                // Store 8 bit register r2 into 8 bit register r1

                let src_index = opcode & 0b0000_0111;
                let src_location = self.index(src_index);
                let src_value = src_location.load8(&self.regs, &memory);

                let dst_index = (opcode & 0b0011_1000) >> 3;
                let dst_location = self.index(dst_index);

                dst_location.store8(&mut self.regs, &mut memory, src_value);

                self.current_op = format!("LD {},{}", dst_location, src_location);
                cycles = if let (Location::Address(_), Location::Address(_)) =
                    (src_location, dst_location)
                {
                    8
                } else {
                    4
                };
            }
            0x80..=0xbf | 0xc6 | 0xd6 | 0xe6 | 0xf6 | 0xce | 0xde | 0xee | 0xfe => {
                // Opcodes operating on A and 8 bit n,
                // where n = A, B, C, D, E, H, L, (HL), immediate.

                let n = if opcode > 0xbf {
                    // immediate
                    self.pc += 1;
                    cycles = 8;

                    memory.load(self.pc as usize)
                } else {
                    let reg_index = opcode & 0b0111;
                    let location = self.index(reg_index);
                    cycles = if let Location::Address(_) = location {
                        8
                    } else {
                        4
                    };

                    location.load8(&self.regs, &memory)
                };

                match (opcode & 0b0011_1000) >> 3 {
                    0 => {
                        // ADD A, n
                        //
                        // Add n to A.
                        //
                        // flags:
                        // Z: Set if result is zero.
                        // N: 0
                        // H: Set if carry from bit 3.
                        // C: Set if carry from bit 7.

                        let half_carry = (self.regs.a & 0xf) + (n & 0xf) > 0xf;
                        self.regs.set_flag(HALF_CARRY_FLAG, half_carry);

                        let (result, carry) = self.regs.a.overflowing_add(n);

                        self.regs.a = result;
                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);
                        self.regs.set_flag(SUBTRACT_FLAG, false);
                        self.regs.set_flag(CARRY_FLAG, carry);
                    }
                    1 => {
                        // ADC A, n
                        //
                        // Add n + Carry flag to A.
                        //
                        // flags:
                        // Z: Set if result is zero.
                        // N: 0
                        // H: Set if carry from bit 3.
                        // C: Set if carry from bit 7.

                        let carry = if self.regs.flags & CARRY_FLAG == 0 {
                            0
                        } else {
                            1
                        };

                        let result = self.regs.a as u16 + carry as u16 + n as u16;

                        self.regs.a = result as u8;

                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);
                        self.regs.set_flag(SUBTRACT_FLAG, false);

                        self.regs.set_flag(CARRY_FLAG, result > 0xff);

                        let half_carry = (self.regs.a & 0xf) + (n & 0xf) + carry > 0xf;
                        self.regs.set_flag(HALF_CARRY_FLAG, half_carry);
                    }
                    2 => {
                        // SUB n
                        //
                        // Subtract n from A.
                        //
                        // flags:
                        // Z: Set if result is zero
                        // N: 1
                        // H: Set if borrow from bit 4
                        // C: Set if no borrow

                        let (_, borrow) = (self.regs.a & 0xf).overflowing_sub(n & 0xf);
                        self.regs.set_flag(HALF_CARRY_FLAG, borrow);

                        let (result, carry) = self.regs.a.overflowing_sub(n);

                        self.regs.a = result;
                        self.regs.set_flag(CARRY_FLAG, carry);
                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);
                        self.regs.set_flag(SUBTRACT_FLAG, true);
                    }
                    3 => unimplemented!("{:#04x} SBC A, opcode", self.pc),
                    4 => {
                        // AND n
                        //
                        // Logically AND n with A, result in A.
                        //
                        // flags:
                        // Z: Set if result is zero.
                        // N: 0, H: 1, C: 0

                        self.regs.a &= n as u8;

                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);
                        self.regs.set_flag(SUBTRACT_FLAG, false);
                        self.regs.set_flag(HALF_CARRY_FLAG, true);
                        self.regs.set_flag(CARRY_FLAG, false);
                    }
                    5 => {
                        // XOR n
                        //
                        // Logical exclusive OR n with register A, result in A.
                        //
                        //
                        // flags:
                        // Z: Set if result is zero
                        // N: 0, H: 0, C: 0

                        self.regs.a ^= n as u8;

                        self.regs.flags = 0;
                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);
                    }
                    6 => {
                        // OR n
                        //
                        // Logical OR n with register A, result in A.
                        //
                        // flags:
                        // Z: Set if result is zero.
                        // N: 0, H: 0, C: 0
                        self.regs.a |= n as u8;

                        self.regs.flags = 0;
                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);
                    }
                    7 => {
                        // CP n
                        //
                        // Compare A with n. This is basically an A - n  subtraction instruction
                        // but the results are thrown away.
                        //
                        // flags:
                        // Z: Set if result is zero. (A == n)
                        // N: 1
                        // H: Set if borrow from bit 4.
                        // C: Set for no borrow. (Set if A < n)

                        self.regs.set_flag(ZERO_FLAG, self.regs.a == n);
                        self.regs.set_flag(SUBTRACT_FLAG, true);
                        self.regs.set_flag(CARRY_FLAG, n > self.regs.a);

                        let (_, borrow) = (self.regs.a & 0xf).overflowing_sub(n & 0xf);
                        self.regs.set_flag(HALF_CARRY_FLAG, borrow);
                    }
                    _ => panic!("Invalid opcode {:#04x} at {:#04x}", opcode, self.pc),
                }
            }
            0xc0 => {
                // RET NZ
                //
                // Return if Z flag is 0.

                if (self.regs.flags & ZERO_FLAG) == 0 {
                    let high = memory.load(self.sp as usize);
                    self.sp += 1;
                    let low = memory.load(self.sp as usize);
                    self.sp += 1;

                    self.pc = u16::from_be_bytes([high, low]);

                    cycles = 20;
                    return cycles;
                }

                cycles = 8;
            }
            0xc8 => {
                // RET NZ
                //
                // Return if Z flag is 1.

                if (self.regs.flags & ZERO_FLAG) != 0 {
                    let high = memory.load(self.sp as usize);
                    self.sp += 1;
                    let low = memory.load(self.sp as usize);
                    self.sp += 1;

                    self.pc = u16::from_be_bytes([high, low]);

                    cycles = 20;
                    return cycles;
                }
                cycles = 8;
            }
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                // POP r16
                //
                // Pop two bytes off stack into 16 bits register r16 and
                // increment the stack pointer twice.
                // (AF, BC, DE, HL)

                let high = memory.load(self.sp as usize);
                self.sp += 1;

                let low = memory.load(self.sp as usize);
                self.sp += 1;

                let value = u16::from_be_bytes([high, low]);

                let reg_name = match (opcode & 0xF0) >> 4 {
                    0xc => {
                        self.regs.write_bc(value);

                        "BC"
                    }
                    0xd => {
                        self.regs.write_de(value);

                        "DE"
                    }
                    0xe => {
                        self.regs.write_hl(value);

                        "HL"
                    }
                    0xf => {
                        self.regs.write_af(value);

                        "AF"
                    }
                    _ => unreachable!(),
                };

                self.current_op = format!("{:9} POP {}", " ", reg_name);
                cycles = 12;
            }
            0xc9 => {
                // RET
                //
                // Pop two bytes from stack and jump to that address.

                let high = memory.load(self.sp as usize);
                self.sp += 1;
                let low = memory.load(self.sp as usize);
                self.sp += 1;

                self.pc = u16::from_be_bytes([high, low]);

                self.current_op = format!("{:10} RET {:#04x}", " ", self.pc);

                cycles = 16;
                return cycles;
            }
            0xc5 | 0xd5 | 0xe5 | 0xf5 => {
                // PUSH r16
                //
                // Push register r16 onto stack and decrement the
                // stack pointer twice.
                // (BC, DE, HL, AF)

                self.sp -= 1;

                let reg_name = match (opcode & 0xF0) >> 4 {
                    0xc => {
                        memory.write(self.sp as usize, self.regs.c);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.b);

                        "BC"
                    }
                    0xd => {
                        memory.write(self.sp as usize, self.regs.e);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.d);

                        "DE"
                    }
                    0xe => {
                        memory.write(self.sp as usize, self.regs.l);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.h);

                        "HL"
                    }
                    0xf => {
                        memory.write(self.sp as usize, self.regs.flags);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.a);

                        "AF"
                    }
                    _ => unreachable!(),
                };

                self.current_op = format!("{:9} PUSH {}", " ", reg_name);
                cycles = 16;
            }
            0xc4 | 0xcc | 0xcd | 0xd4 | 0xdc => {
                // CALL condition, nn
                //
                // Push address of next instruction onto stack and jump to
                // address nn.
                //
                // condition:
                //   None: Always call
                //   NZ: Call if Z flag is 0
                //   Z:  Call if Z flag is 1
                //   NC: Call if C flag is 0
                //   C:  Call if C flag is 1

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let condition = match opcode {
                    // CALL
                    0xcd => true,
                    // CALL NZ
                    0xc4 => self.regs.flags & ZERO_FLAG == 0,
                    // CALL Z
                    0xcc => self.regs.flags & ZERO_FLAG != 0,
                    // CALL NC
                    0xd4 => self.regs.flags & CARRY_FLAG == 0,
                    // CALL C
                    0xdc => self.regs.flags & CARRY_FLAG != 0,
                    _ => unreachable!(),
                };

                cycles = if condition { 24 } else { 12 };

                if condition {
                    let next_inst_addr = self.pc + 1;

                    self.sp -= 1;
                    memory.write(self.sp as usize, next_inst_addr.to_be_bytes()[1]);

                    self.sp -= 1;
                    memory.write(self.sp as usize, next_inst_addr.to_be_bytes()[0]);

                    let addr = u16::from_be_bytes([high, low]);

                    self.pc = addr;
                    return cycles;
                }
            }
            0xcb => {
                self.pc += 1;
                let cb_opcode = memory.load(self.pc as usize);

                match (cb_opcode & 0b1100_0000) >> 6 {
                    0 => {
                        let reg_index = cb_opcode & 0b111;

                        match (cb_opcode & 0b0011_1000) >> 3 {
                            0 => {
                                // 8-bit rotation to the left. The bit leaving on the left
                                // is copied into the carry, and to bit 0.

                                let location = self.index(reg_index);
                                let value = location.load8(&self.regs, &memory).rotate_left(1);

                                location.store8(&mut self.regs, &mut memory, value);

                                self.regs.set_flag(CARRY_FLAG, value & 0x1 != 0);

                                if value == 0 {
                                    self.regs.flags |= ZERO_FLAG;
                                }

                                self.regs.flags &= !SUBTRACT_FLAG;
                                self.regs.flags &= !HALF_CARRY_FLAG;

                                self.current_op =
                                    format!("{:#04x}      RLC {}", cb_opcode, location);
                            }
                            1 => {
                                self.current_op = "RRC".to_string();
                                unimplemented!();
                            }
                            2 => {
                                // RL r8
                                //
                                // 9-bit rotation to the left using the carry flag.
                                // The 8 bit register r8's bits are shifted left,
                                // the carry value is put into 0th bit of the register,
                                // and the leaving 7th bit is put into the carry.
                                //
                                //  before:
                                //
                                //  +---+  +-------------------------------+
                                //  | 8 |  | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
                                //  +---+  +-------------------------------+
                                //      C                                 r8
                                //
                                //  after
                                //
                                //  +---+  +-------------------------------+
                                //  | 7 |  | 6 | 5 | 4 | 3 | 2 | 1 | 0 | 8 |
                                //  +---+  +-------------------------------+
                                //      C                                 r8
                                //
                                // flags:
                                // Z: Set if result is zero, unset otherwise
                                // N: 0
                                // H: 0
                                // C: Contains old bit 7 data.

                                let carry = (self.regs.flags & CARRY_FLAG) >> 4;

                                let location = self.index(reg_index);
                                let initial_value = location.load8(&self.regs, &memory);

                                let value = initial_value << 1 | carry;

                                location.store8(&mut self.regs, &mut memory, value);

                                self.regs
                                    .set_flag(CARRY_FLAG, initial_value & 0b1000_0000 != 0);
                                self.regs.set_flag(ZERO_FLAG, value == 0);

                                self.regs.set_flag(SUBTRACT_FLAG, false);
                                self.regs.set_flag(HALF_CARRY_FLAG, false);

                                self.current_op =
                                    format!("{:#04x}      RL {}", cb_opcode, location);
                            }
                            3 => {
                                // 9-bit rotation to the right. The carry is copied into bit 7,
                                // and the bit leaving on the right is copied into the carry.

                                let carry = (self.regs.flags & CARRY_FLAG) >> 4;

                                let location = self.index(reg_index);
                                let initial_value = location.load8(&self.regs, &memory);

                                let value = carry << 7 | (initial_value >> 1);

                                location.store8(&mut self.regs, &mut memory, value);

                                self.regs.set_flag(CARRY_FLAG, initial_value & 0x1 != 0);
                                self.regs.set_flag(ZERO_FLAG, value == 0);

                                self.regs.set_flag(HALF_CARRY_FLAG, false);
                                self.regs.set_flag(SUBTRACT_FLAG, false);

                                self.current_op =
                                    format!("{:#04x}      RR {}", cb_opcode, location);
                            }
                            4 => {
                                // TODO better doc
                                // SLA n
                                //
                                // Shift n left into Carry. LSB of n set to 0.
                                //   Z - Set if result is zero.
                                //   N - Reset.
                                //   H - Reset.
                                //   C - Contains old bit 7 data.

                                let location = self.index(reg_index);
                                let initial_value = location.load8(&self.regs, &memory);

                                let value = initial_value << 1;
                                location.store8(&mut self.regs, &mut memory, value);

                                self.regs.set_flag(CARRY_FLAG, initial_value & 0b1000_0000 != 0);
                                self.regs.set_flag(ZERO_FLAG, value == 0);

                                self.regs.set_flag(HALF_CARRY_FLAG, false);
                                self.regs.set_flag(SUBTRACT_FLAG, false);

                                self.current_op = "SLA".to_string();
                            }
                            5 => {
                                self.current_op = "SRA".to_string();
                                unimplemented!();
                            }
                            6 => {
                                // TODO doc

                                let location = self.index(reg_index);
                                let value = location.load8(&self.regs, &memory);

                                let swapped = value&0xf << 4 | value&0xf0 >> 4;

                                location.store8(&mut self.regs, &mut memory, swapped);

                                self.regs.set_flag(ZERO_FLAG, swapped == 0);
                                self.regs.set_flag(CARRY_FLAG, false);
                                self.regs.set_flag(HALF_CARRY_FLAG, false);
                                self.regs.set_flag(SUBTRACT_FLAG, false);

                                self.current_op = "SWAP".to_string();
                            }
                            7 => {
                                // TODO better doc
                                // SRL n
                                //
                                // Shift n right into Carry. MSB set to 0.
                                //   Z - Set if result is zero.
                                //   N - Reset.
                                //   H - Reset.
                                //   C - Contains old bit 0 data.

                                let location = self.index(reg_index);
                                let initial_value = location.load8(&self.regs, &memory);

                                let value = initial_value >> 1;
                                location.store8(&mut self.regs, &mut memory, value);

                                self.regs.set_flag(CARRY_FLAG, initial_value & 0x1 != 0);
                                self.regs.set_flag(ZERO_FLAG, value == 0);

                                self.regs.set_flag(HALF_CARRY_FLAG, false);
                                self.regs.set_flag(SUBTRACT_FLAG, false);

                                self.current_op = "SRL".to_string();
                            }
                            _ => unreachable!(),
                        }
                    }
                    1 => {
                        // BIT n,r8
                        //
                        // Test nth bit in register r8.
                        //
                        // flags:
                        // Z: Set if nth bit of register r8 is 0.
                        // N: 0  H: 1, C: no change

                        let bit_pos = (cb_opcode & 0b0011_1000) >> 3;
                        let mask = 1 << bit_pos;

                        let reg_index = cb_opcode & 0b111;

                        let location = self.index(reg_index);
                        let value = location.load8(&self.regs, &memory);

                        self.regs.set_flag(ZERO_FLAG, value & mask == 0);

                        self.regs.set_flag(SUBTRACT_FLAG, false);
                        self.regs.set_flag(HALF_CARRY_FLAG, true);

                        self.current_op = format!("BIT {},{}", bit_pos, location);
                    }
                    2 => {
                        // TODO check and document
                        let bit_pos = (cb_opcode & 0b0011_1000) >> 3;
                        let mask = !(1 << bit_pos);

                        let reg_index = cb_opcode & 0b111;

                        let location = self.index(reg_index);
                        let value = location.load8(&self.regs, &memory);

                        location.store8(&mut self.regs, &mut memory, value & mask);

                        self.current_op = "RES opcode".to_string();
                    }
                    3 => {
                        // TODO check and document
                        let bit_pos = (cb_opcode & 0b0011_1000) >> 3;
                        let mask = 1 << bit_pos;

                        let reg_index = cb_opcode & 0b111;

                        let location = self.index(reg_index);
                        let value = location.load8(&self.regs, &memory);

                        location.store8(&mut self.regs, &mut memory, value | mask);

                        self.current_op = "SET, opcode".to_string();
                    }
                    _ => panic!("Unknown CB opcode"),
                }

                /* FIXME */
                cycles = 4;
            }
            0x20 => {
                // JR NZ, n
                //
                // Jump to the current address + n (signed) if Z is 0.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                self.current_op = format!("{0:#04x}      JR NZ,{0:#04x} {0}", immediate);

                if (self.regs.flags & ZERO_FLAG) == 0 {
                    // Relative jump is calculated starting from the next instruction
                    self.pc += 1;

                    cycles = 12;
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs();
                        self.pc.saturating_sub(immediate as u16)
                    };
                    return cycles;
                }

                cycles = 8;
            }
            0x2f => {
                // CPL
                //
                // Complement A register. (Flip all bits)
                //
                // flags:
                // Z: no change
                // N: 1
                // H: 1
                // C: no change

                self.regs.a = !self.regs.a;

                self.regs.set_flag(SUBTRACT_FLAG, true);
                self.regs.set_flag(HALF_CARRY_FLAG, true);

                cycles = 4;
            }
            0x30 => {
                // JR NC,n
                //
                // Jump to the current address + n (signed) if C is 0.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                self.current_op = format!("{0:#04x}      JR NC,{0:#04x} {0}", immediate);

                self.pc += 1;

                cycles = 8;
                if (self.regs.flags & CARRY_FLAG) == 0 {
                    cycles = 12;
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs();
                        self.pc.saturating_sub(immediate as u16)
                    };
                }
                return cycles;
            }
            0xc3 => {
                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let addr: u16 = u16::from(high) << 8 | u16::from(low);
                self.pc = addr;

                self.current_op = format!("{:9} JP {:#04x}", " ", addr);

                cycles = 16;
                return cycles;
            }
            0xc2 | 0xca | 0xd2 | 0xda => {
                // JP cc,nn
                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let condition = match opcode {
                    // JP NZ, nn
                    0xc2 => self.regs.flags & ZERO_FLAG == 0,
                    // JP Z, nn
                    0xca => self.regs.flags & ZERO_FLAG != 0,
                    // JP NC, nn
                    0xd2 => self.regs.flags & CARRY_FLAG == 0,
                    // JP C, nn
                    0xda => self.regs.flags & CARRY_FLAG != 0,
                    _ => unreachable!(),
                };

                // TODO check 16? 12?
                cycles = 12;

                if condition {
                    let addr = u16::from_be_bytes([high, low]);

                    self.pc = addr;
                    return cycles;
                }
            }
            0xe0 => {
                // LDH (n),A
                //
                // Store A into memory address 0xff00 + n (signed).

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as usize;

                self.current_op = format!("{0:#04x}      LD (0xff00+{0:#04x}),A", immediate);

                memory.write(0xff00 + immediate, self.regs.a);

                cycles = 12;
            }
            0xe8 => {
                // ADD SP, n
                //
                // Add n (signed) to Stack Pointer (SP).

                self.pc += 1;
                let n = memory.load(self.pc as usize);

                self.regs.set_flag(HALF_CARRY_FLAG, (self.sp & 0xf) + ((n & 0xf) as u16) > 0xf);
                self.regs.set_flag(CARRY_FLAG, (self.sp & 0xff) + (n as u16) > 0xff);

                (self.sp, _) = self.sp.overflowing_add_signed((n as i8).into());

                self.regs.set_flag(ZERO_FLAG, false);
                self.regs.set_flag(SUBTRACT_FLAG, false);

                cycles = 16;
            }
            0xe9 => {
                // JP (HL)
                //
                // Jump to address contained in HL.

                self.pc = self.regs.hl();

                cycles = 4;
                return cycles;
            }
            0xf0 => {
                // LDH A,(n)
                //
                // Store the value from memory address 0xff00 + n (signed)
                // into A.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as usize;

                self.current_op = format!("{0:#04x}      LD (0xff00+{0:#04x}),A", immediate);

                self.regs.a = memory.load(0xff00 + immediate);

                cycles = 12;
            }
            0xe2 => {
                // LD (0xff00+C),A
                //
                // Store A into address 0xff0 + register C.

                memory.write(0xff00 + self.regs.c as usize, self.regs.a);

                self.current_op = "LD (0xff00+C),A".to_string();

                cycles = 8;
            }
            0xea => {
                // LD (a16), A
                //
                // Put value A into memory address a16.

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let addr: u16 = u16::from(high) << 8 | u16::from(low);

                memory.write(addr as usize, self.regs.a);

                self.current_op = format!("{:9} LD ({:#06x}),A", " ", addr);

                cycles = 16;
            }
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
                // RST n
                //
                // Push current address onto stack and jump to address n.
                // n = 0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, 0x38 depending on
                // the opcode.

                // XXX check!
                self.pc += 1;

                self.sp -= 1;
                memory.write(self.sp as usize, (self.pc >> 8) as u8);
                self.sp -= 1;
                println!("sp: {} {:#06x} {:#06x}", self.sp, opcode, self.pc);
                memory.write(self.sp as usize, (self.pc & 0xff) as u8);

                self.pc = u16::from(opcode - 0xc7);

                cycles = 16;
                return cycles;
            }
            0xd9 => {
                // RETI
                //
                // Pop two bytes from stack and jump to that address then enable interrupts.

                let high = memory.load(self.sp as usize);
                self.sp += 1;
                let low = memory.load(self.sp as usize);
                self.sp += 1;

                self.pc = u16::from_be_bytes([high, low]);
                self.interrupts_enabled = true;

                cycles = 16;
                return cycles;
            }
            0xfa => {
                // LD A,(a16)
                //
                // Store the value from memory address a16 into A.

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let immediate = u16::from(high) << 8 | u16::from(low);

                self.regs.a = memory.load(immediate as usize);

                cycles = 16;
            }
            0xf2 => {
                // LD A,(C)
                // aka LD A,($FF00+C)
                //
                // Put value at address 0xFF00 + register C into A.

                self.regs.a = memory.load(self.regs.c as usize + 0xff00);

                self.current_op = format!("{:10} LD A,(C)", " ");

                cycles = 8;
            }
            0xf3 => {
                // DI
                //
                // Disable interrupts.
                // Interrupts are disabled after instruction after DI is executed.

                self.interrupts_enabled = false;
                // TODO skip next instruction

                cycles = 4;
            }
            0xfb => {
                // EI
                //
                // Enable interrupts.
                // Interrupts are enabled after the instruction after EI is executed.

                self.interrupts_enabled = true;
                // TODO skip next instruction

                cycles = 4;
            }
            0xf8 => {
                // LD HL, SP+r8
                //
                // Put SP + n effective address into HL.

                self.pc += 1;
                let n = memory.load(self.pc as usize);

                self.regs.set_flag(HALF_CARRY_FLAG, (self.sp & 0xf) + ((n & 0xf) as u16) > 0xf);
                self.regs.set_flag(CARRY_FLAG, (self.sp & 0xff) + (n as u16) > 0xff);

                let (result, _) = self.sp.overflowing_add_signed((n as i8).into());
                self.regs.write_hl(result);

                self.regs.set_flag(ZERO_FLAG, false);
                self.regs.set_flag(SUBTRACT_FLAG, false);

                cycles = 12
            }
            0xf9 => {
                // LD SP,HL
                //
                // Store the value of HL in SP

                self.sp = self.regs.hl();
                cycles = 8;
            }
            _ => {
                println!("Unknown opcode");
                println!("{:08x}:\t{:#04x} ", self.pc, opcode);
                panic!();
            }
        }

        self.pc += 1;

        cycles
    }

    pub fn mem_next(&self) {
        println!(
            "-> pc: {:#08x}\n   {:#04x} {:#04x}",
            self.pc,
            self.memory.borrow().load(self.pc as usize),
            self.memory.borrow().load((self.pc + 1) as usize)
        );
    }

    pub fn vblank_int(&mut self) {
        self.sp -= 1;
        self.memory
            .borrow_mut()
            .write(self.sp as usize, (self.pc & 0xff) as u8);

        self.sp -= 1;
        self.memory
            .borrow_mut()
            .write(self.sp as usize, (self.pc >> 8) as u8);
        // XXX: is this right?
        self.interrupts_enabled = false;

        self.pc = 0x40;
    }

    pub fn status_int(&mut self) {
        self.sp -= 1;
        self.memory
            .borrow_mut()
            .write(self.sp as usize, (self.pc >> 8) as u8);
        self.sp -= 1;
        self.memory
            .borrow_mut()
            .write(self.sp as usize, (self.pc & 0xff) as u8);


        println!("STATUS pc {:#04x}", self.sp);

        // XXX: is this right?
        self.interrupts_enabled = false;

        self.pc = 0x48;
    }

}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "pc: {:#06x}, sp: {:#06x}, {}",
            self.pc, self.sp, self.regs
        )
    }
}
