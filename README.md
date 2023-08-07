![gameboy-icon](images/gameboy.png)

# gameperson

A Nintendo Gameboy emulator written in Rust.

## Usage

You'll need the [ROM file of a game](https://www.google.com/search?q=game+boy+roms) to run:

```shell
cargo run ROM.gb
```

If you want to use a specific [Gameboy boot rom](https://www.google.com/search?q=game+boy+boot+rom):

```shell
cargo run --boot-rom dmg_boot.bin ROM.gb
```

## Status

- [x] CPU opcodes
- [ ] Interrupts
  - [x] VBlank
  - [ ] LCD STAT
  - [ ] Timer
  - [ ] Serial
  - [x] Joypad
- [ ] Timers
- [ ] APU
- [ ] GPU
  - [x] BG map
  - [x] Sprites
  - [ ] Window

## Screenshots

<img src="images/tetris.png" width="150" />

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[GPLv3](https://www.gnu.org/licenses/gpl-3.0.en.html)
