use std::cmp::{Ordering, PartialEq};
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Sub};
use std::fs::read;
use std::hash::Hash;
use std::io::Error;
use std::path::Path;
use crate::display::DisplayProgress;

#[derive(Default)]
pub struct Data {
    pub freq_str_map: HashMap<usize, Vec<usize>>,
    pub freqs: Vec<Frequency>,
    pub strs: Vec<String>,
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
        let mut freq_str_map: HashMap<usize, Vec<usize>> = HashMap::<usize, Vec<usize>>::default();
        let mut freq_vec: Vec<Frequency> = Vec::<Frequency>::default();
        let mut str_vec: Vec<String> = Vec::<String>::default();
        let mut f64_vec: Vec<f64> = Vec::<f64>::default();
        let mut f64_buffer: [u8; DOUBLE_SIZE] = [0; DOUBLE_SIZE];
        let mut str_buffer: Vec<u8> = Vec::default();
        let mut counter: usize = 0;

        let data_progress: DisplayProgress = DisplayProgress::new("Reading Data", raw_data.len() as u64, "Byte no.");
        for byte in raw_data {
            if counter < DOUBLE_SIZE {
                f64_buffer[counter] = byte;
            } else if byte != 0xA {
                str_buffer.push(byte);
            } else {
                let mut valid: bool = true;
                for byte in &str_buffer {
                    if *byte < b'a' || b'z' < *byte {
                        valid = false;
                        break;
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
        let data_progress: DisplayProgress = DisplayProgress::new("Parsing Data", str_vec.len() as u64, "Pair no.");
        let mut check_set: HashSet<Frequency> = HashSet::<Frequency>::default();
        let mut freq_id_temp: HashMap<Frequency, usize> = HashMap::<Frequency, usize>::default();
        let mut unique_freq_id: usize = 0;
        for (i, str) in str_vec.iter().enumerate() {
            let converted_freq: Frequency = Frequency::from(str.as_bytes());
            if check_set.insert(converted_freq.clone()) {
                freq_vec.push(converted_freq.clone());
                freq_str_map.insert(unique_freq_id, vec![i]);
                freq_id_temp.insert(converted_freq, unique_freq_id);
                unique_freq_id += 1;
            } else {
                if let Some(i_val) = freq_str_map.get_mut(&freq_id_temp[&converted_freq]) {
                    i_val.push(i);
                }
            }
            data_progress.increment(1);
        }
        data_progress.finish();
        Ok(Data {
            freq_str_map: freq_str_map,
            freqs: freq_vec,
            strs: str_vec,
            rate: f64_vec,
        })
    }
}
const ALPHA_COUNT: usize = 26;
#[derive(Hash, Clone, Default, Debug)]
pub struct Frequency {
    pub arr: [i8; ALPHA_COUNT],
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
impl PartialOrd for Frequency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut gt: bool = false;
        let mut lt: bool = false;
        for (i, j) in self.arr.iter().zip(other.arr.iter()) {
            if i < j {
                lt = true;
            } else if i > j {
                gt = true;
            }
        }
        match (gt, lt) {
            (false, false) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Greater),
            (false, true) => Some(Ordering::Less),
            (true, true) => None,
        }
    }
}
