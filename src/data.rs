use indexmap::IndexMap;
use std::cmp::PartialEq;
use std::ops::{Add, Sub};
use std::path::{PathBuf, Path};
use clap::Parser;

const ALPHA_COUNT: usize = 26;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    anagram: String,
    #[arg(short, long, default_value_t = 3)]
    words: u8,
    #[arg(short, long, default_value_t = false)]
    file_format: bool,
}
pub struct State {
    args: Args,
    data: Data,
    root_path: PathBuf,
}
impl State {
    pub fn new(args: Args, data: Data, root_path: PathBuf) -> Self {
        State {
            args: args,
            data: data,
            root_path: root_path
        }
    }
}
pub struct Frequency {
    arr: [i8; ALPHA_COUNT],
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
pub struct Data {
    freq_str_idx_map: IndexMap<Frequency, Vec<String>>
}
/// Get dictionary data from a specified path.
impl TryFrom<&Path> for Data {
    type Error = std::io::Error;
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        todo!()
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
}
