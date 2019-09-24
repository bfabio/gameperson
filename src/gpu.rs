use std::cell::RefCell;
use std::fmt;
use std::io::Write;
use std::rc::Rc;

use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;

use termion::clear;
use termion::color;
use termion::cursor;

use crate::memory::Memory;

const BYTES_PER_PIXEL: u8 = 4; // RGBA8888
const BUFFER_HEIGHT: u16 = 256;
const BUFFER_WIDTH: u16 = 256;

const BUFFER_SIZE: usize =
    (BUFFER_HEIGHT as usize * BUFFER_WIDTH as usize * BYTES_PER_PIXEL as usize);

pub struct GpuBuffer {
    pub buffer: [u8; BUFFER_SIZE],
}

impl GpuBuffer {
    pub fn new() -> Self {
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
}

impl Gpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Gpu {
        Gpu {
            memory,
            ly: 0,
            scy: 0,
            scx: 0,
        }
    }

    pub fn display<W: Write>(
        &mut self,
        /*canvas: &mut Canvas<Window>,
        texture: &mut Texture,*/
        buffer: &mut GpuBuffer,
        screen: &mut W,
    ) {
        if self.ly == 0 {
            let memory = self.memory.borrow();

            let mut tile_x: u8;
            let mut tile_y: u8;

            // BG Map Data 1
            for (i, tile_addr) in (0x9800..=0x9bff).enumerate() {
                let tile_num = memory.load(tile_addr);

                tile_x = (i % 32) as u8;
                tile_y = (i / 32) as u8;

                self.print_tile(self.get_tile(tile_num), &mut buffer.buffer, tile_x, tile_y);
            }
            /*
            texture
                .update(
                    None,
                    &buffer.buffer,
                    BUFFER_WIDTH as usize * BYTES_PER_PIXEL as usize,
                )
                .unwrap(); */
        }

        // VBlank
        if self.ly == 144 {
            // let scanline_src = Rect::new(0, self.scy as i32, 160, 144);

            let offset = self.scy as u32 * BYTES_PER_PIXEL as u32 * BUFFER_WIDTH as u32;
            Gpu::term_out(screen, &buffer, offset);

            // canvas.copy(&texture, scanline_src, None).unwrap();

            // canvas.present();
        }

        self.ly = self.ly.wrapping_add(1);
    }

    fn get_tile(&self, tile_num: u8) -> [u8; 16] {
        let memory = self.memory.borrow();

        let tile_start = 0x8000 + u16::from(tile_num) * 16;
        let tile_end = tile_start + 16;

        let mut tile: [u8; 16] = [0; 16];
        let mut i: usize = 0;

        // Tile RAM
        for a in tile_start..tile_end {
            tile[i] = memory.load(a as usize);
            i += 1;
        }

        tile
    }

    fn print_tile(&self, tile: [u8; 16], buffer: &mut [u8], x: u8, y: u8) {
        assert!(x < 32);
        assert!(y < 32);

        let mut xx;
        let mut yy;
        for row in 0..=7 {
            let b = (tile[row * 2], tile[1 + row * 2]);

            for col in (0..=7).rev() {
                let color = if (b.0 & (1 << col)).count_ones() == 0 {
                    (0xff, 0xff, 0xff, 0xff)
                } else {
                    (0xff, 0x00, 0x00, 0x00)
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

    fn term_out<W: Write>(screen: &mut W, buffer: &GpuBuffer, offset: u32) {
        let buf = buffer.buffer;

        let mut frame = String::new();
        for x in 0..144 {
            // 160 / 2, printing two lines at a time
            for y in 0..70 {
                let yy = y * 2;
                let index = ((x as usize + yy as usize * BUFFER_WIDTH as usize)
                    * BYTES_PER_PIXEL as usize)
                    + offset as usize;

                // 4 bytes per pixel
                // ALPHA buffer[index]
                let (r0, g0, b0) = (buf[index + 1], buf[index + 2], buf[index + 3]);

                let index = ((x as usize + (yy as usize + 1) * BUFFER_WIDTH as usize)
                    * BYTES_PER_PIXEL as usize)
                    + offset as usize;

                let (r1, g1, b1) = (buf[index + 1], buf[index + 2], buf[index + 3]);

                frame.push_str(&format!(
                    "{}{}{}▄",
                    cursor::Goto(1 + x, 1 + 3 + y),
                    color::Bg(color::Rgb(r0, g0, b0)),
                    color::Fg(color::Rgb(r1, g1, b1)),
                ));
            }
        }
        write!(screen, "{}", frame);
    }
}

impl<'a> fmt::Display for Gpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "abc")
    }
}
