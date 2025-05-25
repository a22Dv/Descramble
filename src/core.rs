use crate::{
    Args,
    app::State,
    data::{Data, Frequency},
};
use std::{collections::{HashMap, HashSet}, thread::current};

#[derive(Default)]
pub struct Solution {
    pub solutions: Vec<(String, f32)>,
}
const DEFAULT_AGGRESSIVENESS: u8 = 3;
const DEFAULT_WORD_COUNT: u8 = 1;
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
        // Some declarations.
        let mut solution: Solution = Solution::default();
        let anagram_freq: Frequency = Frequency::from(anagram.as_bytes());
        let mut data_map: HashMap<Frequency, Vec<String>> = data.data_map;
        let threshold_freq: Frequency = Frequency::from(-1);
        let mut valid_solutions: Vec<Vec<Frequency>> = vec![];

        // Holds where the loop left at each depth,
        let mut depth_continue_at: Vec<u32> = vec![0; max_n as usize];
        let max_branches_idx: u32 = (data.freqs.len() - 1) as u32;
        let mut current_depth: u8 = 0;
        let max_depth: usize = depth_continue_at.len() - 1;
        println!("START: {}\n", data_map.len());
        while depth_continue_at[max_depth] != max_branches_idx {
            // Initial prune.
            if current_depth == 0 {
                for freq in &data.freqs {
                    if (&anagram_freq - &freq).is_invalid() {
                        data_map.remove(&freq);
                    }
                }
                valid_solutions.push(vec![anagram_freq.clone()]);
                depth_continue_at[0] = max_branches_idx;
                current_depth += 1;
                println!("END: {}\n", data_map.len());
                let mut buf = String::new();
                let _ = std::io::stdin().read_line(&mut buf);
                continue;
            }
            // Until the current depth has all been explored.
            
            for branch in depth_continue_at[current_depth as usize]..=max_branches_idx {
                
            }
            current_depth += 1
        }
        solution
    }
}
