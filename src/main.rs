#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod cartridge;
mod cpu;
mod gpu;
mod input;
mod memory;

use std::cell::RefCell;
use std::env;
use std::error;
use std::fs;
use std::rc::Rc;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;

use cartridge::Cartridge;
use memory::IORegisters;
use memory::Rom;
use memory::Vram;

use input::{Input, JoypadButton};

fn main() -> Result<(), Box<dyn error::Error>> {
    let boot_rom_path = env::args().nth(1).expect("Boot ROM required");
    let rom_path = env::args().nth(2).expect("ROM required");

    let boot_rom = fs::read(&boot_rom_path)?;
    let rom = fs::read(&rom_path)?;

    if let Some(cartridge) = Cartridge::new(&rom) {
        println!("Cartridge info\n{}", &cartridge);
    } else {
        eprintln!("Can't parse cartridge header");
    }

    let mut mem = memory::Memory::new();

    mem.map(0x0000, Box::new(Rom::new(boot_rom)));

    mem.map(0x0000, Box::new(Rom::new(rom)));

    // Video RAM
    mem.map(0x8000, Box::new(Vram::new()));

    let mem = Rc::new(RefCell::new(mem));

    let mut cpu = cpu::Cpu::new(Rc::clone(&mem));

    let gpu = gpu::Gpu::new(Rc::clone(&mem));
    let mut gpu_buffer = gpu::Buffer::new();

    let gpu = Rc::new(RefCell::new(gpu));
    let io_registers = IORegisters::new(Rc::clone(&gpu));
    let b = Box::new(io_registers);

    // I/O Registers
    mem.borrow_mut().map(0xff00, b);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("gb", 160, 144)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGBA8888, 256, 256)?;

    canvas.clear();

    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    let mut _step = false;
    'running: loop {
        let mut _next = false;

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        for _ in 1..50 {
            // if ! step || next {
            cpu.decode();
            //}
        }

        gpu.borrow_mut()
            .display(&mut canvas, &mut texture, &mut gpu_buffer);

        match Input::new(&mut event_pump) {
            Some(Input::Joypad(JoypadButton::Up)) => println!("UP"),
            Some(Input::Joypad(JoypadButton::Down)) => println!("Down"),
            _ => {}
        }
    }

    Ok(())
}
