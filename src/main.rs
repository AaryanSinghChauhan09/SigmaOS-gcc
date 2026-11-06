//! SigmaOS-gcc — compiler driver binary.

use gcc::{Driver, DriverError};

fn main() -> Result<(), DriverError> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    if argv.is_empty() {
        println!("usage: gcc [options] input...");
        println!("       gcc --version");
        return Ok(());
    }
    if argv.iter().any(|a| a == "--version" || a == "-dumpversion") {
        println!("gcc (SigmaOS) 1.0.0");
        return Ok(());
    }
    if argv.iter().any(|a| a == "--help" || a == "-v") {
        println!("gcc driver — options: -O0..-O3 -g -c -E -s -o OUT --target TRIPLE");
        return Ok(());
    }
    let artifacts = Driver::compile(&argv)?;
    for a in &artifacts {
        println!("{}: {} bytes", a.name, a.data.len());
    }
    Ok(())
}
