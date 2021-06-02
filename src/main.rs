#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod cartridge;
mod cpu;
mod gpu;
mod input;
mod memory;

use std::cell::RefCell;
use std::io::stdin;
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
use sdl2::sys::SDL_bool;

fn debug(cpu: &mut cpu::Cpu, mem: &memory::Memory) -> (u16, bool) {
    println!("b HEX - run until - HEX = 0 to reset");
    println!("p HEX - dump memory address");
    println!("j HEX - dump memory address");
    println!("n - next instruction");
    println!("c - continue");

    loop {
        println!();
        println!("{}", cpu);

        let mut s = String::new();

        stdin().read_line(&mut s).expect("Did not enter a correct string");
        let v: Vec<&str> = s.split_whitespace().collect();

        let command = v[0];

        if v.len() == 2 {
            if let Ok(addr) = u16::from_str_radix(v[1], 16) {
                if command == "b" {
                    return (addr, false)
                } else if command == "j" {
                    cpu.pc = addr;
                    return (0, false);
                } else if command == "p" {
                    println!("{:#04x}", mem.load(addr as usize));
                } else {
                    eprintln!("Invalid command");
                }
            }
        } else {
            if command == "c" {
                return (0, false);
            } else if command == "n" {
                return (0, true)
            }
            eprintln!("Invalid command");
        }
    }
}

#[allow(clippy::too_many_lines)]
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

    let mut step = false;

    let mut cycles: u16;
    let mut breakpoint: u16 = 0;

    'running: loop {
        let mut _next = false;
        cycles = 0;

        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    step = true;
                    println!("Break");
                }
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => {
                    let ret = debug(&mut cpu, &mem.borrow());
                    breakpoint = ret.0;
                    step = ret.1;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::N),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::F7),
                    ..
                } => {
                    step = true;
                    println!("Next");
                }
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => {
                    step = false;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::Quit { .. } => break 'running,
                _ => {
                    // println!("{} {}", cpu, gpu.borrow().ly);
                }
            }
        }

        if breakpoint != 0 && cpu.pc == breakpoint {
            println!("Break");
            let ret = debug(&mut cpu, &mem.borrow());
            breakpoint = ret.0;
            step = ret.1;
        }

        for _ in 0..1 {
            cycles += u16::from(cpu.decode());
        }

        match gpu.borrow_mut().display(&mut canvas, &mut texture, &mut gpu_buffer, cycles) {
            Some(gpu::Interrupt::VBlank) => {
                if cpu.interrupts_enabled && mem.borrow().ie & 0x1 != 0 {
                    cpu.vblank_int()
                };
            }
            Some(gpu::Interrupt::Status) => {
                if cpu.interrupts_enabled && mem.borrow().ie & 0x2 != 0 {
                    cpu.status_int();
                }
            }
            _ => (),
        }

        // match Input::new(&mut event_pump) {
        //     Some(Input::Joypad(JoypadButton::Up)) => println!("UP"),
        //     Some(Input::Joypad(JoypadButton::Down)) => println!("Down"),
        //     _ => {}
        // }

        if step {
            println!("{} {}", cpu, gpu.borrow().ly);
            cpu.mem_next();

            let ret = debug(&mut cpu, &mem.borrow());
            breakpoint = ret.0;
            step = ret.1;
        }
    }

    Ok(())
}
