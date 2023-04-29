use rusty_nes::CPU;

use clap::Parser;

#[derive(Parser)]
struct RustyArgs {
    /// Filename of the ROM
    filename: String,
    // TODO: Argument to control logging level?
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().init().unwrap();

    let args = RustyArgs::parse();

    let mut cpu = CPU::new(args.filename);
    for _ in 1..20 {
        cpu.run_opcode();
    }

    // rusty_nes::run();
    Ok(())
}
