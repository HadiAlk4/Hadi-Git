
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
struct Cli
{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands 
{
    Init,
}

fn main() {

    let cli = Cli::parse();

    match cli.command 
    {
        Commands::Init => 
        {
            println!("test")
        }
    }
}
