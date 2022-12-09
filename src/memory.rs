use std::cell::RefCell;
use std::rc::Rc;

use std::fmt;
use std::ops::Range;

use crate::gpu::Gpu;

///!  0x0000              0x4000             0x8000                                 0xffff
///!    ↑                    ↑                  ↑                                      ↑
///!    +--+--+---------------------------------+-------+---+-------+--------+----+----+
///!    |🔴|🕹|      💾      |       💾         |   👾  |⛰🏔|       |        |    |    |
///!    +--+--+--------------+------------------+-------+---+-------+--------+----+----+
///!
///!    🔴 0x0000-0x00ff - Restart and Interrupt Vectors (255 bytes)
///!    ============================================================
///!    Jump vectors for the 8 RST opcodes:
///!      * RST 00h (at 0x0000)
///!      * RST 08h (at 0x0008)
///!      * RST 10h (at 0x0010)
///!      * RST 18h (at 0x0018)
///!      * RST 20h (at 0x0020)
///!      * RST 28h (at 0x0028)
///!      * RST 30h (at 0x0030)
///!      * RST 38h (at 0x0038)
///
///!    and the Interrupt Vector Table for the following interrupts:
///!      * V-Blank (at 0x0040)
///!      * LCDC    (at 0x0048)
///!      * Timer   (at 0x0050)
///!      * Serial  (at 0x0058)
///!      * Joypad  (at 0x0060)
///!
///!    When an interrupt is the program stops where it is, and jumps to the
///!    specified location in the Vector Table.
///!
///!    🕹 0x0100-0x014f - Cartridge Header Area
///!    ========================================
///!    Information about the inserted cartridge, including; type of cartridge,
///!    size of rom, size of ram, a Nintendo logo, and other information.
///!
///!    💾 0x0150-0x3fff - Cartridge ROM, Bank 0 (16048 bytes)
///!    ======================================================
///!
///!    💾 0x4000-0x7fff - Cartridge ROM, Switchable banks (16384 bytes, 16kB)
///!    ======================================================================
///!
///!    👾 0x8000-0x97ff - Tile RAM (AKA Character RAM)
///!    ===============================================
///!    Portion of the VRAM holding tiles.
///!    Each tile is 8x8 pixels of 2-bit color, which makes each tile 16
///!    bytes long (2 bytes per line).
///!
///!     ▤ □ ■ ■ ■ ■ □ ▤  (0x42 0x3c) |------+
///!     □ ■ □ □ □ □ ■ □                     |
///!     ■ □ ■ □ □ ■ □ ■                     |
///!     ■ □ □ □ □ □ □ ■                     |
///!     ■ □ ■ □ □ ■ □ ■                     |
///!     ■ □ □ ■ ■ □ □ ■                     |
///!     □ ■ □ □ □ □ ■ □                     |
///!     ▤ □ ■ ■ ■ ■ □ ▤                     |
///!                                         |
///!       ▤   □   ■   ■   ■   ■   □   ▤     |
///!     +---+---+---+---+---+---+---+---+   v
///!     | 0 | 1 | 0 | 0 | 0 | 0 | 1 | 0 | (0x42)
///!     | 0 | 0 | 1 | 1 | 1 | 1 | 0 | 0 | (0x3c)
///!     +---+---+---+---+---+---+---+---+
///!
///!    Palette colors (2bits):
///!    00: ▤
///!    01: ■
///!    10: □
///!    11: ▥
///!
///!    This area is also divided up into two modes of tiles, signed and
///!    unsigned. In unsigned mode, tiles are numbered from 0-255 at $8000-$9000. In
///!    signed mode, tiles are numbered in two's complement from -127 to 128 at
///!    $87FF-$97FF. Generally most programs use 0-255 tiles, since it's
///!    nice and easy. XXX
///!
///!    ⛰  0x9800-0x9bff - BG Map Data 1 (1024 bytes)
///!    =============================================
///!    This area is what the video processor uses to build the display.
///!
///!    Each byte represents an 8x8 pixel space on the display. This area
///!    is 32x32 tiles large. The display processor takes each byte
///!    and then goes into the Character RAM area and gets the corresponding tile from
///!    that area and draws it to the screen. So, if the first byte in the Map area
///!    contained 0x40, the display processor would get tile 0x40 from the Character RAM
///!    and put it in the top-left corner of the virtual screen. I say virtual screen
///!    because the actual LCD is only 160x144 pixels in size and is basically a
///!    viewport that can be scrolled around the 32x32 tile (256x256 pixel) background
///!    area in video memory.
///!
///!    🏔 0x9c00-0x9bff (BG Map Data 2)
///!    ================================
///!    This area is just a second background map area like the previous one. To
///!    specify which map the video processor uses to build the background image,
///!    change the appropriate bit in the LCDC I/O register, explained later.
///
///!    0xfe00-0xffff
///!    ====================================
///!    0xfe00-0xfe9f   Sprite Attribute Table (aka OAM)
///!    0xfea0-0xfeff   Not usable
///!    0xff00-0xff7f   I/O Registers
///!    0xff80-0xfffe   High RAM (AKA HRAM, AKA Zero Page)
///!    0xffff          Interrupt Enable Register

pub trait Region {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8) -> Option<AddressSpaceAction>;
    fn len(&self) -> usize;
}

pub enum AddressSpaceAction {
    Unmap(u16),
    DmaTransfer(u8),
}

pub struct Rom {
    mem: Vec<u8>,
}

impl Rom {
    pub fn new(mem: Vec<u8>) -> Self {
        Self { mem }
    }
}

impl Region for Rom {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, _address: u16, _value: u8) -> Option<AddressSpaceAction> {
        None
    }

    fn len(&self) -> usize {
        self.mem.len()
    }
}

pub struct IORegisters {
    pub mem: [u8; 0x7f],
    gpu: Rc<RefCell<Gpu>>,
}

impl IORegisters {
    pub fn new(gpu: Rc<RefCell<Gpu>>) -> Self {
        Self {
            mem: [0; 0x7f],
            gpu,
        }
    }
}

impl Region for IORegisters {
    // Usually at 0xff00 to 0xff7f

    fn read(&self, address: u16) -> u8 {
        match address {
            // Joypad buttons
            // TODO doc
            0x00 => self.mem[0x00],
            // 0xff42 - SCY - Scroll Y
            0x40 => self.gpu.borrow().lcdc,
            0x41 => self.gpu.borrow().stat,
            // 0xff42 - SCY - Scroll Y
            // 0xff43 - SCX - Scroll X
            // Specifies the position in the 256x256 pixels BG map (32x32 tiles)
            // which is to be displayed at the upper/left LCD display position.
            //
            // Values in range from 0-255 may be used for X/Y each, the video
            // controller automatically wraps back to the upper (left) position in
            // BG map when drawing exceeds the lower (right) border of the BG map area.
            // // 0x43 => {
            // }
            0x42 => self.gpu.borrow().scy,
            // 0xff44: LY - LCDC Y-Coordinate
            // Indicates the vertical line to which the present data is
            // transferred to the LCD Driver.
            //
            // It can hold any value between 0 through 153.
            // The values between 144 and 153 indicate the V-Blank period.
            //
            // Writing will reset the counter.
            0x44 => self.gpu.borrow().ly,

            // TODO doc
            0x45 => self.gpu.borrow().lyc,
            _ => {
                self.mem[address as usize]
            }
        }
    }
    fn write(&mut self, address: u16, value: u8) -> Option<AddressSpaceAction> {
        match address {
            0x00 => {
                self.mem[0x00] = value;
                None
            }
            0x01 => {
                // println!("Serial: {}", value);
                None
            }
            // 0xff02: SC - Serial Transfer Control (R/W)
            0x02 => {
                if value == 0x81 {
                    // Print 0xff01 (SB - Serial transfer data (R/W))
                    print!("{}", self.mem[0x01]);
                }
                None
            }
            // 0xff40: LCDC - LCD Control (R/W)
            0x40 => {
                self.gpu.borrow_mut().lcdc = value;
                None
            },
            0x41 => {
                self.gpu.borrow_mut().stat = value;
                None
            }
            // 0xff42 - SCY - Scroll Y
            // 0xff43 - SCX - Scroll X
            // Specifies the position in the 256x256 pixels BG map (32x32 tiles)
            // which is to be displayed at the upper/left LCD display position.
            //
            // Values in range from 0-255 may be used for X/Y each, the video
            // controller automatically wraps back to the upper (left) position in
            // BG map when drawing exceeds the lower (right) border of the BG map area.
            // 0x43 => {
            // }
            0x42 => {
                self.gpu.borrow_mut().scy = value;
                None
            }
            0x44 => {
                unimplemented!();
            }
            // FF45 - LYC - LY Compare (R/W)
            // The gameboy permanently compares the value of the LYC and LY registers.
            // When both values are identical, the coincident bit in the STAT register
            // becomes set, and (if enabled) a STAT interrupt is requested.
            0x45 => {
                self.gpu.borrow_mut().lyc = value;
                None
            }
            // Writing to this register launches a DMA transfer from ROM or RAM to
            // Sprite Attribute Table (aka OAM)
            // The written value specifies the transfer source address divided by 0x100h
            //
            // eg.
            // value == 0x1b
            // copies   0x1b00-0x1b9f to
            //          0xfe00-0xfe9f
            //
            // value can be 0x00 to 0xf1
            0x46 => {
                Some(AddressSpaceAction::DmaTransfer(value))
            }
            // 0xff50: Unmap the boot ROM (TODO: Find the documentation)
            0x50 => {
                Some(AddressSpaceAction::Unmap(0x0000))
            },
            _ => {
                //println!(
                //    "(Fake write to I/O register at {:#06x} ({:#04x})) ",
                //    address, value
                //);
                self.mem[address as usize] = value;
                None
            }
        }
    }

    fn len(&self) -> usize {
        self.mem.len()
    }
}

pub struct Vram {
    mem: [u8; 0x2000], // 8KiB
}

impl Vram {
    pub const fn new() -> Self {
        Self { mem: [0; 0x2000] }
    }
}

impl Region for Vram {
    // Usually at 0x8000 to 0x9fff

    fn read(&self, address: u16) -> u8 {
        if address == 0x1910 {
            return 0x19_u8;
        }

        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) -> Option<AddressSpaceAction> {
        /*println!(
            "(Write to VRAM at {:#06x} ({:#04x}))",
            address + 0x8000,
            value
        );*/

        self.mem[address as usize] = value;
        if (0x9800..=0x9fff).contains(&address) {
            println!("Write to {:#04x}: {:#04x}", address, value);
        }
        None
    }

    fn len(&self) -> usize {
        self.mem.len()
    }
}

impl fmt::Display for Vram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // XXX Print just the Tile RAM for now
        // XXX Better method
        writeln!(f, "{:x?}", &self.mem[0x00..0x200])
    }
}

struct Mapping {
    address_range: Range<u16>,
    region: Box<dyn Region>,
}

pub struct Memory {
    ram: Vec<u8>,
    oam: Vec<u8>,
    zero_page: Vec<u8>,
    cartridge: Vec<u8>,
    pub ie: u8,

    mappings: Vec<Mapping>,

    // Whether the Boot ROM is currently mapped at 0x0000.
    boot_rom_mapped: bool,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            // 8KiB
            // ram: vec![0; 0x2000],

            // FIXME
            ram: vec![0; 0x20000],

            /// 0xa0: 160 bytes
            oam: vec![0; 0xa0],

            /// 127 bytes
            zero_page: vec![0; 127],

            cartridge: vec![],
            mappings: vec![],

            // Interrupt Enable (0xffff)
            ie: 0,

            boot_rom_mapped: false,
        }
    }

    pub fn map(&mut self, address: u16, region: Box<dyn Region>) {
        let range = Range {
            start: address,
            end: address + region.len() as u16,
        };
        println!(
            "Mapping {:#06x}:{:#06x}, size: {}",
            range.start,
            range.end,
            region.len()
        );

        let mapping = Mapping {
            address_range: range,
            region,
        };

        self.mappings.push(mapping);

        // The first mapping at 0x0000 is the Boot ROM.
        if address == 0x0000 && ! self.boot_rom_mapped {
            self.boot_rom_mapped = true;
        }
    }

    pub fn unmap(&mut self, address: u16) {
        if let Some(pos) = self
            .mappings
            .iter()
            .position(|mapping| mapping.address_range.contains(&address))
        {
            println!("Unmapping region at {:#04x}", address);

            self.mappings.remove(pos);
        }

        for m in &self.mappings {
            println!(
                "- {:#04x}..{:#04x}",
                m.address_range.start, m.address_range.end
            );
        }
    }

    // XXX doc
    fn dma(&mut self, addr: u8) {
        let address = (addr as u16) << 8;
        for a in address..=address+0x9f {
            // XXX improve
            self.oam[a as usize - address as usize] = self.load(a as usize)
        }
    }

    pub fn load(&self, address: usize) -> u8 {
        if let Some(mapping) = self.mapping(address) {
            return mapping
                .region
                .read(address as u16 - mapping.address_range.start);
        }

        match address {
            // Boot ROM
            (0x0000..=0x00ff) => {
                println!("address: {}", address);
                self.cartridge[address]
            }
            (0x0100..=0x0103) => 0x00,
            // Cartridge ROM
            (0x0104..=0x7fff) => {
                // Happens with the boot ROM which is just 256 bytes.
                // Let's return blank data

                0x00
            }

            // Video RAM
            // (0x8000..=0x9fff) => vram.read(address as u16 - 0x8000),

            // I/O Registers
            // (0xff00..=0xff7f) => region.read(address as u16 - 0xff00),

            // Internal RAM
            (0xc000..=0xdfff) => self.ram[address - 0xc000],

            // Mirror of 0xc000~0xddff (Echo RAM) - Typically not used
            // FIXME Used by Tetris?
            0xe000..=0xfdff => self.ram[address - 0x2000 - 0xc000],

            // Sprite Attribute Table (aka OAM)
            0xfe00..=0xfe9f => self.oam[address - 0xfe00],

            // Zero Page
            (0xff80..=0xfffe) => self.zero_page[address - 0xff80],

            // FIXME: unimplemented: IE Interrupt Enable
            0xffff => self.ie,
            _ => 0x00,
        }
    }

    fn mapping(&self, address: usize) -> Option<&Mapping> {
        let addr = address as u16;

        self.mappings
            .iter()
            .find(|m| m.address_range.contains(&addr))
    }

    fn mapping_mut(&mut self, address: usize) -> Option<&mut Mapping> {
        let addr = address as u16;

        self.mappings
            .iter_mut()
            .find(|m| m.address_range.contains(&addr))
    }

    pub fn write(&mut self, address: usize, value: u8) {
        if let Some(mapping) = self.mapping_mut(address) {
            match mapping.region.write(address as u16 - mapping.address_range.start, value) {
                Some(AddressSpaceAction::Unmap(u)) if self.boot_rom_mapped => {
                    self.boot_rom_mapped = false;
                    self.unmap(u);
                }
                Some(AddressSpaceAction::DmaTransfer(u)) => self.dma(u),
                _ => {},
            }
        }

        match address {
            // Cartridge ROM
            //(0x0..=0x7fff) => panic!(),

            // Internal RAM
            (0xc000..=0xdfff) => self.ram[address - 0xc000] = value,

            // Sprite Attribute Table (aka OAM)
            0xfe00..=0xfe9f => self.oam[address - 0xfe00] = value,

            // Zero Page
            (0xff80..=0xfffe) => self.zero_page[address - 0xff80] = value,

            0xffff => self.ie = value,

            _ => {}
        }
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.ram)
    }
}
