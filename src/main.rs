use std::{env, fs, io::Read, process::exit};

use huffman_tree::{decode, encode};

mod huffman_tree;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];
    let mut file = match fs::File::open(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open file: {e}");
            exit(-1);
        }
    };
    let mut contents = String::new();
    let _ = file.read_to_string(&mut contents);
    let mut output_file = fs::File::create("./encoded").unwrap();
    encode(&mut output_file, contents);
    drop(output_file);

    let mut encoded = fs::File::open("./encoded").unwrap();
    let mut decoded = fs::File::create("./decoded").unwrap();
    decode(&mut encoded, &mut decoded);
}

