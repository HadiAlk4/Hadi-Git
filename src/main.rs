use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;


use flate2::write::ZlibEncoder;
use flate2::Compression;


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
    CatFile{
        #[arg(short = 'p')]
        pretty: bool, // map -p
        object: String, // store SHA-1 String
    },
    WriteTree,
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
            // the first two letters of the hash will become the name of the subfolder and the rest become the name of the actual sub folder 
            use sha1::{Sha1, Digest};
            let mut hasher = Sha1::new();
            hasher.update(&store_data);
            let result = hasher.finalize();
            let hash_string = hex::encode(result);

            let sub_folder_path = format!(".hgit/objects/{}", &hash_string[0..2]);
            let file_path = format!("{}/{}", sub_folder_path, &hash_string[2..]);

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&store_data).expect("Compression Failed");
            let compressed_data = encoder.finish().expect("Finalizing Compression Failed");

            fs::create_dir_all(&sub_folder_path).expect("Failed to Create Object Subfolders");
            fs::write(&file_path, compressed_data).expect("Failed to Write Object Data File");

            println!("SHA-1 Hash:\n{}", hash_string);
            //println!("File Content:\n{}", content);
        }

        Commands::CatFile { pretty, object } => { // locate the file, read compressed bytes, decompress the data, strip the header 
            let sub_folder_path = format!(".hgit/objects/{}", &object[0..2]);
            let file_path = format!("{}/{}", sub_folder_path, &object[2..]);

            use flate2::read::ZlibDecoder;
            use std::io::Read;

            let compressed_data = fs::read(&file_path).unwrap_or_else(|_| panic!("Could not find object: {}", object));
            let mut decoder = ZlibDecoder::new(&compressed_data[..]);
            let mut uncompressed_data = Vec::new();
            decoder.read_to_end(&mut uncompressed_data).expect("Failed to decompress object data");

            if let Some(null_pos) = uncompressed_data.iter().position(|&b| b == 0){
                let content_bytes = &uncompressed_data[null_pos + 1..];

                let content_string = String::from_utf8_lossy(content_bytes);

                if pretty {
                    print!("{}", content_string);
                }
            }
        
        }

        Commands::WriteTree => {
            let mut entries = Vec::new();

            let paths = fs::read_dir(".").expect("failed to read current directory");

            for path_result in paths {
                let entry = path_result.expect("failed to read directory entry");
                let file_name = entry.file_name().into_string().expect("invalid file name");

                if file_name == "target" || file_name.starts_with(".") || file_name == "main" {
                    continue;
                }

                let file_type = entry.file_type().expect("failed to get file type");

                if file_type.is_file() {
                    entries.push(file_name);
                }
            }

            entries.sort();

            let mut tree_body = Vec::new();
            use sha1::{Sha1, Digest};

            for file_name in entries 
            {
                // read file content
                let content = fs::read_to_string(&file_name).unwrap_or_else(|_| panic!("Could not read file: {}", file_name));


                // prepare blob
                let mut store_data = Vec::new();
                use std::io::Write;
                write!(&mut store_data, "blob {}\0{}", content.len(), content).expect("failed to format blob");

                let mut hasher = Sha1::new();
                hasher.update(&store_data);
                let result = hasher.finalize(); // this is a generic 20 byte hash
                let hash_string = hex::encode(result);

                // save blob to disk
                let sub_folder_path = format!(".hgit/objects/{}", &hash_string[0..2]);
                let file_path = format!("{}/{}", sub_folder_path, &hash_string[2..]);

                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&store_data).expect("Compression Failure");
                let compressed_data = encoder.finish().expect("Finalizing Compression");

                fs::create_dir_all(&sub_folder_path).expect("Failed to create object subfolder");
                fs::write(&file_path, compressed_data).expect("Failed to write object data file");

                // append entry to tree
                write!(&mut tree_body, "100644 {}\0", file_name).expect("Failed to write tree entry header");
                tree_body.extend_from_slice(&result);
            }

            // wrap tree in a tree size header

            let mut final_tree_object = Vec::new();
            use std::io::Write;
            write!(&mut final_tree_object, "tree {}\0", tree_body.len()).expect("failed to format tree header");
            final_tree_object.extend_from_slice(&tree_body);

            // hash and store the tree object itself
            let mut tree_hasher = Sha1::new();
            tree_hasher.update(&final_tree_object);
            let tree_hash_result = tree_hasher.finalize();
            let tree_hash_string = hex::encode(tree_hash_result);

            let tree_sub_folder = format!(".hgit/objects/{}", &tree_hash_string[0..2]);
            let tree_file_path = format!("{}/{}", tree_sub_folder, &tree_hash_string[2..]);

            let mut tree_encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            tree_encoder.write_all(&final_tree_object).expect("Compression Failed");
            let compressed_tree = tree_encoder.finish().expect("Finalizing Compression");
            

            fs::create_dir_all(&tree_sub_folder).expect("failed to create tree sunfolder");
            fs::write(&tree_file_path, compressed_tree).expect("Failed to write tree object file");

            println!("{}", tree_hash_string);




        }
    }
}
