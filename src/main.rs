use serde_bencode::to_string;
use serde_json::{Result,Value};
use std::env;
use std::fmt::Write;

// Available if you need it!
// use serde_bencode
//
//
//

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &[u8], json_string: &mut String) {
    // If encoded_value starts with a digit, it's a number
    let mut current_chunk: String = String::new();
    let mut current_pos = 0;
    let mut current_chunk_pos = 0;
    let mut braces: Vec<char> = vec![];
    let mut enc_iter = encoded_value.iter().peekable();

    //enter the state machine
    while let Some(current_byte) = enc_iter.next() {
        if current_byte.is_ascii_digit() {
            //figure out how far we have to go ..
            // bytes and shit to deal with ..
            current_chunk.push_str((*current_byte as char).to_string().as_str());
            while let Some(stuff) = enc_iter.peek() {
                current_chunk_pos += 1;
                if **stuff == b':' {
                    break;
                }
                current_chunk.push_str((**stuff as char).to_string().as_str());
                enc_iter.next();
            }

            let string_length: i64 = current_chunk.parse::<i64>().unwrap().into();
            enc_iter.next();
            current_pos += current_chunk_pos + 1;
            // these can be non utf-8 grabagio .. handle dat shyet..
            if let Ok(string) = String::from_utf8(
                encoded_value[current_pos..(current_pos + string_length as usize)].to_vec(),
            ) {
                write!(json_string, "\"{}\"", string);
            } else {
                let bytes = format!(
                    "\"{:02x?}\"",
                    encoded_value[current_pos..(current_pos + string_length as usize)].to_owned()
                );
                write!(json_string, "{}", bytes);
            }
            current_pos += string_length as usize;
            //advance the iterator the length of the string ..
            enc_iter.nth(string_length as usize - 1);
            current_chunk.clear();
        } else {
            match *current_byte {
                b'l' => {
                    braces.push('[');
                    write!(json_string, "[");
                    current_pos += 1;
                    // current_chunk_pos = 0;
                    continue;
                }
                b'd' => {
                    braces.push('{');
                    write!(json_string, "{{");
                    current_pos += 1;
                    // current_chunk_pos = 0;
                    continue;
                }
                b'e' => {
                    //print the closing brace - depending
                    //which one we saw last
                    if let Some(brace) = braces.pop() {
                        match brace {
                            '[' => write!(json_string, "]").unwrap(),
                            '{' => write!(json_string, "}}").unwrap(),
                            ':' => write!(json_string, "}}").unwrap(),
                            _ => (),
                        }
                    }
                    current_pos += 1;
                }
                b'i' => {
                    while let Some(stuff) = enc_iter.peek() {
                        current_pos += 1;
                        if **stuff == b'e' {
                            write!(json_string, "{}", current_chunk);
                            current_chunk.clear();
                            enc_iter.next();
                            current_chunk_pos += 1;
                            break;
                        }
                        current_chunk.push_str((**stuff as char).to_string().as_str());
                        enc_iter.next();
                    }
                    current_pos += current_chunk_pos;
                }
                _ => (),
            }
        }
        current_chunk_pos = 0;
        //seperators,ending braces etc
        if let Some(stuff) = enc_iter.peek() {
            if **stuff != b'e' {
                if let Some(brace) = braces.last() {
                    match brace {
                        '{' => {
                            write!(json_string, ":");
                            braces.push(':');
                        }
                        ':' => {
                            _ = braces.pop();
                            write!(json_string, ",");
                        }
                        _ => {
                            write!(json_string, ",");
                        }
                    }
                }
            }
        }
    }
    //println!();
    ()
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let mut json_string: String = String::new();

    match &*command.to_string() {
        "decode" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            //println!("Logs from your program will appear here!");

            // Uncomment this block to pass the first stage
            let encoded_value = args[2].as_bytes();
            decode_bencoded_value(encoded_value, &mut json_string);
            //json_string.remove(0);
            println!("{}", json_string);
        }
        "info" => {
            if let Ok(content) = std::fs::read(&args[2]){
              decode_bencoded_value(&content, &mut json_string);
              //println!("{}", json_string);
              let metadata : Value = serde_json::from_str(&json_string).unwrap();
              println!("Tracker URL: {}", metadata["announce"].as_str().unwrap());
              println!("Length: {}", metadata["info"]["length"]);
            }
        }
        _ => println!("unknown command: {}", args[1]),
    }
}
