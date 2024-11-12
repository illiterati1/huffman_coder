use std::fs;
use std::path::PathBuf;
use clap::{Parser, ArgGroup};
use huffman_tree::{decode, encode};

mod huffman_tree;

fn main() {
    let args = Args::parse();
    if args.encode {
        let mut input_file = fs::File::open(args.input)
            .expect("Failed to open input file");
        let mut output_file = fs::File::create(&args.output)
            .expect("Failed to open output file");
        encode(&mut input_file, &mut output_file);
    } else if args.decode {
        let mut input_file = fs::File::open(args.input)
            .expect("Failed to open input file");
        let mut output_file = fs::File::create(&args.output)
            .expect("Failed to open output file");
        decode(&mut input_file, &mut output_file);
    }
}

// Huffman encoder/decoder
#[derive(Parser)]
#[clap(about)]
#[clap(group(
    ArgGroup::new("coding")
    .required(true)
    .args(&["encode", "decode"]),
))]
struct Args {
    // The input file
    input: PathBuf,
    // The destination file
    output: PathBuf,
    // Encode file
    #[clap(short, conflicts_with = "decode")]
    encode: bool,
    // Decode file
    #[clap(short, conflicts_with = "encode")]
    decode: bool,
}
