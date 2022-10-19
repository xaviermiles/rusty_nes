use rusty_nes::{System, CPU};

use clap::Parser;

#[derive(Parser, Debug)]
struct RustyArgs {
    /// Filename of the ROM
    filename: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = RustyArgs::parse();

    let system = System::new(args.filename);
    let cpu = CPU::new(&system);

    // rusty_nes::run();
    Ok(())
}
