#![warn(
 clippy::all,
 clippy::pedantic,
 clippy::nursery,
 clippy::cargo,
)]

mod cpu;
mod gpu;
mod input;
mod memory;

use std::env;
use std::error;
use std::fs;
use std::io;
use std::io::{Write, stdin, stdout};
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::str;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::surface::Surface;

use termion::async_stdin;
use termion::clear;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::event;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::style;

use rand::prelude::*;

use memory::Rom;
use memory::Vram;
use memory::IORegisters;

use input::{Input,JoypadButton};

enum Cartridge {
    Type = 0x147,
}

fn cartridge_info(path: &str) -> Result<(), io::Error> {
    let rom = match fs::read(path) {
        Ok(rom) => rom,
        Err(e) => return Err(e),
    };

    println!("Title: {:?}", String::from_utf8(rom[0x134..0x143].to_vec()));


    // TODO CGB Flag
    // TODO Licensee Code
    // TODO SGB Flag

    println!("{:?}", String::from_utf8(rom[0x134..0x143].to_vec()));

    /* Cartridge type */
    match rom[0x147] {
        0x00 => println!("ROM ONLY"),
        0x01 => println!("MBC1"),
        0x02 => println!("MBC1+RAM"),
        0x03 => println!("MBC1+RAM+BATTERY"),
        0x05 => println!("MBC2"),
        0x06 => println!("MBC2+BATTERY"),
        0x08 => println!("ROM+RAM"),
        0x09 => println!("ROM+RAM+BATTERY"),
        0x0B => println!("MMM01"),
        0x0C => println!("MMM01+RAM"),
        0x0D => println!("MMM01+RAM+BATTERY"),
        0x0F => println!("MBC3+TIMER+BATTERY"),
        0x10 => println!("MBC3+TIMER+RAM+BATTERY"),
        0x11 => println!("MBC3"),
        0x12 => println!("MBC3+RAM"),
        0x13 => println!("MBC3+RAM+BATTERY"),
        0x15 => println!("MBC4"),
        0x16 => println!("MBC4+RAM"),
        0x17 => println!("MBC4+RAM+BATTERY"),
        0x19 => println!("MBC5"),
        0x1A => println!("MBC5+RAM"),
        0x1B => println!("MBC5+RAM+BATTERY"),
        0x1C => println!("MBC5+RUMBLE"),
        0x1D => println!("MBC5+RUMBLE+RAM"),
        0x1E => println!("MBC5+RUMBLE+RAM+BATTERY"),
        0xFC => println!("POCKET CAMERA"),
        0xFD => println!("BANDAI TAMA5"),
        0xFE => println!("HuC3"),
        0xFF => println!("HuC1+RAM+BATTERY"),
        _ => println!("Unknown"),
    }

    println!("Rom size: {} KBytes", 32 + rom[0x148] * 2);

    println!("Destination: {}", if rom[0x14a] == 0x00 { "JP" } else { "World" });
    println!("Old licensee: {:x}", rom[0x14b]);

    Ok(())
}

fn main() -> Result<(), Box<error::Error>> {
    let boot_rom_path = env::args().nth(1).expect("Boot ROM required");
    let rom_path = env::args().nth(2).expect("ROM required");

    let boot_rom = fs::read(&boot_rom_path)?;
    let rom = fs::read(&rom_path)?;

    let mut mem = memory::Memory::new();

    mem.map(0x0000, Box::new(Rom::new(boot_rom)));

    mem.map(0x0000, Box::new(Rom::new(rom)));

    // Video RAM
    mem.map(0x8000, Box::new(Vram::new()));

    let mem = Rc::new(RefCell::new(mem));

    let mut cpu = cpu::Cpu::new(Rc::clone(&mem));

    let gpu = gpu::Gpu::new(Rc::clone(&mem));
    let mut gpu_buffer = gpu::GpuBuffer::new();

    let gpu = Rc::new(RefCell::new(gpu));
    let io_registers = IORegisters::new(Rc::clone(&gpu));
    let b = Box::new(io_registers);

    // I/O Registers
    mem.borrow_mut().map(0xff00, b);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    /*
    let window = video_subsystem.window("gb", 160, 144)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGBA8888, 256, 256)?;

    canvas.clear();

    canvas.present();
    */

    let mut event_pump = sdl_context.event_pump()?;

    let stdinput = async_stdin();
    let mut stdout = stdout().into_raw_mode()?;

    let (_, height) = termion::terminal_size()?;
    write!(stdout,
           "{}{}",
           color::Bg(color::White),
           termion::clear::All);
    write!(stdout,
           "{}{}{}{}(q) quit - (b) break - (s) step - (c) continue",
           cursor::Goto(1, height),
           color::Bg(color::Blue),
           color::Fg(color::White),
           clear::CurrentLine);

    stdout.flush()?;

    let mut step = false;
    let mut events = stdinput.events();
    loop {

        if let Some(result) = events.next() {
            let event = result.unwrap();
            match event {
                event::Event::Key(Key::Char('q')) => break,
                event::Event::Key(Key::Char('b')) => step = true,
                _ => (),
            }
        }

        if step {
            let stdinput = stdin();
            for c in stdinput.keys() {
                match c.unwrap() {
                    Key::Char('s') => break,
                    Key::Char('c') => {
                        step = false;
                        break;
                    }
                    _ => (),
                }
            }
        }
        for _ in 1..50 {
            cpu.decode();
        }

        // show_box(&mut stdout, "Registers", &cpu, 1)?;
        // stdout.flush()?;

        // gpu.borrow_mut().display(&mut canvas, &mut texture, &mut gpu_buffer, &mut stdout);
        gpu.borrow_mut().display(&mut gpu_buffer, &mut stdout);

        match Input::new(&mut event_pump) {
            Some(Input::Joypad(JoypadButton::Up)) => println!("UP"),
            Some(Input::Joypad(JoypadButton::Down)) => println!("Down"),
            _ => {},
        }
    }

    Ok(())
}

fn show_box<W: Write, D: Display>(
    screen: &mut W,
    title: &str,
    content: D,
    y: u16,
) -> Result<(), Box<dyn error::Error>> {

    let (width, _) = termion::terminal_size()?;

    let fill: usize = width as usize - title.len()
        - 9;

    write!(screen,
           "{}{}{}┌───┤ {} ├{}┐",
           color::Fg(color::White),
           color::Bg(color::Blue),
           termion::cursor::Goto(1, y),
           title,
           "─".repeat(fill)).unwrap();

    write!(screen,
           "{}{}│ {}",
           cursor::Goto(1, 2),
           clear::CurrentLine,
           content);
    write!(screen, "{}│", termion::cursor::Goto(width, y));

    write!(screen,
           "{}{}└{}┘",
           cursor::Goto(1, 3),
           clear::CurrentLine,
           "─".repeat(width as usize - 2));

    Ok(())
}
