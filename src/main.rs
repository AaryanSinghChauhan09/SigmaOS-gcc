use anyhow::Result;
use clap::Parser;
use gcc::gcc;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    auto: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = gcc::Config { auto_start: cli.auto, ..Default::default() };
    let mut component = gcc::new(config);
    
    component.initialize()?;
    component.run()?;
    
    println!("SigmaOS-gcc operational");
    Ok(())
}
