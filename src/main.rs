use ::gcc::{gcc, Config};

fn main() -> anyhow::Result<()> {
    let config = Config::default();
    let mut component = gcc::new(config);
    component.initialize().map_err(|e| anyhow::anyhow!("{}", e))?;
    component.run().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("SigmaOS-gcc operational");
    Ok(())
}
