use crate::data::{Frequency, State};
use std::collections::HashMap;
#[derive(Debug)]
pub struct Solutions {
    solutions: Vec<Vec<Vec<String>>>,
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
        let mut solutions: Vec<Vec<usize>> = vec![];
        if branches > 0 {
            match state.args.word_count {
                // Finds all solutions in an iterative DFS manner.
                0 => {
                    let mut stack: Vec<usize> = vec![0];
                    'main: loop {
                        // Sum up the current path then subtract to the anagram, get validity.
                        // TODO: Inefficient. Find a way to cache intermediate sums.
                        let sum: Frequency = {
                            let mut sum: Frequency = Frequency::from(0);
                            for idx in &stack {
                                sum = &sum + &frequencies[*idx];
                            }
                            sum
                        };
                        let validity: bool = Frequency::is_valid(&(&anagram_frequency - &sum));
                        // If the current node is valid but isn't the solution, this means it is less.
                        // Go deeper, start at the initial position.
                        if validity && sum != anagram_frequency {
                            stack.push(*stack.last().unwrap());
                            continue;
                        }
                        // If it is the solution
                        // Push path to solutions.
                        else if sum == anagram_frequency {
                            solutions.push(stack.clone());
                        }
                        // Loop over until you meet a valid node.
                        // Go to sibling if possible.
                        // If at edge, go up to uncle, if it is not available, go to the next uncle and so on.
                        // If 0, break.
                        loop {
                            let depth: usize = stack.len() - 1;
                            if stack[depth] != branches - 1 {
                                stack[depth] += 1;
                                break;
                            } else if depth == 0 && stack[depth] == branches - 1 {
                                break 'main;
                            } else if stack[depth] == branches - 1 {
                                stack.pop();
                            }
                        }
                    }
                }
                // Major bug. Multiple inclusions of the same solution in different orders.
                // Discrete algorithm for finding "N"-word solutions to an anagram.
                1..=u8::MAX => {
                    if state.args.word_count == 1 {
                        match frequencies.iter().position(|i| *i == anagram_frequency) {
                            Some(idx) => solutions.push(vec![idx]),
                            None => (),
                        }
                    } else {
                        let map: &HashMap<Frequency, Vec<String>> = &state.data.str_map;
                        let mut odometer: Vec<usize> = vec![0; (state.args.word_count - 1) as usize];
                        'main: loop {
                            // TODO: Inefficient. Find a way to cache intermediate sums.
                            let sum: Frequency = {
                                let mut sum: Frequency = Frequency::from(0);
                                for idx in &odometer {
                                    sum = &sum + &frequencies[*idx];
                                }
                                sum
                            };
                            let intermediate: Frequency = &anagram_frequency - &sum;
                            if Frequency::is_valid(&sum) && map.contains_key(&intermediate) {
                                let solution: Vec<usize> = {
                                    let mut solution: Vec<usize> = odometer.clone();
                                    match frequencies.iter().position(|i| *i == intermediate)  {
                                        Some(idx) => solution.push(idx),
                                        None => ()
                                    }
                                    solution
                                };
                                solutions.push(solution);
                            } 
                            // - 2 due to hash map avoiding another iteration. -1 taken care of explicitly above.
                            let mut i: usize = (state.args.word_count - 2) as usize;
                            loop {
                                if odometer[i] < branches - 1 {
                                    odometer[i] += 1;
                                    for j in (i + 1)..(state.args.word_count - 1) as usize {
                                        odometer[j] = odometer[i];
                                    }
                                    break;
                                } else if odometer[0] == branches - 1 {
                                    break 'main;
                                }
                                i -= 1;
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
