use clap::Parser;
use serde::Deserialize;
use serde_json::{self, Deserializer};
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::fs::read;
use std::ops::{Add, Sub};
use std::path::PathBuf;

const ALPHA_COUNT: usize = 26;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    pub anagram: String,
    #[arg(short, long, default_value_t = 0)]
    pub word_count: u8,
    #[arg(short, long, default_value_t = 50)]
    pub top_results: u32,
    #[arg(short, long, default_value_t = 0)]
    pub strength: u8,
}

pub struct State {
    pub args: Args,
    pub data: Data,
    pub root_path: PathBuf,
}
impl State {
    pub fn new(args: Args, data: Data, root_path: PathBuf) -> Self {
        State {
            args: args,
            data: data,
            root_path: root_path,
        }
    }
}

#[derive(Default, Eq, Hash, Debug, Clone, Copy)]
pub struct Frequency {
    pub arr: [i8; ALPHA_COUNT],
}
impl Frequency {
    pub fn is_valid(freqeuncy: &Frequency) -> bool {
        for val in freqeuncy.arr {
            if val < 0 {
                return false;
            }
        }
        true
    }
}
impl From<&[u8]> for Frequency {
    fn from(byte_array: &[u8]) -> Self {
        let mut data: [i8; ALPHA_COUNT] = [0; ALPHA_COUNT];
        for byte in byte_array {
            let val: u8 = *byte | 0b00100000;
            if b'a' <= val && val <= b'z' {
                data[(val - b'a') as usize] += 1;
            }
        }
        Frequency { arr: data }
    }
}
impl From<i8> for Frequency {
    fn from(value: i8) -> Self {
        let mut data: [i8; ALPHA_COUNT] = [0; ALPHA_COUNT];
        for val in data.iter_mut() {
            *val = value;
        }
        Frequency { arr: data }
    }
}
impl<'a, 'b> Add<&'b Frequency> for &'a Frequency {
    type Output = Frequency;
    fn add(self, other: &'b Frequency) -> Frequency {
        let mut result: [i8; ALPHA_COUNT] = [0; ALPHA_COUNT];
        for i in 0..ALPHA_COUNT {
            result[i] = self.arr[i] + other.arr[i];
        }
        Frequency { arr: result }
    }
}
impl<'a, 'b> Sub<&'b Frequency> for &'a Frequency {
    type Output = Frequency;
    fn sub(self, other: &'b Frequency) -> Frequency {
        let mut result: [i8; ALPHA_COUNT] = [0; ALPHA_COUNT];
        for i in 0..ALPHA_COUNT {
            result[i] = self.arr[i] - other.arr[i];
        }
        Frequency { arr: result }
    }
}
impl PartialEq for Frequency {
    fn eq(&self, other: &Frequency) -> bool {
        self.arr == other.arr
    }
}
const F64_SIZE: usize = size_of::<f64>();

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub frequency: f64,
    pub tag: String,
}
#[derive(Debug)]
pub enum DataError {
    ParseError(std::string::FromUtf8Error),
    IOError(std::io::Error),
}
pub struct Data {
    pub string_mapping: HashMap<Frequency, Vec<String>>,
    pub string_data: HashMap<String, Entry>,
}
/// Get dictionary data from a specified path.
impl TryFrom<&PathBuf> for Data {
    type Error = DataError;
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let data: HashMap<String, Entry> =
            serde_json::from_slice(&read(path.join("data.json")).unwrap()).unwrap();
        let mut mappings: HashMap<Frequency, Vec<String>> = HashMap::default();
        let valid_short_strings: HashSet<&str> = HashSet::from([
            "a", "i", "am", "an", "as", "at", "be", "by", "do", "he", "hi", "if", "in", "is", "it",
            "me", "my", "no", "of", "oh", "on", "or", "ox", "so", "to", "up", "us",
        ]);
        'main: for string in data.keys() {
            for char in string.as_bytes() {
                if !(b'a' <= *char && *char <= b'z') {
                    continue 'main;
                }
            }
            if string.len() < 3 && !valid_short_strings.contains(string.as_str()) {
                continue 'main;
            }
            let frequency = Frequency::from(string.as_bytes());
            if mappings.contains_key(&frequency) {
                mappings.get_mut(&frequency).unwrap().push(string.clone());
            } else {
                mappings.insert(frequency, vec![string.clone()]);
            }
        }
        Ok(Data {
            string_mapping: mappings,
            string_data: data,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::data::Frequency;
    #[test]
    fn test_frequency_array() {
        let string = "AAAbbba";
        assert_eq!(
            [
                4, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            Frequency::from(string.as_bytes()).arr
        );
    }
    #[test]
    fn test_from_freq() {
        let a = Frequency::from(1);
        assert_eq!(a, Frequency::from("abcdefghijklmnopqrstuvwxyz".as_bytes()));
    }
    #[test]
    fn test_freq_ops() {
        let a = Frequency::from(1);
        assert_eq!(&a - &a, Frequency::default());
        assert_eq!(&a + &a, Frequency::from(2));
    }
    #[test]
    fn test_freq_fn() {
        let mut a = Frequency::from(1);
        a.arr[0] = -1;
        assert_eq!(Frequency::is_valid(&a), false);
    }
}
