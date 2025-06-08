use crate::data::{Frequency, State};
use std::collections::{HashMap, HashSet};
use std::fs::read;

#[derive(Debug)]
pub struct Solutions {
    // A vector of solutions where each solution is a vector that has each frequency point to a vector of valid strings
    // for that frequency.
    solutions: Vec<Vec<Vec<String>>>,
}
const POS_TAG_COUNT: usize = 14;
const TEMPLATE_FIT_REWARD: f64 = 10.0;
const TEMPLATE_UNFIT_PENALTY: f64 = 0.1;
impl Solutions {
    pub fn parse(&self, state: &State) -> Vec<(String, f64)> {
        // Clunky but whatever.
        let repr_index: HashMap<String, usize> = HashMap::from([
            ("ADJ".to_string(), 0),
            ("ADP".to_string(), 1),
            ("ADV".to_string(), 2),
            ("AUX".to_string(), 3),
            ("CCONJ".to_string(), 4),
            ("DET".to_string(), 5),
            ("INTJ".to_string(), 6),
            ("NOUN".to_string(), 7),
            ("NUM".to_string(), 8),
            ("PART".to_string(), 9),
            ("PRON".to_string(), 10),
            ("PROPN".to_string(), 11),
            ("SCONJ".to_string(), 12),
            ("VERB".to_string(), 13),
        ]);
        // POS frequency count to ordered template.
        let templates: HashMap<[u8; POS_TAG_COUNT], Vec<String>> = {
            let mut templates: HashMap<[u8; POS_TAG_COUNT], Vec<String>> = HashMap::default();
            let raw_data: Vec<Vec<String>> = serde_json::from_slice(
                &read(state.root_path.join("data").join("templates.json")).unwrap(),
            )
            .unwrap();
            for template in raw_data {
                let mut key: [u8; POS_TAG_COUNT] = [0; POS_TAG_COUNT];
                for tag in template.iter() {
                    key[repr_index[tag]] += 1;
                }
                templates.insert(key, template);
            }
            templates
        };

        let mut parsed_solution: Vec<(String, f64)> = vec![];
        let solutions: &Vec<Vec<Vec<String>>> = &self.solutions;
        for solution in solutions {
            // Holds possible final solutions, combinations of the final words. Needs reordering.
            let mut phrases: Vec<(Vec<String>, f64)> = Solutions::get_phrases(&solution, state);
            for (phrase, score) in phrases.iter_mut() {
                let tags: Vec<String> = {
                    let mut tags: Vec<String> = vec![];
                    for word in phrase.iter() {
                        tags.push(state.data.string_data[word].tag.clone());
                    }
                    tags
                };
                let key: [u8; POS_TAG_COUNT] = {
                    let mut key: [u8; POS_TAG_COUNT] = [0; POS_TAG_COUNT];
                    for tag in tags.iter() {
                        key[repr_index[tag]] += 1;
                    }
                    key
                };
                if templates.contains_key(&key) {
                    // Modify phrase in place.
                    let template: &Vec<String> = &templates[&key];
                    let pos_idx: HashMap<String, usize> = {
                        let mut pos_idx: HashMap<String, usize> = HashMap::default();
                        for (i,tag) in tags.iter().enumerate() {
                            pos_idx.insert(tag.to_string(), i);
                        }
                        pos_idx
                    };
                    let mut new_phrase: Vec<String> = vec![];
                    for pos in template {
                        new_phrase.push(phrase[pos_idx[pos]].clone());
                    }
                    *phrase = new_phrase;

                    // Boosted due to fitting to a template.
                    *score *= TEMPLATE_FIT_REWARD;
                } else {
                    *score *= TEMPLATE_UNFIT_PENALTY;
                }
                parsed_solution.push((phrase.join(" "), *score))
            }
            
        }
        // Final normalization.
        let mut total_sum: f64 = 0.0;
        for solution in parsed_solution.iter() {
            total_sum += solution.1;
        }
        for solution in parsed_solution.iter_mut() {
            solution.1 = (solution.1 / total_sum) * 100.0;
        }
        parsed_solution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        parsed_solution
    }
    pub fn get_phrases(words: &Vec<Vec<String>>, state: &State) -> Vec<(Vec<String>, f64)> {
        let mut phrases: Vec<(Vec<String>, f64)> = vec![];
        let mut odometer: Vec<usize> = vec![0; words.len()];
        let odo_len: usize = odometer.len();
        'main: loop {
            let mut phrase_mean: f64 = 0.0;
            let phrase_words: Vec<String> = {
                let mut phrase_words: Vec<String> = vec![];
                for (i, idx) in odometer.iter().enumerate() {
                    phrase_mean += state.data.string_data[&words[i][*idx]].frequency;
                    phrase_words.push(words[i][*idx].clone());
                }
                phrase_mean /= odo_len as f64;
                phrase_words
            };
            phrases.push((phrase_words, phrase_mean));
            for i in 0..odo_len {
                if odometer[i] < words[i].len() - 1 {
                    odometer[i] += 1;
                    break;
                } else if i == odo_len - 1 {
                    break 'main;
                } else {
                    for j in 0..=i {
                        odometer[j] = 0
                    }
                }
            }
        }
        phrases
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
            for freq in state.data.string_mapping.keys() {
                // Prunes frequencies where every string goes below the threshold.
                let passes: bool = {
                    let mut flag: bool = false;
                    for str in &state.data.string_mapping[freq] {
                        if state.data.string_data[str].frequency > threshold {
                            flag = true;
                            break;
                        }
                    }
                    flag
                };
                // If it fits within the anagram and at least one of the constituent strings pass frequency threshold.
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
                    solution_buffer.push(state.data.string_mapping[&frequencies[idx]].clone());
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
