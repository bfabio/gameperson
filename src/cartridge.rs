use std::fmt::Display;
use std::str;
pub struct Cartridge<'a> {
    title: &'a str,
    size: u16,
    cartridge_type: &'a str,
    licensee: &'a str,
    licensee_code: String,
    destination_market: &'a str,
}

impl Display for Cartridge<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Size: {}KiB", self.size)?;
        writeln!(f, "Type: {}", self.cartridge_type)?;
        writeln!(f, "Licensee: {} ({})", self.licensee, self.licensee_code)?;
        writeln!(f, "Destination market: {}", self.destination_market)
    }
}

impl<'a> Cartridge<'a> {
    #[allow(clippy::too_many_lines)]
    pub fn new(rom: &'a [u8]) -> Option<Self> {
        // ROM is smaller than a cartridge header.
        if rom.len() < 0x150 {
            return None;
        }

        let title = str::from_utf8(&rom[0x134..=0x143]).unwrap_or("<invalid data>");

        // TODO CGB Flag

        // 0x14b - Old Licensee Code
        // A value of 0x33 means the New License Code at 0x144-0x145 is used instead.
        let licensee_code: String = format!("{:#02x}", rom[0x14b]);
        let licensee = match rom[0x14b] {
            0x00 => "<none>",
            0x01 => "Nintendo",
            0x08 | 0x38 => "Capcom",
            0x09 => "hot-b",
            0x0a => "jaleco",
            0x0b => "coconuts",
            0x0c => "elite systems",
            0x13 => "electronic arts",
            0x18 => "hudsonsoft",
            0x19 => "itc entertainment",
            0x1a => "yanoman",
            0x1d => "clary",
            0x1f => "virgin",
            0x20 => "KSS",
            0x24 => "pcm complete",
            0x25 => "san-x",
            0x28 => "kotobuki systems",
            0x29 => "seta",
            0x30 => "infogrames",
            0x31 => "nintendo",
            0x32 => "bandai",
            0x33 => {
                let new_licensee = str::from_utf8(&rom[0x144..=0x145]);

                if let Ok(licensee_code) = new_licensee {
                    match licensee_code {
                        "00" => "<none>",
                        "01" => "Nintendo R&D1",
                        "08" => "Capcom",
                        "13" | "69" => "Electronic Arts",
                        "18" => "Hudson Soft",
                        "19" => "B-ai",
                        "20" => "Kss",
                        "22" => "Pow",
                        "24" => "PCM Complete",
                        "25" => "San-x",
                        "28" => "Kemco Japan",
                        "29" => "Seta",
                        "30" => "Viacom",
                        "31" => "Nintendo",
                        "32" => "Bandai",
                        "33" | "93" => "Ocean/Acclaim",
                        "34" | "54" => "Konami",
                        "35" => "Hector",
                        "37" => "Taito",
                        "38" => "Hudson",
                        "39" => "Banpresto",
                        "41" => "Ubi Soft",
                        "42" => "Atlus",
                        "44" => "Malibu",
                        "46" => "Angel",
                        "47" => "Bullet-Proof",
                        "49" => "Irem",
                        "50" => "Absolute",
                        "51" => "Acclaim",
                        "52" => "Activision",
                        "53" => "American Sammy",
                        "55" => "Hitech entertainment",
                        "56" => "LJN",
                        "57" => "Matchbox",
                        "58" => "Mattel",
                        "59" => "Milton Bradley",
                        "60" => "Titus",
                        "61" => "Virgin",
                        "64" => "LucasArts",
                        "67" => "Ocean",
                        "70" => "Infogrames",
                        "71" => "Interplay",
                        "72" => "Broderbund",
                        "73" => "Sculptured",
                        "75" => "Sci",
                        "78" => "THQ",
                        "79" => "Accolade",
                        "80" => "Misawa",
                        "83" => "Lozc",
                        "86" => "Tokuma shoten i*",
                        "87" => "Tsukuda ori*",
                        "91" => "Chunsoft",
                        "92" => "Video system",
                        "95" => "Varie",
                        "96" => "Yonezawa/s'pal",
                        "97" => "Kaneko",
                        "99" => "Pack in soft",
                        "A4" => "Konami (Yu-Gi-Oh!)",
                        _ => "<unknown new licensee code>",
                    }
                } else {
                    "<invalid data>"
                }
            }
            0x34 | 0xa4 => "Konami",
            0x35 => "Hector",
            0x39 | 0x9d | 0xd9 => "Banpresto",
            0x3c => "*entertainment i",
            0x3e => "Gremlin",
            0x41 => "Ubisoft",
            0x42 | 0xeb => "Atlus",
            0x44 | 0x4d => "Malibu",
            0x46 | 0xcf => "Angel",
            0x47 => "Spectrum holoby",
            0x49 => "Irem",
            0x4f => "U.S. gold",
            0x50 => "Absolute",
            0x51 | 0xb0 => "Acclaim",
            0x52 => "Activision",
            0x53 => "American Sammy",
            0x54 => "Gametek",
            0x55 => "Park place",
            0x56 | 0xff => "LJN",
            0x57 => "Matchbox",
            0x59 => "Milton bradley",
            0x5a => "Mindscape",
            0x5b => "Romstar",
            0x5c | 0xd6 => "Naxat soft",
            0x5d => "Tradewest",
            0x60 => "Titus",
            0x4a | 0x61 => "Virgin",
            0x67 => "Ocean",
            0x69 => "Electronic arts",
            0x6e => "Elite systems",
            0x6f => "Electro brain",
            0x70 => "Infogrammes",
            0x71 => "Interplay",
            0x72 => "Br\u{f8}derbund",
            0x73 => "Sculptered soft",
            0x75 => "The sales curve",
            0x78 => "T*hq",
            0x79 => "Accolade",
            0x7a => "Triffix entertainment",
            0x7c => "Microprose",
            0x7f | 0xc2 => "Kemco",
            0x80 => "Misawa entertainment",
            0x83 => "Lozc",
            0x86 | 0xc4 => "Tokuma shoten intermedia",
            0x8b => "Bullet-proof software",
            0x8c => "Vic tokai",
            0x8e => "Ape",
            0x8f => "I'max",
            0x91 => "Chun soft",
            0x92 => "Video system",
            0x93 => "Tsuburava",
            0x95 | 0xe3 => "Varie",
            0x96 => "Yonezawa/s'pal",
            0x97 => "Kaneko",
            0x99 => "Arc",
            0x9a => "Nihon bussan",
            0x9b => "Tecmo",
            0x9c => "Imagineer",
            0x9f => "Nova",
            0xa1 => "Hori electric",
            0xa2 | 0xb2 => "Bandai",
            0xa6 => "Kawada",
            0xa7 => "Takara",
            0xa9 => "Technos japan",
            0xaa => "Broderbund",
            0xac => "Toei animation",
            0xad => "Toho",
            0xaf => "Namco",
            0xb1 => "Ascii or nexoft",
            0xb4 => "Enix",
            0xb6 => "HAL",
            0xb7 => "SNK",
            0xb9 => "Pony canyon",
            0xba => "*culture brain o",
            0xbb => "Sunsoft",
            0xbd => "Sony imagesoft",
            0xbf => "Sammy",
            0xc0 | 0xd0 => "Taito",
            0xc3 => "Squaresoft",
            0xc5 => "Data east",
            0xc6 => "Tonkin house",
            0xc8 => "Koei",
            0xc9 => "Ufl",
            0xca => "Ultra",
            0xcb => "Vap",
            0xcc => "Use",
            0xcd => "Meldac",
            0xce => "*pony canyon or",
            0xd1 => "Sofel",
            0xd2 => "Quest",
            0xd3 => "Sigma enterprises",
            0xd4 => "Ask kodansha",
            0xd7 => "Copya systems",
            0xda => "Tomy",
            0xdb => "Ljn",
            0xdd => "Ncs",
            0xde => "Human",
            0xdf => "Altron",
            0xe0 => "Jaleco",
            0xe1 => "Towachiki",
            0xe2 => "Uutaka",
            0xe5 => "Epoch",
            0xe7 => "Athena",
            0xe8 => "Asmik",
            0xe9 => "Natsume",
            0xea => "King records",
            0xec => "Epic/Sony records",
            0xee => "Igs",
            0xf0 => "A wave",
            0xf3 => "Extreme entertainment",
            _ => "<unknown old licensee code>",
        };

        // TODO SGB Flag

        /* Cartridge type */
        let cartridge_type = match rom[0x147] {
            0x00 => "ROM ONLY",
            0x01 => "MBC1",
            0x02 => "MBC1+RAM",
            0x03 => "MBC1+RAM+BATTERY",
            0x05 => "MBC2",
            0x06 => "MBC2+BATTERY",
            0x08 => "ROM+RAM",
            0x09 => "ROM+RAM+BATTERY",
            0x0B => "MMM01",
            0x0C => "MMM01+RAM",
            0x0D => "MMM01+RAM+BATTERY",
            0x0F => "MBC3+TIMER+BATTERY",
            0x10 => "MBC3+TIMER+RAM+BATTERY",
            0x11 => "MBC3",
            0x12 => "MBC3+RAM",
            0x13 => "MBC3+RAM+BATTERY",
            0x15 => "MBC4",
            0x16 => "MBC4+RAM",
            0x17 => "MBC4+RAM+BATTERY",
            0x19 => "MBC5",

            0x1A => "MBC5+RAM",
            0x1B => "MBC5+RAM+BATTERY",
            0x1C => "MBC5+RUMBLE",
            0x1D => "MBC5+RUMBLE+RAM",
            0x1E => "MBC5+RUMBLE+RAM+BATTERY",
            0xFC => "POCKET CAMERA",
            0xFD => "BANDAI TAMA5",
            0xFE => "HuC3",
            0xFF => "HuC1+RAM+BATTERY",
            _ => "<unknown cartridge type>",
        };

        // let size = 32 + rom[0x148] as u16 * 2;
        let size = 32_u16 << rom[0x148];

        let destination_market = if rom[0x14a] == 0x00 { "JP" } else { "World" };

        Some(Self {
            title,
            size,
            cartridge_type,
            licensee,
            licensee_code,
            destination_market,
        })
}
}
