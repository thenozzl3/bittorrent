mod bencode_parser;
mod metainfo;

use std::env;

use crate::bencode_parser::decode_bencoded_value;
use crate::metainfo::MetaInfo;
use hex::ToHex;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value.as_bytes());

            if let Some(val) = decoded_value {
                println!("{}", val.to_json());
            } else {
                panic!("poop")
            }
        }
        "info" => {
            if let Ok(contents) = std::fs::read(&args[2]) {
                if let Some(meta_info) = MetaInfo::from_string(&contents) {
                    println!(
                        "Info Hash: {}\nTracker URL: {}\nLength: {}\n",
                        meta_info.info_hash.encode_hex::<String>(),
                        meta_info.announce,
                        meta_info.length,
                    );
                    println!("pieces: ... \n");
                    let mut byte_count = 0;
                    for piece in meta_info.pieces.iter() {
                        byte_count += 1;
                        print!("{:02x}", piece);
                        if byte_count % 20 == 0 {println!("");}
                    }
                } else {
                    panic!("bad file 1");
                }
            } else {
            panic!("file not found");
            }
        }
        _ => {
            println!("unknown command");
        }
    }
}

