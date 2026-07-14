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
            let content = fs::read_to_string(&file).unwrap_or_else(|_| panic!("Could not read file: -{}-", file));
            let mut store_data = Vec::new();
            use std::io::Write;
            write!(&mut store_data, "blob {}\0{}", content.len(), content).expect("failed to format git object"); // formats the git header 

            // hashing - well calculate the SHA-1 hash so that if even a single comma changes well be able to see that theyre different 
            //after feeding it data_store SHA-1 will return a 20 byte sequence of binary data and then well encode them in a 40-character hex string 
            use sha1::{Sha1, Digest};
            let mut hasher = Sha1::new();
            hasher.update(&store_data);
            let result = hasher.finalize();
            let hash_string = hex::encode(result);

            println!("SHA-1 Hash:\n{}", hash_string);
            //println!("File Content:\n{}", content);
        }
    }
}
