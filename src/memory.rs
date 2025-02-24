use std::fmt;
use std::ops::Range;

use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;

use crate::gpu::Gpu;
use crate::gpu::Interrupt;

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
    fn write(&mut self, address: u16, value: u8);
    fn len(&self) -> usize;
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

    fn write(&mut self, _address: u16, _value: u8) {
        ()
    }

    fn len(&self) -> usize {
        self.mem.len()
    }
}

struct Mapping {
    address_range: Range<u16>,
    region: Box<dyn Region>,
}

pub struct Memory {
    gpu: Gpu,

    ram: Vec<u8>,
    zero_page: Vec<u8>,
    cartridge: Vec<u8>,
    io_registers: Vec<u8>,
    pub ie: u8,

    joy_action: u8,
    joy_direction: u8,

    mappings: Vec<Mapping>,
}

impl Memory {
    pub fn new(gpu: Gpu) -> Self {
        Self {
            gpu,
            // 8KiB
            // ram: vec![0; 0x2000],

            // FIXME
            ram: vec![0; 0x20000],

            io_registers: vec![0; 0x7f],

            // 127 bytes
            zero_page: vec![0; 127],

            cartridge: vec![],
            mappings: vec![],

            joy_action: 0,
            joy_direction: 0,

            // Interrupt Enable (0xffff)
            ie: 0,
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
    }

    pub fn unmap(&mut self, address: u16) {
        if let Some(pos) = self
            .mappings
            .iter()
            .position(|mapping| mapping.address_range.contains(&address))
        {
            println!("Unmapping Boot ROM at {:#04x}", address);

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
        for a in address..=address + 0x9f {
            // XXX improve
            self.gpu.write(a - address + 0xfe00, self.load(a as usize))
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
                if address <= 0x0133 {
                    let nintendo_logo = [
                        0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00, 0x83,
                        0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e,
                        0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63,
                        0x6e, 0x0e, 0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e,
                    ];

                    println!(
                        "ADDR: {:#06x} value: {:#04x}",
                        address,
                        nintendo_logo[address - 0x0104]
                    );
                    println!("Mappings:");
                    for mapping in &self.mappings {
                        println!(
                            "  {:#04x}..{:#04x}",
                            mapping.address_range.start, mapping.address_range.end
                        );
                    }
                    return nintendo_logo[address - 0x0104];
                }
                // Happens with the boot ROM which is just 256 bytes.
                // Let's return blank data

                0x00
            }

            // Video RAM
            (0x8000..=0x9fff) => self.gpu.read(address as u16),

            // I/O Registers
            // (0xff00..=0xff7f)

            // Joypad buttons
            // Bit 7 - Not used
            // Bit 6 - Not used
            // Bit 5 - P15 Select Action buttons    (0=Select)
            // Bit 4 - P14 Select Direction buttons (0=Select)
            // Bit 3 - P13 Input: Down  or Start    (0=Pressed) (Read Only)
            // Bit 2 - P12 Input: Up    or Select   (0=Pressed) (Read Only)
            // Bit 1 - P11 Input: Left  or B        (0=Pressed) (Read Only)
            // Bit 0 - P10 Input: Right or A        (0=Pressed) (Read Only)
            0xff00 => {
                let high_nibble = self.io_registers[0] & 0xf0;

                if self.io_registers[address - 0xff00] & 0b0010_0000 == 0 {
                    high_nibble | !(self.joy_action & 0x0f) & 0xf
                } else if self.io_registers[address - 0xff00] & 0b0001_0000 == 0 {
                    high_nibble | !(self.joy_direction & 0x0f) & 0xf
                } else {
                    0xff
                }
            }
            0xff40 => self.gpu.read(address as u16),
            0xff41 => self.gpu.read(address as u16),

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
            0xff42 => self.gpu.read(address as u16),

            // 0xff44: LY - LCDC Y-Coordinate
            // Indicates the vertical line to which the present data is
            // transferred to the LCD Driver.
            //
            // It can hold any value between 0 through 153.
            // The values between 144 and 153 indicate the V-Blank period.
            //
            // Writing will reset the counter.
            0xff44 => self.gpu.read(address as u16),

            // TODO doc
            0xff45 => self.gpu.read(address as u16),

            0xff47..=0xff48 => self.gpu.read(address as u16),

            // Internal RAM
            (0xc000..=0xdfff) => self.ram[address - 0xc000],

            // Mirror of 0xc000~0xddff (Echo RAM) - Typically not used
            // FIXME Used by Tetris?
            0xe000..=0xfdff => self.ram[address - 0x2000 - 0xc000],

            // Sprite Attribute Table (aka OAM)
            0xfe00..=0xfe9f => self.gpu.read(address as u16),

            // Zero Page
            (0xff80..=0xfffe) => self.zero_page[address - 0xff80],

            // FIXME: unimplemented: IE Interrupt Enable
            0xffff => self.ie,

            _ => 0, // panic!("Unsupported load from address {:#06x}", address),
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
            mapping
                .region
                .write(address as u16 - mapping.address_range.start, value)
        }

        match address {
            // Cartridge ROM
            //(0x0..=0x7fff) => panic!(),

            // Video RAM
            (0x8000..=0x9fff) => self.gpu.write(address as u16, value),

            // Internal RAM
            (0xc000..=0xdfff) => self.ram[address - 0xc000] = value,

            // Sprite Attribute Table (aka OAM)
            0xfe00..=0xfe9f => self.gpu.write(address as u16, value),

            // I/O Registers

            // P1/JOYP: Joypad
            // The eight Game Boy action/direction buttons are arranged as a 2x4 matrix.
            // Select either action or direction buttons by writing to this register,
            // then read out the bits 0-3.

            // Bit 7 - Not used
            // Bit 6 - Not used
            // Bit 5 - P15 Select Action buttons    (0=Select)
            // Bit 4 - P14 Select Direction buttons (0=Select)
            // Bit 3 - P13 Input: Down  or Start    (0=Pressed) (Read Only)
            // Bit 2 - P12 Input: Up    or Select   (0=Pressed) (Read Only)
            // Bit 1 - P11 Input: Left  or B        (0=Pressed) (Read Only)
            // Bit 0 - P10 Input: Right or A        (0=Pressed) (Read Only)
            0xff00 => self.io_registers[address - 0xff00] = value,
            0xff01 => {
                // println!("Serial: {}", value);
            }
            // SC - Serial Transfer Control (R/W)
            0xff02 => {
                if value == 0x81 {
                    // Print 0xff01 (SB - Serial transfer data (R/W))
                    // print!("{}", self.mem[0x01]);
                }
            }
            // LCDC - LCD Control (R/W)
            0xff40 => {
                self.gpu.write(address as u16, value);
                println!("LCDC change: {:08b}", value)
            }
            0xff41 => {
                self.gpu.write(address as u16, value);
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
            0xff42 => {
                self.gpu.write(address as u16, value);
            }
            0xff44 => {
                unimplemented!();
            }
            // LYC - LY Compare (R/W)
            // The gameboy permanently compares the value of the LYC and LY registers.
            // When both values are identical, the coincident bit in the STAT register
            // becomes set, and (if enabled) a STAT interrupt is requested.
            0xff45 => {
                self.gpu.write(address as u16, value);
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
            0xff46 => self.dma(value),
            0xff47..=0xff48 => self.gpu.write(address as u16, value),

            // Unmap the boot ROM (TODO: Find the documentation)
            0xff50 => self.unmap(0x0000),

            // Zero Page
            (0xff80..=0xfffe) => self.zero_page[address - 0xff80] = value,

            0xffff => self.ie = value,

            _ => {}
        }
    }

    // TODO: document bits, this is custom just to keep the state
    pub fn set_joy_state(&mut self, action: u8, direction: u8) {
        self.joy_action |= action;
        self.joy_direction |= direction;
    }

    pub fn unset_joy_state(&mut self, action: u8, direction: u8) {
        self.joy_action &= !action;
        self.joy_direction &= !direction;
    }

    pub fn display(
        &mut self,
        canvas: &mut Canvas<Window>,
        texture: &mut Texture,
        cycles: u16,
    ) -> Option<Interrupt> {
        self.gpu.display(canvas, texture, cycles)
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.ram)
    }
}
