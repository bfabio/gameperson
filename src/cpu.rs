use std::fmt;
use std::cell::RefCell;
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

pub struct Opcode {
    name: String,
    size: u8,
}

impl Location {
    fn load8(&self, regs: &Registers, address_space: &Memory) -> u8 {
        match self {
            Location::Register(reg) => {
                match reg {
                    Register8::A => regs.a,
                    Register8::B => regs.b,
                    Register8::C => regs.c,
                    Register8::D => regs.d,
                    Register8::E => regs.e,
                    Register8::H => regs.h,
                    Register8::L => regs.l,
                }
            }
            Location::DoubleRegister(_) => {
                panic!("load8() called on a 16bit register");
            }
            Location::Address(address) => {
                address_space.load(*address as usize)
            }
        }
    }

    fn load16(&self, regs: &Registers, address_space: &Memory) -> u16 {
        match self {
            Location::Register(_) => {
                panic!("load16() called on a 8bit register");
            }
            Location::DoubleRegister(reg) => {
                match reg {
                    Register16::BC => regs.bc(),
                    Register16::HL => regs.hl(),
                    Register16::DE => regs.de(),
                }
            }
            Location::Address(address) => {
                let (high, low) = (*address as usize, (*address + 1) as usize);
                u16::from(address_space.load(high)) | u16::from(address_space.load(low) << 8)
            }
        }
    }

    fn store8(&self, regs: &mut Registers, address_space: &mut Memory, value: u8) {
        match self {
            Location::Register(reg) => {
                match reg {
                    Register8::A => regs.a = value,
                    Register8::B => regs.b = value,
                    Register8::C => regs.c = value,
                    Register8::D => regs.d = value,
                    Register8::E => regs.e = value,
                    Register8::H => regs.h = value,
                    Register8::L => regs.l = value,
                }
            }
            Location::DoubleRegister(_) => {
                panic!("store8() called on a 16bit register");
            }
            Location::Address(address) => {
                address_space.write(*address as usize, value);
            }
        }
    }

    fn store16(&self, regs: &mut Registers, address_space: &mut Memory, value: u16) {
        match self {
            Location::Register(_) => {
                panic!("store16() called on a 8bit register");
            }
            Location::DoubleRegister(reg) => {
                match reg {
                    Register16::BC => regs.write_bc(value),
                    Register16::HL => regs.write_hl(value),
                    Register16::DE => regs.write_de(value),
                }
            }
            Location::Address(_) => {
                panic!("Unimplemented");
            }
        };
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Location::Register(reg8) => {
                match reg8 {
                    Register8::A => write!(f, "A"),
                    Register8::B => write!(f, "B"),
                    Register8::C => write!(f, "C"),
                    Register8::D => write!(f, "D"),
                    Register8::E => write!(f, "E"),
                    Register8::H => write!(f, "H"),
                    Register8::L => write!(f, "L"),
                }
            }
            Location::DoubleRegister(reg16) => {
                match reg16 {
                    Register16::BC => write!(f, "BC"),
                    Register16::HL => write!(f, "HL"),
                    Register16::DE => write!(f, "DE"),
                }
            }
            Location::Address(addr) => write!(f, "({:#06x})", addr),
        }
    }
}

pub struct Cpu {
    regs: Registers,

    // Stack pointer
    sp: u16,

    // Program counter
    pc: u16,

    cycles: u64,

    memory: Rc<RefCell<Memory>>,
}

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
        self.c = (value & 0xff) as u8;
    }

    pub fn write_de(&mut self, value: u16) {
        self.d = ((value & 0xff00) >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }

    pub fn write_hl(&mut self, value: u16) {
        self.h = ((value & 0xff00) >> 8) as u8;
        self.l = (value & 0xff) as u8;
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
        write!(f, "a: {:#04x}, b: {:#04x}, c: {:#04x}, d: {:#04x}, e: {:#04x}, \
                   h: {:#04x}, l: {:#04x} flags: {:08b}",
               self.a,
               self.b,
               self.c,
               self.d,
               self.e,
               self.h,
               self.l,
               self.flags)
     }
}

const ZERO_FLAG: u8       = 1 << 7;
const SUBTRACT_FLAG: u8   = 1 << 6;
const HALF_CARRY_FLAG: u8 = 1 << 5;
const CARRY_FLAG: u8      = 1 << 4;

impl Cpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Cpu {
        let regs = Registers {
            flags: 0,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
        };

        Cpu {
            regs,
            sp: 0,
            pc: 0,
            cycles: 0,
            memory,
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

    pub fn decode(&mut self) -> String {
        let mut memory = {
            self.memory.borrow_mut()
        };

        let opcode = memory.load(self.pc as usize);

        let mut decoded = format!("{:08x}:\t{:#04x} ", self.pc, opcode);

        macro_rules! println {
            ($($arg:tt)*) => {()}
        }
        macro_rules! print {
            ($($arg:expr),*) => {()}
        }

        match opcode {
            0x00 => println!("NOP"),
            0x01 | 0x11 | 0x21 | 0x31 => {
                // LD r16,n
                //
                // Store 16 bit immediate value n into 16 bit register
                // (BC, DE, HL, SP).

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let immediate: u16 = (high as u16) << 8 | low as u16;

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
                    _ => unreachable!()
                };

                println!("{:#04x} {:#04x} LD {},{:#x}", low, high, reg_name, immediate);
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

                println!("{:9} DEC {}", " ", location);
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

                self.regs.set_flag(CARRY_FLAG, self.regs.a & 0b1000_0000 != 0);

                self.regs.a = self.regs.a << 1 | carry;

                self.regs.set_flag(ZERO_FLAG, false);
                self.regs.set_flag(SUBTRACT_FLAG, false);
                self.regs.set_flag(HALF_CARRY_FLAG, false);

                print!("{:10} RLA", " ");
            }
            0x0a | 0x1a | 0x2a | 0x3a => {
                // LD A,(r16)
                //
                // Store 16 bit value from memory location at register r16 to A.
                // (BC, DE, HL).

                let (value, reg_name) = match (opcode & 0xF0) >> 4 {
                    0 => {
                        (memory.load(self.regs.bc() as usize), "BC")
                    }
                    1 => {
                        println!("DE: {:#06x}", self.regs.de());
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
                    _ => unreachable!()
                };

                self.regs.a = value;

                print!("{:10} LD A,({})", " ", reg_name);
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

                println!("{1:#04x}      LD {0},{1:#x}",
                         location,
                         immediate);
            }
            0x03 | 0x13 | 0x23 | 0x33 => {
                // 16bit INC
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
                        self.sp += 1;

                        "SP"
                    }
                    _ => unreachable!()
                };

                println!("{:9} INC {}", " ", reg_name);
            }
            0x18 => {
                // JR n
                //
                // Jump to the current address + n (signed).

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                println!("{0:#04x}      JR {0:#04x} {0}", immediate);

                self.pc += 1;

                // XXX
                self.pc = if immediate >= 0 {
                    self.pc.saturating_add(immediate as u16)
                } else {
                    let immediate = immediate.abs(); self.pc.saturating_sub(immediate as u16)
                };
                return decoded;
            }
            0x28 => {
                // JR Z,n
                //
                // Jump to the current address + n (signed) if Z is 1.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                println!("{0:#04x}      JR Z,{0:#04x} {0}", immediate);

                self.pc += 1;

                if (self.regs.flags & ZERO_FLAG) != 0 {
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs(); self.pc.saturating_sub(immediate as u16)
                    };
                }
                return decoded;
            }
            0x38 => {
                // JR C,n
                //
                // Jump to the current address + n (signed) if C is 1.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                println!("{0:#04x}      JR C,{0:#04x} {0}", immediate);

                self.pc += 1;

                if (self.regs.flags & CARRY_FLAG) != 0 {
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs(); self.pc.saturating_sub(immediate as u16)
                    };
                }
                return decoded;
            }
            0x0b | 0x1b | 0x2b | 0x3b => {
                // 16bit DEC
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
                    _ => unreachable!()
                };

                println!("{:9} DEC {}", " ", reg_name);
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
                self.regs.set_flag(HALF_CARRY_FLAG, value & 0x0F == 0);

                println!("{:9} INC {}", " ", location);
            }
            0x02 => {
                // LD (BC), A
                print!("{:10}", " ");

                let address = self.regs.bc() as usize;
                memory.write(address, self.regs.a);

                println!("LD (BC),A");
            }
            0x12 => {
                // LD (DE),A
                print!("{:10}", " ");

                let address = self.regs.de() as usize;
                memory.write(address, self.regs.a);

                println!("LD (DE),A");
            }

            0x22 => {
                // LD (HL+),A
                print!("{:10}", " ");

                let address = self.regs.hl() as usize;
                memory.write(address, self.regs.a);
                self.regs.write_hl(self.regs.hl() + 1);

                println!("LD (HL+),A");
            }
            0x32 => {
                // LD (HL-),A
                //
                // Copy A into memory address HL and decrement HL.

                print!("{:10}", " ");

                let address = self.regs.hl() as usize;
                memory.write(address, self.regs.a);
                self.regs.write_hl(self.regs.hl() - 1);

                println!("LD (HL-),A");
            }
            (0x40..=0x7f) => {
                // LD r1,r2
                //
                // Store 8 bit register r2 into 8 bit register r1

                print!("{:10}", " ");

                let src_index = opcode & 0b0000_0111;
                let src_location = self.index(src_index);
                let src_value = src_location.load8(&self.regs, &memory);

                let dst_index = (opcode & 0b0011_1000) >> 3;
                let dst_location = self.index(dst_index);

                dst_location.store8(&mut self.regs, &mut memory, src_value);

                println!("LD {},{}", dst_location, src_location);
            }
            (0x80..=0xbf) => {
                print!("{:10}", " ");

                match (opcode & 0b0011_1000) >> 3 {
                    0 => println!("ADD A, opcode"),
                    1 => println!("ADC A, opcode"),
                    2 => {
                        // SUB r8
                        // Subtract r8 from A.
                        //
                        // flags:
                        // Z: Set if result is zero
                        // N: 1
                        // H: Set if no borrow from bit 4
                        // C: Set if borrow

                        let reg_index = opcode & 0b0111;
                        let location = self.index(reg_index);

                        self.regs.a = self.regs.a.wrapping_sub(
                            location.load8(&self.regs, &memory)
                        );

                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);
                        self.regs.set_flag(SUBTRACT_FLAG, true);

                        // XXX
                        // self.regs.set_flag(HALF_CARRY_FLAG, true);
                        // self.regs.set_flag(CARRY_FLAG, true);

                        println!("SUB {}", location);
                    }
                    3 => println!("SUB A, opcode"),
                    4 => println!("AND opcode"),
                    5 => {
                        // XOR r8
                        // Logical exclusive OR n with register A, result in A.
                        //
                        // flags:
                        // Z: Set if result is zero
                        // N: 0,  H: 0, C: 0

                        let reg_index = opcode & 0b0111;
                        let location = self.index(reg_index);

                        let value = location.load8(&self.regs, &memory) ^ self.regs.a;

                        location.store8(&mut self.regs, &mut memory, value);

                        self.regs.flags = 0;
                        self.regs.set_flag(ZERO_FLAG, self.regs.a == 0);

                        println!("XOR {}", location);
                    }
                    6 => println!("OR opcode"),
                    7 => {
                        self.regs.set_flag(ZERO_FLAG, true); // XXX Remove me
                        println!("CP opcode");
                    }
                    _ => {
                        println!("CPU Status: {}", self);

                        panic!("Unknown opcode");
                    }
                }
            },
            0xc1 | 0xd1 | 0xe1 | 0xf1 => {
                // POP r16
                //
                // Pop two bytes off stack into 16 bits register r16 and
                // increment the stack pointer twice.
                // (AF, BC, DE, HL)

                let low = memory.load(self.sp as usize);
                self.sp += 1;

                let high = memory.load(self.sp as usize);
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
                        // TODO
                        // self.regs.write_af(value);

                        "AF"
                    }
                    _ => unreachable!()
                };

                println!("{:9} POP {}", " ", reg_name);
            }
            0xc9 => {
                let high = memory.load(self.sp as usize);
                self.sp += 1;
                let low = memory.load(self.sp as usize);
                self.sp += 1;

                self.pc = u16::from_be_bytes([high, low]);

                println!("{:10} RET {:#04x}", " ", self.pc);

                return decoded;
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
                        memory.write(self.sp as usize, self.regs.b);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.c);

                        "BC"
                    }
                    0xd => {
                        memory.write(self.sp as usize, self.regs.d);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.e);

                        "DE"
                    }
                    0xe => {
                        memory.write(self.sp as usize, self.regs.h);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.l);

                        "HL"
                    }
                    0xf => {
                        memory.write(self.sp as usize, self.regs.a);

                        self.sp -= 1;
                        memory.write(self.sp as usize, self.regs.flags);

                        "AF"
                    }
                    _ => unreachable!()
                };

                println!("{:9} PUSH {}", " ", reg_name);
            }
            0xcd => {
                // CALL nn
                //
                // Push address of next instruction onto stack and jump to
                // address nn.

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let next_inst_addr = self.pc + 1;

                self.sp -= 1;
                memory.write(self.sp as usize, next_inst_addr.to_be_bytes()[1]);

                self.sp -= 1;
                memory.write(self.sp as usize, next_inst_addr.to_be_bytes()[0]);

                let addr = u16::from_be_bytes([high, low]);

                self.pc = addr;

                println!("{:9} CALL {:#06x}", " ", addr);
                return decoded;
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

                                println!("{:#04x}      RLC {}", cb_opcode, location);
                            }
                            1 => {
                                println!("RRC");
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

                                self.regs.set_flag(CARRY_FLAG, initial_value & 0b1000_0000 != 0);
                                self.regs.set_flag(ZERO_FLAG, value == 0);

                                self.regs.set_flag(SUBTRACT_FLAG, false);
                                self.regs.set_flag(HALF_CARRY_FLAG, false);

                                println!("{:#04x}      RL {}", cb_opcode, location);
                            }
                            3 => {
                                // 9-bit rotation to the right. The carry is copied into bit 7,
                                // and the bit leaving on the right is copied into the carry.

                                let carry = (self.regs.flags & CARRY_FLAG) >> 3;

                                let location = self.index(reg_index);
                                let initial_value = location.load8(&self.regs, &memory);

                                let value = (initial_value >> 1) | carry;

                                location.store8(&mut self.regs, &mut memory, value);

                                self.regs.set_flag(CARRY_FLAG, initial_value & 0x1 != 0);
                                self.regs.set_flag(CARRY_FLAG, value == 0);

                                self.regs.flags &= !SUBTRACT_FLAG;
                                self.regs.flags &= !HALF_CARRY_FLAG;

                                println!("{:#04x}      RR {}", cb_opcode, location);
                            }
                            4 => {
                                println!("SLA");
                            }
                            5 => {
                                println!("SRA");
                            }
                            6 => {
                                println!("SWAP");
                            }
                            7 => {
                                println!("SRL");
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

                        println!("BIT {},{}", bit_pos, location);
                    }
                    2 => println!("RES opcode"),
                    3 => println!("SET, opcode"),
                    _ => panic!("Unknown CB opcode"),
                }
            }
            0x20 => {
                // JR NZ,n
                //
                // Jump to the current address + n (signed) if Z is 0.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                println!("{0:#04x}      JR NZ,{0:#04x} {0}", immediate);

                self.pc += 1;

                if (self.regs.flags & ZERO_FLAG) == 0 {
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs(); self.pc.saturating_sub(immediate as u16)
                    };
                }
                return decoded;
            }
            0x30 => {
                // JR NC,n
                //
                // Jump to the current address + n (signed) if C is 0.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as i8;

                println!("{0:#04x}      JR NC,{0:#04x} {0}", immediate);

                self.pc += 1;

                if (self.regs.flags & CARRY_FLAG) == 0 {
                    // XXX
                    self.pc = if immediate >= 0 {
                        self.pc.saturating_add(immediate as u16)
                    } else {
                        let immediate = immediate.abs(); self.pc.saturating_sub(immediate as u16)
                    };
                }
                return decoded;
            }
            0xc3 => {
                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let addr: u16 = (high as u16) << 8 | low as u16;
                self.pc = addr;

                println!("{:9} JP {:#04x}", " ", addr);
                return decoded;
            }
            0xe0 => {
                // LDH (n),A
                //
                // Store A into memory address 0xff00 + n (signed).

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as usize;

                println!("{0:#04x}      LD (0xff00+{0:#04x}),A", immediate);

                memory.write(0xff00 + immediate, self.regs.a);
            }
            0xf0 => {
                // LDH A,(n)
                //
                // Store the value from memory address 0xff00 + n (signed)
                // into A.

                self.pc += 1;
                let immediate = memory.load(self.pc as usize) as usize;

                println!("{0:#04x}      LD (0xff00+{0:#04x}),A", immediate);

                self.regs.a = memory.load(0xff00 + immediate);
            }
            0xe2 => {
                // LD (0xff00+C),A
                //
                // Store A into address 0xff0 + register C.

                print!("{:10}", " ");

                memory.write(0xff00 + self.regs.c as usize, self.regs.a);

                println!("LD (0xff00+C),A");
            }
            0xea => {
                //

                self.pc += 1;
                let low = memory.load(self.pc as usize);
                self.pc += 1;
                let high = memory.load(self.pc as usize);

                let addr: u16 = (high as u16) << 8 | low as u16;

                self.regs.a = memory.load(addr as usize);

                println!("{:9} LD ({:#06x}),A", " ", addr);

            }
            0xf3 => {
                // LD A,(C)
                //
                // Store A into the memory location at register C.

                self.regs.a = memory.load(self.regs.c as usize);

                print!("{:10} LD A,(C)", " ");

            }
            0xfe => {
                self.pc += 1;
                let immediate = memory.load(self.pc as usize);

                if self.regs.a == immediate {
                    self.regs.flags |= ZERO_FLAG;
                } else {
                    self.regs.flags &= !ZERO_FLAG;
                    if self.regs.a < immediate {
                        self.regs.flags |= CARRY_FLAG;
                    }
                }
                // TODO   H - Set if no borrow from bit 4.

                self.regs.flags |= SUBTRACT_FLAG;

                println!("{:10} CP {:#04x}", " ", immediate);
            }
            _ => println!("Unknown opcode")
        }

        self.pc += 1;
        println!("  {}", self);

        decoded
    }
}

impl fmt::Display for Cpu {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pc: {:#06x}, sp: {:#06x}, {}", self.pc, self.sp, self.regs)
     }
}
