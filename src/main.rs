use rusty_nes::{CartLoadError, CPU};

use clap::Parser;

#[derive(Parser)]
struct RustyArgs {
    /// Filename of the ROM
    filename: String,

    /// Whether to disable the debugger mode
    #[arg(short, long, action)]
    nodebug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = RustyArgs::parse();

    let mut cpu = CPU::new(args.filename, !args.nodebug).unwrap_or_else(|err| match err {
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
        cpu.run_opcode();
    }

    // rusty_nes::run();
    Ok(())
}
