use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode
//
//

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) {
    // If encoded_value starts with a digit, it's a number
    let mut current_chunk: String = String::new();
    let mut current_pos = 0;
    let mut current_chunk_pos = 0;

    let mut enc_iter = encoded_value.chars().peekable();

    //enter the state machine
    while let Some(current_char) = enc_iter.next() {
        if current_char.is_ascii_digit() {
            //figure out how far we have to go ..
            current_chunk.push_str(&current_char.to_string());
            while let Some(stuff) = enc_iter.peek() {
                current_chunk_pos += 1;
                if *stuff == ':' {
                    break;
                }
                current_chunk.push_str(&*stuff.to_string());
                enc_iter.next();
            }

            let string_length:i64 = current_chunk.parse::<i64>().unwrap().into();
            enc_iter.next();
            current_pos += current_chunk_pos + 1 ;
            print!("\"{}\"", &encoded_value[current_pos..(current_pos + string_length as usize)]);
            current_pos += string_length  as usize  ;
            //advance the iterator the length of the string ..
            enc_iter.nth(string_length as usize -1);
            current_chunk.clear();

        } else {
            match current_char {
                'l' => {
                    print!("[");
                    current_pos += 1;
                   // current_chunk_pos = 0;
                    continue;
                }
                'e' => {
                    print!("]");
                    current_pos += 1;
                }
                'i' => {
                    while let Some(stuff) = enc_iter.peek() {
                        current_pos += 1;
                        if *stuff == 'e' {
                            print!("{}",current_chunk);
                            current_chunk.clear();
                            enc_iter.next();
                            current_chunk_pos += 1;
                            break;
                        }
                        current_chunk.push_str(&*stuff.to_string());
                        enc_iter.next();
                    }
                    current_pos += current_chunk_pos ;
                }
                _ => (),
            }
        }
        current_chunk_pos = 0;
        if let Some(stuff) = enc_iter.peek(){
            if *stuff != 'e' {
              print!(",");
            }
        }
    }
    println!();
    ()
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        //println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        //let decoded_value = decode_bencoded_value(encoded_value);
        decode_bencoded_value(encoded_value);
    } else {
        println!("unknown command: {}", args[1]);
    }
}
