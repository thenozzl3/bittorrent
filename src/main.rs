mod bencode_parser;
mod metainfo;

use std::env;
use std::fs::File;
use std::io::Read;

use crate::bencode_parser::decode_bencoded_value;
use crate::metainfo::MetaInfo;
use bencode_parser::BencodeValue;
use hex::ToHex;

struct BencReader {
    reader: Box<dyn Read>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let mut input = Vec::<u8>::new();
    let mut bencreader: BencReader;
    let mut decoded_value: Option<BencodeValue>;

    match command.as_str() {
        "decode" => {
            if args.len() == 2 {
                bencreader = BencReader {
                    reader: Box::new(std::io::stdin()),
                };
                match bencreader.reader.read_to_end(&mut input) {
                    Ok(n) => println!("read {} bytes", n),
                    Err(err) => println!("error reading {}", err),
                }
                decoded_value = decode_bencoded_value(&input);
            } else {
                let encoded_value = &args[2];
                decoded_value = decode_bencoded_value(encoded_value.as_bytes());
            }

            if let Some(val) = decoded_value {
                println!("{}", val.to_json());
            } else {
                panic!("poop")
            };
        }
        "info" => {
            if args.len() == 2 {
                bencreader = BencReader {
                    reader: Box::new(std::io::stdin()),
                };
            } else {
                bencreader = BencReader {
                    reader: Box::new(File::open(&args[2]).unwrap()),
                };
            }

            if let Ok(_) = bencreader.reader.read_to_end(&mut input) {
                if let Some(meta_info) = MetaInfo::from_string(&input) {
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
                        if byte_count % 20 == 0 {
                            println!("");
                        }
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
