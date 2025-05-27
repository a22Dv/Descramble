use crate::{
    Args, DisplayProgress,
    app::State,
    data::{Data, Frequency},
};
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct Solution {
    pub solutions: Vec<String>,
}
const DEFAULT_AGGRESSIVENESS: u8 = 3;
const DEFAULT_WORD_COUNT: u8 = 3;
impl From<State> for Solution {
    fn from(state: State) -> Self {
        let args: Args = state.args;
        let word_count_value: u8 = args.word_count.unwrap_or(DEFAULT_WORD_COUNT);
        Solution::get_solution(
            args.anagram,
            args.aggressiveness.unwrap_or(DEFAULT_AGGRESSIVENESS),
            word_count_value,
            args.llm_check,
            state.data,
        )
    }
}
impl Solution {
    fn get_solution(
        anagram: String,
        aggressiveness: u8,
        max_n: u8,
        llm_check: bool,
        data: Data,
    ) -> Solution {
        // Threshold curve.
        // let threshold: f32 = 0.25_f32.powf(12.5 - aggressiveness as f32);
        let max_depth_idx: usize = (max_n - 1) as usize;
        let anagram_frequency: Frequency = Frequency::from(anagram.as_bytes());
        let zeroed: Frequency = Frequency::from(0);
        let mut data_map = data.data_map;
        // Pruned list.
        let frequencies: Vec<Frequency> = {
            let mut valid_frequencies: Vec<Frequency> = vec![];
            for freq in data.freqs {
                if !(&anagram_frequency - &freq).is_invalid() {
                    valid_frequencies.push(freq.clone());
                } else {
                    data_map.remove(&freq);
                }
            }
            valid_frequencies
        };
        let branches: u32 = (frequencies.len() - 1) as u32;
        let mut solutions: Vec<Vec<Frequency>> = vec![];
        let mut odometer: Vec<u32> = vec![0; max_depth_idx + 1];
        let mut signature: HashSet<Vec<u32>> = HashSet::default();
        let end: Vec<u32> = vec![branches; max_depth_idx + 1];
        loop {
            let mut sig: Vec<u32> = odometer.clone();
            sig.sort();
            if !signature.contains(&sig) {
                signature.insert(sig.clone());
                let mut eval_branch: Vec<Frequency> = vec![];
                for i in 0..=max_depth_idx as usize {
                    let branch_sum = Frequency::sum(&eval_branch);
                    let is_solution = &anagram_frequency - &branch_sum == zeroed;
                    // If NOT invalid, push then continue.
                    if !branch_sum.is_invalid() && !is_solution  {
                        eval_branch.push(frequencies[odometer[i] as usize].clone());
                        continue;
                    // Is a solution, therefore push then go onto the next.
                    } else if is_solution {
                        
                        solutions.push(eval_branch.clone());
                        
                    }
                    // Is an invalid node or valid solution -
                    // from the get-go, so you cut prematurely.
                    if i < max_depth_idx && odometer[i] < branches {
                        for j in i + 1..=max_depth_idx {
                            odometer[j] = 0;
                        }
                        odometer[i] += 1;
                        break;
                    }
                }            
            }
            // Reached end.
            if odometer == end {
                break;
            }
            for ptr in odometer.iter_mut().rev() {
                if *ptr != branches {
                    *ptr += 1;
                    break;
                } else {
                    *ptr = 0;
                }
            }
        }
        for v in solutions {
            println!("NEW SET");
            for s in v {
                dbg!(&data_map[&s]);
            }
        }
        let solution: Solution = Solution::default();
        solution
    }
}
