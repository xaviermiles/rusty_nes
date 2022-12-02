use rusty_nes::{System, CPU};

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

    let mut system = System::new(args.filename);
    let mut cpu = CPU::new(&mut system);
    for _ in 1..100 {
        cpu.print_state();
        cpu.run_opcode();
    }

    // rusty_nes::run();
    Ok(())
}
