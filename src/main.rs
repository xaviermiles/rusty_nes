use rusty_nes::CPU;

use clap::Parser;

#[derive(Parser)]
struct RustyArgs {
    /// Filename of the ROM
    filename: String,
    // TODO: Argument to control logging level?
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = RustyArgs::parse();

    let mut cpu = CPU::new(args.filename);
    for _ in 1..100 {
        cpu.print_state();
        cpu.run_opcode();
    }

    // rusty_nes::run();
    Ok(())
}
