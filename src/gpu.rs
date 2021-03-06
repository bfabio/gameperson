use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;

use crate::memory::Memory;

const BYTES_PER_PIXEL: u8 = 4; // RGBA8888
const BUFFER_HEIGHT: u16 = 256;
const BUFFER_WIDTH: u16 = 256;

const BUFFER_SIZE: usize =
    BUFFER_HEIGHT as usize * BUFFER_WIDTH as usize * BYTES_PER_PIXEL as usize;

pub struct Buffer {
    pub buffer: [u8; BUFFER_SIZE],
}

impl Buffer {
    pub const fn new() -> Self {
        Self {
            buffer: [0; BUFFER_SIZE],
        }
    }
}

pub struct Gpu {
    memory: Rc<RefCell<Memory>>,

    // The current vertical scanline being drawn.
    //
    // It can hold any value between 0 through 153.
    // The values between 144 and 153 indicate the V-Blank period.
    //
    // Writing will reset the counter.
    pub ly: u8,

    // The Y position in the 256x256 pixels BG map (32x32 tiles)
    // which is to be displayed at the upper/left LCD display position.
    pub scy: u8,

    // The X position in the 256x256 pixels BG map (32x32 tiles)
    // which is to be displayed at the upper/left LCD display position.
    pub scx: u8,

    // TODO doc
    // FF40 - LCDC - LCD Control (R/W)
    // Bit 7 - LCD Display Enable             (0=Off, 1=On)
    // Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
    // Bit 5 - Window Display Enable          (0=Off, 1=On)
    // Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
    // Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
    // Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
    // Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
    // Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
    pub lcdc: u8,


    // TODO  FF41 - STAT - LCDC Status (R/W)
    pub stat: u8,

    // TODO doc LY Compare
    pub lyc: u8,


    // TODO doc
    // internal cycles counter
    cycles: u16,
}

pub enum Interrupt {
    VBlank,
    Status,
}

impl Gpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Self {
        Self {
            memory,
            ly: 0,
            scy: 0,
            scx: 0,
            cycles: 0,
            lcdc: 0,
            lyc: 0,
            stat: 0,
        }
    }

    pub fn display(
        &mut self,
        canvas: &mut Canvas<Window>,
        texture: &mut Texture,
        buffer: &mut Buffer,
        cycles: u16,
    ) -> Option<Interrupt> {
        // TODO document
        // XXX is this right?
        // Bit 7 - LCD Display Enable (0=Off, 1=On)
        if self.lcdc & 0b1000_0000 == 0 {
            return None
        }

        self.cycles += cycles;

        // println!("gpu cycles {:#04x} ly {:#04x}", cycles, self.ly);
        // TODO document
        // A new scanline every 116 ticks (1MHz clock CPU)
        if self.cycles >= 116 {
            /* ly range is 0 through 153 (0x99) */
            self.ly = if self.ly == 153 { 0 } else { self.ly + 1 };

            self.cycles = 0;
        }

        if self.ly == 0 {
            let memory = self.memory.borrow();

            let mut tile_x: u8;
            let mut tile_y: u8;

            let tile_map_range = if self.lcdc & 0b1000 == 0 {
                // BG Map Data 1
                0x9800..=0x9bff
            } else {
                0x9c00..=0x9fff
                // BG Map Data 2
            };

            for (i, tile_addr) in tile_map_range.enumerate() {
                let tile_num = memory.load(tile_addr);

                tile_x = (i % 32) as u8;
                tile_y = (i / 32) as u8;

                self.print_tile(self.get_tile(tile_num), &mut buffer.buffer, tile_x, tile_y);
            }
            texture
                .update(
                    None,
                    &buffer.buffer,
                    BUFFER_WIDTH as usize * BYTES_PER_PIXEL as usize,
                )
                .unwrap();
        }

        // LYC=LY Coincidence Interrupt enabled
        // if self.stat & 0b100_0000 != 0 && self.lyc == self.ly {
        //     return Some(Interrupt::Status);
        // }

        // VBlank
        if self.ly == 144 {
            let scanline_src = Rect::new(0, self.scy as i32, 160, 144);

            canvas.copy(texture, scanline_src, None).unwrap();

            canvas.present();

            return Some(Interrupt::VBlank);
        }

        None
    }

    fn get_tile(&self, tile_num: u8) -> [u8; 16] {
        let memory = self.memory.borrow();

        // TODO doc
        // Bit 4 - BG & Window Tile Data Select (0=8800-97FF, 1=8000-8FFF)
        let tiles_address = if self.lcdc & 0b1_0000 == 0 { 0x8800 } else { 0x8000 };

        let tile_start = tiles_address + u16::from(tile_num) * 16;
        let tile_end = tile_start + 16;

        let mut tile: [u8; 16] = [0; 16];

        // Tile RAM
        for (i, addr) in (tile_start..tile_end).enumerate() {
            tile[i] = memory.load(addr as usize);
        }

        tile
    }

    fn palette_color() {
        // FF47 - BGP - BG Palette Data (R/W) - Non CGB Mode Only
        // This register assigns gray shades to the color numbers of the BG and Window tiles.
        
        //   Bit 7-6 - Shade for Color Number 3
        //   Bit 5-4 - Shade for Color Number 2
        //   Bit 3-2 - Shade for Color Number 1
        //   Bit 1-0 - Shade for Color Number 0

        unimplemented!();
    }

    fn print_tile(&self, tile: [u8; 16], buffer: &mut [u8], x: u8, y: u8) {
        assert!(x < 32);
        assert!(y < 32);

        let mut xx;
        let mut yy;
        for row in 0..=7 {
            let b = (tile[row * 2], tile[1 + row * 2]);

            for col in 0..=7 {
                let palette = (b.0 & (1 << col), b.1 & (1 << col));
                let color = match palette {
                    (0, 0) => (0xff, 0xff, 0xff, 0xff),
                    (_, 0) => (0xff, 0x00, 0x00, 0x00),
                    (0, _) => (0xff, 0x00, 0xff, 0x00),
                    (_, _) => (0xff, 0xff, 0x00, 0x00),
                };

                xx = x as i32 * 8 + (col as i8 - 7).abs() as i32;
                yy = (y as i32 * 8) + row as i32;

                let index =
                    (xx as usize + yy as usize * BUFFER_WIDTH as usize) * BYTES_PER_PIXEL as usize;

                // 4 bytes per pixel
                buffer[index] = color.0;
                buffer[index + 1] = color.1;
                buffer[index + 2] = color.2;
                buffer[index + 3] = color.3;
            }
        }
    }
}

impl<'a> fmt::Display for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "abc")
    }
}
