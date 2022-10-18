use rusty_nes::{self, CartLoadStatus};

use clap::Parser;

#[derive(Parser, Debug)]
struct RustyArgs {
    /// Filename of the ROM
    filename: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = RustyArgs::parse();

    let cart = rusty_nes::load_to_cart(args.filename);
    match cart {
        CartLoadStatus::Success(_) => print!("yay"),
        CartLoadStatus::FileNotARom => {
            eprint!("Not a valid ROM file.")
        }
        CartLoadStatus::FileNotFound => {
            eprint!("ROM file not found.")
        }
    }

    // rusty_nes::run();
    Ok(())
}
