use crate::bencode_parser::decode_bencoded_value_with_rest;
use sha1::{Digest, Sha1};

pub struct MetaInfo {
    pub announce: String,
    pub length: i64,
    pub name: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
    pub info_hash: Vec<u8>,
}

impl MetaInfo {
    pub fn from_string(bytes: &[u8]) -> Option<Self> {
        let (obj, rest) = decode_bencoded_value_with_rest(bytes)?;
        if !std::str::from_utf8(rest).ok()?.trim().is_empty() {
            println!("not empty");
            return None;
        }

        let dict = obj.to_map()?;
        let announce = dict.get("announce")?.to_str()?;
        let info = dict.get("info")?.to_map()?;
        let info_obj = dict.get("info")?;
        let length = info.get("length")?.to_i64()?;
        let name = info.get("name")?.to_str()?;
        let piece_length = info.get("piece length")?.to_i64()?;
        let pieces = info.get("pieces")?.to_bytes()?;
        let mut hasher = Sha1::new();
        hasher.update(info_obj.to_bencode());
        let info_hash: Vec<u8> = hasher.finalize().to_vec();

        Some(Self {
            announce: announce.to_string(),
            length: *length,
            name: name.to_string(),
            piece_length: *piece_length,
            pieces: pieces.to_vec(),
            info_hash,
        })
    }
}
