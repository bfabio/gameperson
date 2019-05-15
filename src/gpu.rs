use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
// use sdl2::event::Event;
// use sdl2::keyboard::Keycode;
use sdl2::rect::Point;

use crate::memory::Memory;

pub struct Gpu {
    memory: Rc<RefCell<Memory>>,
}

impl Gpu {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Gpu {
        Gpu {
            memory,
        }
    }

    pub fn display(&self, canvas: &mut Canvas<Window>) {
        let memory = self.memory.borrow();

        let mut tile_x = 0;
        let mut tile_y = 0;

        // BG Map Data 1
        for tile_addr in 0x9800..=0x9bff {
            let tile_num = memory.load(tile_addr);

            self.print_tile(
                self.get_tile(tile_num),
                canvas,
                tile_x,
                tile_y
            );

            if tile_x < 31 {
                tile_x += 1;
            } else {
                tile_x = 0;
                tile_y += 1;
            }
        }
        canvas.present();
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

    fn print_tile(&self, tile: [u8; 16], canvas: &mut Canvas<Window>, x: u8, y: u8) {
        assert!(x <= 31);
        assert!(y <= 31);

        for line in 0..=7 {
            let b = (tile[line * 2], tile[1 + line * 2]);

            for row in (0..=7).rev() {
                if (b.0 & (1 << row)).count_ones() == 0 {
                    canvas.set_draw_color(Color::RGB(0xff, 0xff, 0xff));
                } else {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                }

                let xx = x as i32 * 8 + (row as i8 - 7).abs() as i32;
                let yy = y as i32 * 8 + line as i32;
                canvas.draw_point(Point::new(xx, yy)).unwrap();
            }
        }
    }
}

impl<'a> fmt::Display for Gpu {
     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "abc")
     }
}
