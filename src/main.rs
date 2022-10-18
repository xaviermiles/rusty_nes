use rusty_nes::System;

use clap::Parser;

#[derive(Parser, Debug)]
struct RustyArgs {
    /// Filename of the ROM
    filename: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = RustyArgs::parse();

    let system = System::new(args.filename);

    // rusty_nes::run();
    Ok(())
}
