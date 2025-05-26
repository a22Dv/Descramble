use crate::{
    Args,
    app::State,
    data::{Data, Frequency},
    DisplayProgress
};
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct Solution {
    pub solutions: Vec<String>,
}
const DEFAULT_AGGRESSIVENESS: u8 = 3;
const DEFAULT_WORD_COUNT: u8 = 3;
const DEFAULT_WORD_COUNT_RANGE: u8 = 3;
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
        let afreq: Frequency = Frequency::from(anagram.as_bytes());
        let matched: Frequency = Frequency::from(0);
        let freqs: Vec<Frequency> = data.freqs;
        let threshold: f64 = {
            let mut t: f64 = 1e-10;
            for _ in 0..aggressiveness {
                t *= 10.0;
            }
            t
        };
        DisplayProgress::send_message(&format!("Set threshold: {}\n", threshold), (180, 180, 180));
        let mut data_map: HashMap<Frequency, Vec<String>> = data.data_map.clone();
        // Initial pruning.
        DisplayProgress::send_message(&format!("Total Words: {}\n", data_map.len()), (180, 180, 180));
        for freq in &freqs {
            if (&afreq - freq).is_invalid() || {
                let mut mean: f64 = 0.0;
                for str in &data_map[freq] {
                    mean += data.str_rate_map[str];
                }
                (mean / data_map[freq].len() as f64) < threshold
            } {
                data_map.remove(freq);
            }
        }
        DisplayProgress::send_message(&format!("Invalid words: "), (180, 180, 180));
        DisplayProgress::send_message(&format!("{}\n", data.data_map.len() - data_map.len()), (200, 100, 100));
        DisplayProgress::send_message(&format!("Valid words: "), (180, 180, 180));
        DisplayProgress::send_message(&format!("{}\n", data_map.len()), (100, 200, 100));

        let progress: DisplayProgress = DisplayProgress::new("Exploring branches: ", (data_map.len() as u64).pow(max_n as u32), "Branch no. ");
        let freqs: Vec<Frequency> = data_map.keys().cloned().collect();
        let branch_count: u32 = data_map.len() as u32;
        let end: Vec<u32> = vec![branch_count - 1 as u32; max_n as usize];
        let mut solutions: Vec<Vec<Frequency>> = vec![];
        let mut pointers: Vec<u32> = vec![0; max_n as usize];
        let mut signatures: HashSet<u64> = HashSet::default();
        let mut depth: u8 = 0;
        let mut depth_adjusted_end: Vec<u32> = vec![0; max_n as usize];
        depth_adjusted_end[depth as usize] = branch_count - 1;
        // Main loop.
        loop {
            let mut xor_hash: u64 = 0;
            for ptr in &pointers[0..=depth as usize] {
                xor_hash ^= (*ptr as u64)
                    .wrapping_mul(0x9E3779B97F4A7C15)
                    .wrapping_add(0x7F4A7C159E3779B9)
                    >> 30;
            }
            // Build current frequency based on pointers and depth.
            if !signatures.contains(&xor_hash) {
                let eval_freq: Vec<Frequency> = {
                    let mut eval_freqs: Vec<Frequency> = vec![];
                    for key in &pointers[0..=depth as usize] {
                        eval_freqs.push(freqs[*key as usize].clone());
                    }
                    eval_freqs
                };
                // Valid solution if match.
                if &afreq - &Frequency::sum(&eval_freq) == matched {
                    solutions.push(eval_freq);
                }
                signatures.insert(xor_hash);
            }
            // Break before update if all branches have been explored.
            if pointers == end {
                break;
            }
            // Update pointers.
            for i in (0..=depth as usize).rev() {
                if pointers[i] == branch_count - 1 {
                    pointers[i] = 0;
                } else {
                    pointers[i] += 1;
                    break;
                }
            }
            // Check if all branches at current depth is explored.
            if pointers == depth_adjusted_end {
                // Increment only if 2nd to the last valid.
                if depth < max_n - 1 {
                    // Reset.
                    pointers[0] = 0;
                    depth += 1;
                    depth_adjusted_end[depth as usize] = branch_count - 1;
                }
            }
            progress.increment(1);
        }
        progress.finish();
        for sol in solutions {
            println!("SET");
            for s in sol {
                println!("{:?}", data_map[&s]);
                
            }
        }
        let solution: Solution = Solution::default();
        solution
    }
}
