use crate::data::{Frequency, State};
use std::collections::{HashMap, HashSet};
#[derive(Debug)]
pub struct Solutions {
    solutions: Vec<Vec<Vec<String>>>,
}
impl Solutions {
    pub fn parse(&self, state: &State) -> Vec<(String, f64)> {
        let mut parsed_solution: Vec<(String, f64)> = vec![];
        let rate_map: &HashMap<String, f64> = &state.data.rate_map;
        for solution in &self.solutions {
            let mut phrases: Vec<String> = vec![];
            let mut mean_scores: Vec<f64> = vec![];
            let mut odometer: Vec<usize> = vec![0; solution.len()];
            let odo_len: usize = odometer.len();

            // Create phrases.
            'main: loop {
                let mut words: Vec<String> = vec![];
                let mut mean: f64 = 0.0;
                for (i, idx) in odometer.iter().enumerate() {
                    words.push(solution[i][*idx].clone());
                    let score: f64 = rate_map[&solution[i][*idx]];
                    mean += score;
                }
                mean_scores.push(mean / words.len() as f64);
                phrases.push(words.join(" "));
                for i in 0..odo_len {
                    if odometer[i] + 1 < solution[i].len() {
                        odometer[i] += 1;
                        break;
                    } else {
                        if i == 0 {
                            break 'main;
                        }
                        for j in i..odo_len {
                            odometer[j] = odometer[i];
                        }
                    }
                }
            }
            for (phrase, score) in phrases.iter().zip(mean_scores) {
                parsed_solution.push((phrase.clone(), score));
            }
        }
        let mut sol_sum: f64 = 0.0;
        for solution in &parsed_solution {
            sol_sum += solution.1;
        }
        for solution in parsed_solution.iter_mut() {
            solution.1 = (solution.1 / sol_sum) * 100.0;
        }
        parsed_solution.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        return parsed_solution;
    }
}
impl From<&State> for Solutions {
    fn from(state: &State) -> Self {
        let threshold: f64 =
            { 1e-9_f64 + (1e-4_f64 - 1e-9_f64) * (f64::from(state.args.strength) / 10_f64) };
        let anagram_frequency: Frequency = Frequency::from(state.args.anagram.as_bytes());
        // `frequencies` is already pre-filtered from the initial list based on frequency.
        let frequencies: Vec<Frequency> = {
            let mut frequencies: Vec<Frequency> = vec![];
            for freq in state.data.str_map.keys() {
                // Prunes frequencies where every string goes below the threshold.
                let passes: bool = {
                    let mut flag: bool = false;
                    for str in &state.data.str_map[freq] {
                        if state.data.rate_map[str] > threshold {
                            flag = true;
                            break;
                        }
                    }
                    flag
                };
                if Frequency::is_valid(&(&anagram_frequency - freq)) && passes {
                    frequencies.push(*freq);
                }
            }
            frequencies
        };
        let branches: usize = frequencies.len();
        let mut solutions: HashSet<Vec<usize>> = HashSet::default();
        if branches > 0 {
            match state.args.word_count {
                0 => {
                    // Set stack and sum caching to avoid repeated recalculations.
                    let mut sum_cache: Frequency = Frequency::from(0);
                    let mut stack: Vec<usize> = vec![0];
                    'main: loop {
                        // Get depth and sum using current and cache.
                        let mut depth: usize = stack.len() - 1;
                        let sum: Frequency = &sum_cache + &frequencies[stack[depth]];
                        let validity: bool = Frequency::is_valid(&(&anagram_frequency - &sum));
                        // Valid but not the answer. Less than frequency
                        if validity && sum != anagram_frequency {
                            sum_cache = sum;
                            // Forces combinations instead of permutations.
                            stack.push(*stack.last().unwrap());
                            continue;
                        // Is the answer.
                        } else if sum == anagram_frequency {
                            solutions.insert(stack.clone());
                        }
                        loop {
                            depth = stack.len() - 1;
                            // Can still iterate through depth.
                            if stack[depth] < branches - 1 {
                                stack[depth] += 1;
                                break;
                            // Go up to sibling in next iteration.
                            } else {
                                // We're already at the root, break.
                                if depth == 0 {
                                    break 'main;
                                // We can still go higher.
                                } else {
                                    // We pop the cache by subtracting the frequency that the pointer
                                    // is pointing to above.
                                    sum_cache = &sum_cache - &frequencies[stack[depth - 1]];
                                    stack.pop();
                                }
                            }
                        }
                    }
                }
                1..=u8::MAX => {
                    let freq_idx: HashMap<Frequency, usize> = {
                        let mut map: HashMap<Frequency, usize> = HashMap::default();
                        for (i, freq) in frequencies.iter().enumerate() {
                            map.insert(freq.clone(), i);
                        }
                        map
                    };
                    if state.args.word_count == 1 {
                        if freq_idx.contains_key(&anagram_frequency) {
                            solutions.insert(vec![freq_idx[&anagram_frequency]]);
                        }
                    } else {
                        let mut odometer: Vec<usize> =
                            vec![0; (state.args.word_count - 1) as usize];

                        'main: loop {
                            let sum = {
                                let mut sum: Frequency = Frequency::default();
                                for idx in odometer.iter() {
                                    sum = &sum + &frequencies[*idx];
                                }
                                sum
                            };
                            let other = &anagram_frequency - &sum;
                            if freq_idx.contains_key(&other) {
                                let mut solution = odometer.clone();
                                solution.push(freq_idx[&other]);
                                solution.sort();
                                solutions.insert(solution);
                            }
                            for i in (0..odometer.len()).rev() {
                                // Has not rolled over.
                                if odometer[i] < branches - 1 {
                                    odometer[i] += 1;
                                    break;
                                // Should roll over. But breaks since we're at the root.
                                } else if i == 0 {
                                    break 'main;
                                // Roll over.
                                } else {
                                    for j in i..odometer.len() {
                                        if odometer[i - 1] + 1 < branches {
                                            odometer[j] = odometer[i - 1] + 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // 3D Vectors due to storing vector solutions where each frequency points to vectors of strings.
        let raw_solution: Vec<Vec<Vec<String>>> = {
            let mut raw: Vec<Vec<Vec<String>>> = vec![];
            for solution in solutions {
                let mut solution_buffer: Vec<Vec<String>> = vec![];
                for idx in solution {
                    solution_buffer.push(state.data.str_map[&frequencies[idx]].clone());
                }
                raw.push(solution_buffer);
            }
            raw
        };
        Solutions {
            solutions: raw_solution,
        }
    }
}
