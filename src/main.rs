use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "hgit", version, about)]
struct Cli
{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands 
{
    Init,
    HashObject{
        file: String,
    },
}

fn main() {

    let cli = Cli::parse();

    match cli.command 
    {
        Commands::Init => 
        {
            if Path::new(".hgit").exists()
            {
                println!("Repository Already Exists!");
                return;
            }

            fs::create_dir_all(".hgit/objects").expect("did not create main hgit folder");
            fs::create_dir_all(".hgit/refs/heads").expect("did not create sub hgit folders");
        }

        Commands::HashObject { file } => {
            println!("Target File: {}", file);
        }
    }
}
