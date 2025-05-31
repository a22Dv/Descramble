use clap::Parser;
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
    #[arg(short, long, default_value_t = false)]
    pub formatted: bool,
    #[arg(short, long, default_value_t = 5)]
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
pub struct Data {
    pub str_map: HashMap<Frequency, Vec<String>>,
    pub rate_map: HashMap<String, f64>,
}
#[derive(Debug)]
pub enum DataError {
    ParseError(std::string::FromUtf8Error),
    IOError(std::io::Error),
}
/// Get dictionary data from a specified path.
impl TryFrom<&PathBuf> for Data {
    type Error = DataError;
    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let data: Vec<u8> = match read(path.join("base.bin")) {
            Ok(data) => data,
            Err(err) => return Err(DataError::IOError(err)),
        };
        let valid_short_seq: HashSet<String> = {
            let mut set: HashSet<String> = HashSet::default();
            for word in [
                "a", "i", "am", "an", "as", "at", "be", "by", "do", "he", "hi", "if", "in", "is",
                "it", "me", "my", "no", "of", "oh", "on", "or", "ox", "so", "to", "up", "us"
            ] {
                set.insert(word.to_string());
            }
            set
        };
        let mut freq_map: HashMap<Frequency, Vec<String>> = HashMap::default();
        let mut rate_map: HashMap<String, f64> = HashMap::default();
        let mut seq_idx: usize = 0;
        let mut entry: (String, f64) = (String::default(), 0.0_f64);
        let mut buffer: Vec<u8> = vec![];
        for byte in data.iter() {
            buffer.push(*byte);
            if seq_idx == F64_SIZE - 1 {
                entry.1 = f64::from_le_bytes({
                    let mut arr: [u8; F64_SIZE] = [0; F64_SIZE];
                    for i in 0..F64_SIZE {
                        arr[i] = buffer[i];
                    }
                    arr
                });
                buffer.clear();
            } else if F64_SIZE <= seq_idx && *byte == 0xA {
                buffer.pop();
                entry.0 = match String::from_utf8(buffer.clone()) {
                    Ok(string) => string,
                    Err(err) => return Err(DataError::ParseError(err)),
                };
                if (entry.0.len() < 3 && valid_short_seq.contains(&entry.0)) || entry.0.len() >= 3 {
                    let ltr_freq: Frequency = Frequency::from(entry.0.as_bytes());
                    if freq_map.contains_key(&ltr_freq) {
                        freq_map.get_mut(&ltr_freq).unwrap().push(entry.0.clone())
                    } else {
                        freq_map.insert(ltr_freq, vec![entry.0.clone()]);
                    }
                    rate_map.insert(entry.0, entry.1);
                }
                buffer.clear();
                seq_idx = 0;
                continue;
            }
            seq_idx += 1;
        }
        let data_obj: Data = Data {
            str_map: freq_map,
            rate_map: rate_map,
        };
        Ok(data_obj)
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
