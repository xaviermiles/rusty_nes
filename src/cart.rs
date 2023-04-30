use std::{
    fmt::Debug,
    fs::File,
    io::{BufReader, Read},
};

pub enum CartLoadError {
    FileNotARom,
    FileNotFound,
    IoError(std::io::Error),
}

pub type CartLoadResult<T> = Result<T, CartLoadError>;

#[allow(dead_code)]
pub struct Cart {
    prg_rom: usize,
    chr_rom: usize,
    mirroring: Mirroring,

    // Currently unused:
    battery_present: bool,
    trainer_present: bool,
    hard_wired_four_screen_mode: bool,

    mapper: u8,
    pub prg_rom_pages: Vec<Vec<u8>>,
    pub chr_rom_pages: Vec<Vec<u8>>,
}

impl Debug for Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cart")
            .field("prg_rom", &self.prg_rom)
            .field("chr_rom", &self.chr_rom)
            .field("mirroring", &self.mirroring)
            .field("mapper", &self.mapper)
            .finish()
    }
}

#[derive(Debug)]
pub enum Mirroring {
    HorizontalOrMapperControlled,
    Vertical,
}

/// Load contents of file to Cart
pub fn load_to_cart(filename: String) -> CartLoadResult<Cart> {
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(_) => {
            return Err(CartLoadError::FileNotFound);
        }
    };
    let mut buf_reader = BufReader::new(file);
    let mut contents: Vec<u8> = Vec::new();
    if let Err(err) = buf_reader.read_to_end(&mut contents) {
        return Err(CartLoadError::IoError(err));
    }

    // Check that this is a valid ROM file
    if &contents[0..3] != b"NES" || contents[3] != 0x1a {
        return Err(CartLoadError::FileNotARom);
    }

    let prg_rom = contents[4] as usize;
    let chr_rom = contents[5] as usize;
    let mirroring = match (contents[6]) & 0x1 {
        0 => Mirroring::HorizontalOrMapperControlled,
        1 => Mirroring::Vertical,
        _ => Mirroring::HorizontalOrMapperControlled, // TODO: should this be necessary?
    };
    let battery_present = contents[6] & 0x2 == 0x2;
    let trainer_present = contents[6] & 0x3 == 0x3;
    let hard_wired_four_screen_mode = contents[6] & 0x4 == 0x4;

    let mut mapper = contents[6] >> 4;
    mapper += contents[7] & 0xf0;

    // TODO: convert prg_rom_pages/chr_rom_pages for-loops
    let prg_rom_page_size = 16 * 1024;
    let mut prg_rom_pages = Vec::new();
    let mut current_start = 16;
    for page in 0..prg_rom {
        let mut current_page = Vec::new();
        for offset in 0..prg_rom_page_size {
            current_page.push(contents[current_start + page * prg_rom_page_size + offset]);
        }
        prg_rom_pages.push(current_page);
    }

    let chr_rom_page_size = 8 * 1024;
    let mut chr_rom_pages = Vec::new();
    current_start += prg_rom_page_size * prg_rom;
    for page in 0..chr_rom {
        let mut current_page = Vec::new();
        for offset in 0..chr_rom_page_size {
            current_page.push(contents[current_start + page * chr_rom_page_size + offset]);
        }
        chr_rom_pages.push(current_page);
    }

    Ok(Cart {
        prg_rom,
        chr_rom,
        mirroring,
        battery_present,
        trainer_present,
        hard_wired_four_screen_mode,
        mapper,
        prg_rom_pages,
        chr_rom_pages,
    })
}
