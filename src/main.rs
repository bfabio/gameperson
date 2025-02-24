#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

mod cartridge;
mod cpu;
mod gpu;
mod input;
mod memory;

use std::cell::RefCell;
use std::io::stdin;

use std::error;
use std::fs;
use std::rc::Rc;

use clap::Parser;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use sdl2::pixels::PixelFormatEnum;

use cartridge::Cartridge;
use memory::Rom;

use input::{Input, JoypadButton};

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

        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        let v: Vec<&str> = s.split_whitespace().collect();

        let command = v[0];

        if v.len() == 2 {
            if let Ok(addr) = u16::from_str_radix(v[1], 16) {
                if command == "b" {
                    return (addr, false);
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
                return (0, true);
            }
            eprintln!("Invalid command");
        }
    }
}

#[derive(Parser)]
struct Args {
    rom: String,
    #[arg(short, long)]
    boot_rom: Option<String>,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn error::Error>> {
    let args = Args::parse();

    let rom = fs::read(&args.rom)?;

    if let Some(cartridge) = Cartridge::new(&rom) {
        println!("Cartridge info\n{}", &cartridge);
    } else {
        eprintln!("Can't parse cartridge header");
    }

    let mut mem = memory::Memory::new(gpu::Gpu::new());

    if let Some(boot_rom) = &args.boot_rom {
        let boot_rom = fs::read(&boot_rom)?;
        mem.map(0x0000, Box::new(Rom::new(boot_rom)));
    }

    mem.map(0x0000, Box::new(Rom::new(rom)));

    let mem = Rc::new(RefCell::new(mem));

    let mut cpu = if args.boot_rom.is_some() {
        cpu::Cpu::new(Rc::clone(&mem))
    } else {
        cpu::Cpu::new_initialized(Rc::clone(&mem))
    };

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("", 160 * 2, 144 * 2)
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

        while cycles < 200 {
            cycles += u16::from(cpu.decode());
        }

        let ie = mem.borrow().ie;

        let int = mem.borrow_mut().display(&mut canvas, &mut texture, cycles);

        match int {
            Some(gpu::Interrupt::VBlank) => {
                if cpu.interrupts_enabled && mem.borrow().ie & 0x1 != 0 {
                    cpu.vblank_int()
                };
            }
            Some(gpu::Interrupt::Status) => {
                if cpu.interrupts_enabled && ie & 0x2 != 0 {
                    cpu.status_int();
                }
            }
            _ => (),
        }

        // Bit 3 - P13 Input: Down  or Start    (0=Pressed) (Read Only)
        // Bit 2 - P12 Input: Up    or Select   (0=Pressed) (Read Only)
        // Bit 1 - P11 Input: Left  or B        (0=Pressed) (Read Only)
        // Bit 0 - P10 Input: Right or A        (0=Pressed) (Read Only)

        match Input::new(&mut event_pump) {
            Some(Input::JoypadPress(JoypadButton::Up)) => {
                println!("UP");
                mem.borrow_mut().set_joy_state(0, 0b0100)
            }
            Some(Input::JoypadPress(JoypadButton::Down)) => {
                println!("Down");
                mem.borrow_mut().set_joy_state(0, 0b1000)
            }
            Some(Input::JoypadPress(JoypadButton::Left)) => {
                println!("Left");
                mem.borrow_mut().set_joy_state(0, 0b0010)
            }
            Some(Input::JoypadPress(JoypadButton::Right)) => {
                println!("Right");
                mem.borrow_mut().set_joy_state(0, 0b0001)
            }
            Some(Input::JoypadPress(JoypadButton::Start)) => {
                println!("Start");
                mem.borrow_mut().set_joy_state(0b1000, 0)
            }
            Some(Input::JoypadPress(JoypadButton::Select)) => {
                println!("Select");
                mem.borrow_mut().set_joy_state(0b0100, 0)
            }
            Some(Input::JoypadPress(JoypadButton::A)) => {
                println!("A");
                mem.borrow_mut().set_joy_state(0b0001, 0)
            }
            Some(Input::JoypadPress(JoypadButton::B)) => {
                println!("B");
                mem.borrow_mut().set_joy_state(0b0010, 0)
            }

            Some(Input::JoypadRelease(JoypadButton::Up)) => {
                println!("UP");
                mem.borrow_mut().unset_joy_state(0, 0b0100)
            }
            Some(Input::JoypadRelease(JoypadButton::Down)) => {
                println!("Down");
                mem.borrow_mut().unset_joy_state(0, 0b1000)
            }
            Some(Input::JoypadRelease(JoypadButton::Left)) => {
                println!("Left");
                mem.borrow_mut().unset_joy_state(0, 0b0010)
            }
            Some(Input::JoypadRelease(JoypadButton::Right)) => {
                println!("Right");
                mem.borrow_mut().unset_joy_state(0, 0b0001)
            }
            Some(Input::JoypadRelease(JoypadButton::Start)) => {
                println!("Start");
                mem.borrow_mut().unset_joy_state(0b1000, 0)
            }
            Some(Input::JoypadRelease(JoypadButton::Select)) => {
                println!("Select");
                mem.borrow_mut().unset_joy_state(0b0100, 0)
            }
            Some(Input::JoypadRelease(JoypadButton::A)) => {
                println!("A");
                mem.borrow_mut().unset_joy_state(0b0001, 0)
            }
            Some(Input::JoypadRelease(JoypadButton::B)) => {
                println!("B");
                mem.borrow_mut().unset_joy_state(0b0010, 0)
            }

            _ => {}
        }

        if step {
            cpu.mem_next();

            let ret = debug(&mut cpu, &mem.borrow());
            breakpoint = ret.0;
            step = ret.1;
        }
    }

    Ok(())
}
