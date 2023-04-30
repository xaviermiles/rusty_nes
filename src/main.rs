use rusty_nes::{CartLoadError, CPU};

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

    let mut cpu = CPU::new(args.filename).unwrap_or_else(|err| match err {
        CartLoadError::FileNotARom => {
            panic!("Not a valid ROM file.")
        }
        CartLoadError::FileNotFound => {
            panic!("ROM file not found.")
        }
        CartLoadError::IoError(io_err) => {
            panic!("IO Error: {}", io_err);
        }
    });
    for _ in 1..100 {
        cpu.print_state();
        cpu.run_opcode();
    }

    // rusty_nes::run();
    Ok(())
}
