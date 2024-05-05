mod bencode_parser;
mod metainfo;

use std::env;

use crate::bencode_parser::decode_bencoded_value;
use crate::metainfo::MetaInfo;
use hex::ToHex;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    args.push("info".to_string());
    args.push("sample.torrent".to_string());
    let command = &args[1];
    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value.as_bytes());

        if let Some(val) = decoded_value {
            println!("{}", val.to_json());
        } else {
            panic!("poop")
        }
    } else if command == "info" {
        if let Ok(contents) = std::fs::read(&args[2]) {
            if let Some(meta_info) = MetaInfo::from_string(&contents) {
                println!(
                    "Info Hash: {}\nTracker URL: {}\nLength: {}\n",
                    meta_info.info_hash.encode_hex::<String>(),
                    meta_info.announce,
                    meta_info.length,
                );
            } else {
                panic!("bad file 1");
            }
        } else {
            panic!("file not found");
        }
    } else {
        println!("unknown command");
    }
}
