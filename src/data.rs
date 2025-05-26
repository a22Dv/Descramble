use crate::display::DisplayProgress;
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use std::fs::read;
use std::hash::Hash;
use std::io::Error;
use std::ops::{Add, Sub};
use std::path::Path;

#[derive(Default)]
pub struct Data {
    pub data_map: HashMap<Frequency, Vec<String>>,
    pub str_rate_map: HashMap<String, f64>,
    pub freqs: Vec<Frequency>,
    pub rate: Vec<f64>,
}
const DOUBLE_SIZE: usize = 8;

#[derive(Debug)]
pub enum DataError {
    Io(Error),
    InvalidFormat(String),
}
impl From<Error> for DataError {
    fn from(err: Error) -> Self {
        DataError::Io(err)
    }
}
impl From<String> for DataError {
    fn from(value: String) -> Self {
        DataError::InvalidFormat(value)
    }
}
impl TryFrom<&Path> for Data {
    type Error = DataError;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let raw_data = read(value)?;
        let mut freq_vec: Vec<Frequency> = Vec::<Frequency>::default();
        let mut str_vec: Vec<String> = Vec::<String>::default();
        let mut f64_vec: Vec<f64> = Vec::<f64>::default();
        let mut f64_buffer: [u8; DOUBLE_SIZE] = [0; DOUBLE_SIZE];
        let mut str_buffer: Vec<u8> = Vec::default();
        let mut counter: usize = 0;
        let mut allowed_sequences: HashSet<String> = HashSet::default();
        for char in "ai".chars() {
            allowed_sequences.insert(char.to_string());
        }
        for word in vec![
            "ai", "am", "an", "as", "at", "ax", "be", "by", "do", "ex", "go", "he", "hi", "id",
            "if", "in", "is", "it", "me", "my", "no", "of", "oh", "ok", "on", "or", "ox", "pi",
            "qi", "so", "to", "uh", "um", "up", "us", "we",
        ] {
            allowed_sequences.insert(word.to_string());
        }

        let data_progress: DisplayProgress =
            DisplayProgress::new("Reading Data", raw_data.len() as u64, "Byte no.");
        for byte in raw_data {
            if counter < DOUBLE_SIZE {
                f64_buffer[counter] = byte;
            } else if byte != 0xA {
                str_buffer.push(byte);
            } else {
                let mut valid: bool = true;
                if str_buffer.len() < 3 {
                    let string: &str = match &str::from_utf8(&str_buffer) {
                        Ok(string) => string,
                        Err(_) => {
                            return Err(DataError::InvalidFormat(
                                "UTF-8 parsing error in Data::TryFrom.".to_string(),
                            ));
                        }
                    };
                    if !allowed_sequences.contains(string) {
                        valid = false;
                    }
                }
                if valid {
                    for byte in &str_buffer {
                        if *byte < b'a' || b'z' < *byte {
                            valid = false;
                            break;
                        }
                    }
                }
                if valid {
                    str_vec.push(match String::from_utf8(str_buffer.clone()) {
                        Ok(string) => string,
                        Err(_e) => {
                            return Err(DataError::InvalidFormat(
                                "UTF-8 parsing error in Data::TryFrom.".to_string(),
                            ));
                        }
                    });
                    f64_vec.push(f64::from_le_bytes(f64_buffer));
                }
                str_buffer.clear();
                counter = 0;
                continue;
            }
            data_progress.increment(1);
            counter += 1;
        }
        data_progress.finish();
        let data_progress: DisplayProgress =
            DisplayProgress::new("Parsing Data", str_vec.len() as u64, "Pair no.");
        let mut str_rate_map: HashMap<String, f64> = HashMap::default();
        let mut check_set: HashSet<Frequency> = HashSet::<Frequency>::default();
        let mut data_map: HashMap<Frequency, Vec<String>> =
            HashMap::<Frequency, Vec<String>>::default();
        for (i, str) in str_vec.iter().enumerate() {
            let converted_freq: Frequency = Frequency::from(str.as_bytes());
            if check_set.insert(converted_freq.clone()) {
                freq_vec.push(converted_freq.clone());
                data_map.insert(converted_freq.clone(), vec![str.to_string()]);
            } else {
                if let Some(vector) = data_map.get_mut(&converted_freq) {
                    vector.push(str.to_string());
                }
            }
            str_rate_map.insert(str.clone(), f64_vec[i]);
            data_progress.increment(1);
        }
        data_progress.finish();
        Ok(Data {
            data_map: data_map,
            str_rate_map: str_rate_map,
            freqs: freq_vec,
            rate: f64_vec,
        })
    }
}
const ALPHA_COUNT: usize = 26;
#[derive(Hash, Clone, Default, Debug)]
pub struct Frequency {
    pub arr: [i8; ALPHA_COUNT],
}
impl Frequency {
    pub fn is_invalid(&self) -> bool {
        for v in self.arr {
            if v < 0 {
                return true;
            }
        }
        false
    }
    pub fn sum(vec_freq: &Vec<Self>) -> Frequency {
        let mut sum: Frequency = Frequency::from(0);
        for freq in vec_freq {
            sum = &sum + freq;
        }
        sum
    }
}
impl From<&[u8]> for Frequency {
    fn from(value: &[u8]) -> Self {
        let v: Vec<u8> = value.to_ascii_lowercase();
        let mut arr = [0; ALPHA_COUNT];
        for byte in v {
            if b'a' <= byte && byte <= b'z' {
                arr[(byte - b'a') as usize] += 1;
            }
        }
        Frequency { arr: arr }
    }
}
impl From<i8> for Frequency {
    fn from(value: i8) -> Self {
        Frequency {
            arr: [value; ALPHA_COUNT],
        }
    }
}
impl Add for Frequency {
    type Output = Frequency;
    fn add(self, other: Self) -> Self {
        let a = &self;
        let b = &other;
        a + b
    }
}
impl<'a, 'b> Add<&'b Frequency> for &'a Frequency {
    type Output = Frequency;
    fn add(self, other: &'b Frequency) -> Self::Output {
        let mut result = Frequency::default();
        for i in 0..ALPHA_COUNT {
            result.arr[i] = self.arr[i] + other.arr[i];
        }
        result
    }
}
impl Sub for Frequency {
    type Output = Frequency;
    fn sub(self, other: Self) -> Self {
        let a = &self;
        let b = &other;
        a - b
    }
}
impl<'a, 'b> Sub<&'b Frequency> for &'a Frequency {
    type Output = Frequency;
    fn sub(self, other: &'b Frequency) -> Self::Output {
        let mut result = Frequency::default();
        for i in 0..ALPHA_COUNT {
            result.arr[i] = self.arr[i] - other.arr[i];
        }
        result
    }
}
impl PartialEq for Frequency {
    fn eq(&self, other: &Self) -> bool {
        self.arr == other.arr
    }
}
impl Eq for Frequency {}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_init() {
        assert_eq!(Frequency::from(0).arr, [0 as i8; ALPHA_COUNT]);
        assert_eq!(
            Frequency::from("he7;o".as_bytes()).arr,
            [
                0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            Frequency::from("HELLO".as_bytes()).arr,
            [
                0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            Frequency::from("hello".as_bytes()).arr,
            [
                0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            Frequency::from("HeLLo".as_bytes()).arr,
            [
                0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
    #[test]
    fn test_impl() {
        assert_eq!(Frequency::from(-1).is_invalid(), true);
        assert_eq!(
            (Frequency::from("From".as_bytes()) - Frequency::from("mm".as_bytes())).is_invalid(),
            true
        );
    }
    #[test]
    fn test_ops() {
        assert_eq!(
            Frequency::from("Hello".as_bytes()) + Frequency::from("Hello".as_bytes()),
            Frequency::from("HelloHello".as_bytes())
        );
        assert_eq!(
            Frequency::from("Hello".as_bytes()) - Frequency::from("Hello".as_bytes()),
            Frequency::from("".as_bytes())
        );
        assert_eq!(
            Frequency::from("Hello".as_bytes()) - Frequency::from("llo".as_bytes()),
            Frequency::from("He".as_bytes())
        )
    }
}
