use std::{collections::BTreeMap, iter,str::from_utf8};

const BENCODE_END_DELIMITER: u8 = 101; //e
const BENCODE_INTEGER_START: u8 = 105; //i
const BENCODE_LIST_START: u8 = 108; //i
const BENCODE_DICTIONARY_START: u8 = 100; //i
const BENCODE_STRING_SEPERATOR: u8 = 58; //i

#[derive(PartialEq, Debug)]
enum BencodeType {
    String,
    Integer,
    List,
    Dictionary,
}

#[derive(PartialEq, Debug)]
pub enum BencodeValue<'a> {
    String(&'a str),
    Bytes(&'a [u8]),
    Integer(i64),
    List(Vec<BencodeValue<'a>>),
    Dictionary(BTreeMap<&'a str, BencodeValue<'a>>),
}

impl<'a> BencodeValue<'a> {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::String(string) => serde_json::Value::String(string.to_string()),
            Self::Bytes(bytes) => serde_json::Value::String(format!("{:02x?}", bytes).to_string()),
            Self::Integer(int) => serde_json::Value::Number((*int).into()),
            Self::List(vec) => {
                serde_json::Value::Array(vec.iter().map(|el| el.to_json()).collect())
            }
            Self::Dictionary(dict) => {
                let mut map = serde_json::Map::with_capacity(dict.len());
                for (key, value) in dict {
                    map.insert(key.to_string(), value.to_json());
                }
                serde_json::Value::Object(map)
            }
        }
    }
    pub fn to_bencode(&self) -> Vec<u8> {
        fn str_to_bencode(string: &str) -> String {
            format!("{}:{string}", string.len())
        }

        match self {
            Self::String(string) => str_to_bencode(string).as_bytes().to_vec(),
            Self::Bytes(bytes) => bytes
                .len()
                .to_string()
                .as_bytes()
                .iter()
                .chain(iter::once(&BENCODE_STRING_SEPERATOR))
                .chain(bytes.iter())
                .cloned()
                .collect(),

            Self::Integer(int) => format!("i{int}e").as_bytes().to_vec(),

            Self::List(vec) => iter::once(BENCODE_LIST_START)
                .chain(vec.iter().flat_map(|el| el.to_bencode()))
                .chain(iter::once(BENCODE_END_DELIMITER))
                .collect(),

            Self::Dictionary(dict) => iter::once(BENCODE_DICTIONARY_START)
                .chain(dict.iter().flat_map(|(k, v)| {
                    let k_str = str_to_bencode(k);
                    let k_arr = k_str.as_bytes();
                    let mut v_vec = v.to_bencode();
                    let mut res = Vec::with_capacity(k_arr.len() + v_vec.len());
                    res.extend_from_slice(k_arr);
                    res.append(&mut v_vec);
                    res
                }))
                .chain(iter::once(BENCODE_END_DELIMITER))
                .collect(),
        }
    }

    pub fn to_str(&self) -> Option<&str> {
        if let BencodeValue::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    pub fn to_bytes(&self) -> Option<&[u8]> {
        if let BencodeValue::Bytes(bytes) = self {
            Some(bytes)
        } else {
            None
        }
    }

    pub fn to_i64(&self) -> Option<&i64> {
        if let BencodeValue::Integer(int) = self {
            Some(int)
        } else {
            None
        }
    }
    pub fn to_array(&self) -> Option<&[BencodeValue]> {
        if let BencodeValue::List(vec) = self {
            Some(vec)
        } else {
            None
        }
    }

    pub fn to_map(&self) -> Option<&BTreeMap<&str, BencodeValue>> {
        if let BencodeValue::Dictionary(dict) = self {
            Some(dict)
        } else {
            None
        }
    }
}

fn find_bencode_type(value: &[u8]) -> Option<BencodeType> {
    let first_char = value.iter().next()?;
    if first_char.is_ascii_digit() {
        Some(BencodeType::String)
    } else if *first_char == BENCODE_INTEGER_START {
        Some(BencodeType::Integer)
    } else if *first_char == BENCODE_LIST_START {
        Some(BencodeType::List)
    } else if *first_char == BENCODE_DICTIONARY_START {
        Some(BencodeType::Dictionary)
    } else {
        None
    }
}

type ParsingResult<'a> = (BencodeValue<'a>, &'a [u8]);

fn find(bytes: &[u8], delimiter: &u8) -> Option<usize> {
    let (i, _) = bytes
        .iter()
        .enumerate()
        .find(|(_, val)| **val == *delimiter)?;
    Some(i)
}

fn decode_bencoded_string(encoded_value: &[u8]) -> Option<ParsingResult> {
    let colon_index = find(encoded_value, &BENCODE_STRING_SEPERATOR)?;
    let number = from_utf8(&encoded_value[..colon_index])
        .ok()?
        .parse::<i64>()
        .ok()?;
    let bytes = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
    let rest = &encoded_value[colon_index + 1 + number as usize..];
    println!("rest: {:?}",rest);
    if let Ok(string) = from_utf8(bytes) {
        Some((BencodeValue::String(string), rest))
    } else {
        Some((BencodeValue::Bytes(bytes), rest))
    }
}

fn decode_bencoded_integer(encoded_value: &[u8]) -> Option<ParsingResult> {
    let i = find(encoded_value, &BENCODE_END_DELIMITER)?;
    let int = from_utf8(&encoded_value[1..i]).ok()?;
    let rest = &encoded_value[i + 1..];
    Some((BencodeValue::Integer(int.parse::<i64>().ok()?.into()), rest))
}

fn decode_multiple<'a, F: FnMut(&'a [u8]) -> Option<&'a [u8]>>(
    encoded_value: &'a [u8],
    mut parse: F,
) -> Option<&'a [u8]> {
    println!("decode_multiple");
    let mut elements = &encoded_value[1..];
    while !elements.is_empty() {
        if let Some(rest) = parse(elements) {
            elements = rest;
        } else if *elements.iter().next().unwrap() == BENCODE_END_DELIMITER {
            elements = &elements[1..];
            break;
        } else {
            return None;
        }
    }
    Some(elements)
}

fn decode_bencoded_list(encoded_value: &[u8]) -> Option<ParsingResult> {
    println!("decode list");
    let mut list: Vec<BencodeValue> = Vec::new();
    let rest = decode_multiple(encoded_value, |elements: &[u8]| {
        let (element, rest) = decode_bencoded_value_with_rest(elements)?;
        list.push(element);
        Some(rest)
    })?;
    Some((BencodeValue::List(list), rest))
}

fn decode_bencoded_dictionary(encoded_value: &[u8]) -> Option<ParsingResult> {
    println!("decode dict");
    let mut dict: BTreeMap<&str, BencodeValue> = BTreeMap::new();

    let rest = decode_multiple(encoded_value, |elements: &[u8]| {
        if let (BencodeValue::String(key), rest) = decode_bencoded_string(elements)? {
            let (value, rest) = decode_bencoded_value_with_rest(rest)?;
            dict.insert(key, value);
            Some(rest)
        } else {
            println!("bad dict");
            None
        }
    })?;

    Some((BencodeValue::Dictionary(dict), rest))
}

pub fn decode_bencoded_value_with_rest(encoded_value: &[u8]) -> Option<ParsingResult> {
    println!("value_with_rest {:?}", encoded_value);
    match find_bencode_type(encoded_value)? {
        BencodeType::Integer => decode_bencoded_integer(encoded_value),
        BencodeType::String => decode_bencoded_string(encoded_value),
        BencodeType::List => decode_bencoded_list(encoded_value),
        BencodeType::Dictionary => decode_bencoded_dictionary(encoded_value),
    }
}

pub fn decode_bencoded_value(encoded_value: &[u8]) -> Option<BencodeValue> {
    println!("value");
    let (value, rest) = decode_bencoded_value_with_rest(encoded_value)?;
    if rest.is_empty() {
        Some(value)
    } else {
        None
    }
}
