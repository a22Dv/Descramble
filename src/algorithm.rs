use crate::data::{Frequency, State};
use indicatif::{ProgressBar, ProgressStyle};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fs::read;
use std::time::Duration;
use std::usize;

#[derive(Debug)]
pub struct Solutions {
    // A vector of solutions where each solution is a vector that has each frequency point to a vector of valid strings
    // for that frequency.
    solutions: Vec<Vec<Vec<String>>>,
}
const POS_TAG_COUNT: usize = 14;
const TEMPLATE_FIT_REWARD: f64 = 1.0;
const TEMPLATE_UNFIT_PENALTY: f64 = 0.1;
impl Solutions {
    pub fn parse(&self, state: &State) -> Vec<((u32, u32), String, f64)> {
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

        let mut parsed_solution: Vec<((u32, u32), String, f64)> = vec![];
        // [[[statue, astute], [of], [liberty]], ...]
        let solutions: &Vec<Vec<Vec<String>>> = &self.solutions;
        let pb: ProgressBar = ProgressBar::new(solutions.len() as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "[{bar:.white}] [{pos} of {len}] [{elapsed_precise}] {msg}",
            )
            .unwrap()
            .progress_chars("▪▪ "),
        );
        pb.set_message("Processing solutions...");
        for (i, solution) in solutions.iter().enumerate() {
            // Holds possible final solutions, combinations of the final words. Needs reordering.
            // [[astute, of, liberty], [statue, of, liberty]]
            let mut phrases: Vec<(Vec<String>, f64)> = Solutions::get_phrases(&solution, state);
            for (j, (phrase, score)) in phrases.iter_mut().enumerate() {
                // phrase: [statue, of, liberty]
                // Get the tags for the phrase.
                let tags: Vec<String> = {
                    let mut tags: Vec<String> = vec![];
                    for word in phrase.iter() {
                        let tag: &String = &state.data.string_data[word].tag;
                        // Proper nouns can be treated as nouns.
                        if tag != "PROPN" {
                            tags.push(tag.clone());
                        } else {
                            tags.push("NOUN".to_string());
                        }
                    }
                    tags
                };
                // Get the key.
                let key: [u8; POS_TAG_COUNT] = {
                    let mut key: [u8; POS_TAG_COUNT] = [0; POS_TAG_COUNT];
                    for tag in tags.iter() {
                        key[repr_index[tag]] += 1;
                    }
                    key
                };
                // Can fit a template.
                if templates.contains_key(&key) {
                    let template: &Vec<String> = &templates[&key];
                    // Map tags to the positions they were found in.
                    let pos_idx: HashMap<String, Vec<usize>> = {
                        let mut pos_idx: HashMap<String, Vec<usize>> = HashMap::default();
                        for (i, tag) in tags.iter().enumerate() {
                            if pos_idx.contains_key(tag) {
                                pos_idx.get_mut(tag).unwrap().push(i);
                            } else {
                                pos_idx.insert(tag.to_string(), vec![i]);
                            }
                        }
                        pos_idx
                    };
                    *score *= TEMPLATE_FIT_REWARD;
                    let phrases_indices: Vec<Vec<usize>> = Solutions::reorder(&template, &pos_idx);
                    let mut final_solutions: Vec<((u32, u32), String, f64)> = vec![];
                    for indices in phrases_indices {
                        let mut phrase_solution: Vec<String> = vec![];
                        for idx in indices.iter() {
                            phrase_solution.push(phrase[*idx].clone());
                        }
                        final_solutions.push((
                            (i as u32, j as u32),
                            phrase_solution.join(" "),
                            *score,
                        ));
                    }
                    parsed_solution.extend(final_solutions);
                } else {
                    *score *= TEMPLATE_UNFIT_PENALTY;
                    parsed_solution.push(((i as u32, j as u32), phrase.join(" "), *score));
                }
            }
            pb.inc(1);
        }
        // Final normalization.
        let mut total_sum: f64 = 0.0;
        for solution in parsed_solution.iter() {
            total_sum += solution.2;
        }
        for solution in parsed_solution.iter_mut() {
            solution.2 = (solution.2 / total_sum) * 100.0;
        }
        parsed_solution.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        if parsed_solution.len() < state.args.top_results as usize {
            return parsed_solution;
        }
        parsed_solution[0..state.args.top_results as usize].to_vec()
    }
    pub fn reorder(
        template: &Vec<String>,
        pos_idx: &HashMap<String, Vec<usize>>,
    ) -> Vec<Vec<usize>> {
        // Map the positions inside each POS tag in POS_IDX to the template.
        // Maps POS tags to their individual order permutations.
        // "NOUN: [(0, 1), (1, 0)], ADJ: [(2)]"
        let mut subpermutations: HashMap<String, Vec<Vec<usize>>> = HashMap::default();
        for pos_id in pos_idx.keys() {
            let data: &Vec<usize> = &pos_idx[pos_id];
            let permutations: Vec<Vec<usize>> =
                data.iter().cloned().permutations(data.len()).collect();
            subpermutations.insert(pos_id.clone(), permutations);
        }
        // Consistent key order.
        let cartesian_element_order: Vec<String> = {
            let mut cartesian_element_order: Vec<String> = vec![];
            for key in subpermutations.keys() {
                cartesian_element_order.push(key.clone());
            }
            cartesian_element_order
        };

        // Get the cartesian product of these elements. -> (0 1 2, 1 0 2)
        // Means first noun permut, second adj permut, third verb permut, etc...
        // Map these permutations to the template itself in order.
        let mut orders: Vec<Vec<usize>> = vec![];
        let mut odometer: Vec<usize> = vec![0; subpermutations.len()];
        let odo_len: usize = odometer.len();
        'main: loop {
            orders.push(odometer.clone());
            for i in 0..odo_len {
                if odometer[i] < subpermutations[&cartesian_element_order[i]].len() - 1 {
                    odometer[i] += 1;
                    break;
                } else if i == odo_len - 1 {
                    break 'main;
                } else {
                    for j in 0..=i {
                        odometer[j] = 0;
                    }
                }
            }
        }
        let mut weaved_elements: Vec<Vec<usize>> = vec![];
        // We go through each conceived order
        for order in orders {
            // Each order follows the element order, so we loop through that.
            let mut weaved_element: Vec<usize> = vec![0; template.len()];
            for (i, tag) in cartesian_element_order.iter().enumerate() {
                // For each tag, we get the corresponding permutation described by the order.
                let tag_permutation: &Vec<usize> = &subpermutations[tag][order[i]];
                let mut matched: usize = 0;
                for (j, pos) in template.iter().enumerate() {
                    if pos == tag {
                        weaved_element[j] = tag_permutation[matched];
                        matched += 1
                    }
                }
            }
            weaved_elements.push(weaved_element);
        }
        weaved_elements
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
    pub fn display_solution(solutions: Vec<((u32, u32), String, f64)>) {
        let mut original_order: Vec<(u32, u32)> = vec![];
        let mut format_variations: HashMap<(u32, u32), (Vec<String>, f64)> = HashMap::default();
        for solution in solutions {
            if format_variations.contains_key(&solution.0) {
                format_variations
                    .get_mut(&solution.0)
                    .unwrap()
                    .0
                    .push(solution.1);
            } else {
                format_variations.insert(solution.0, (vec![solution.1], solution.2));
                original_order.push(solution.0);
            }
        }
        let mut max_sequence_length: usize = 0;
        let mut display_string: Vec<u8> = vec![];
        for key in &original_order {
            let string_value: String = (format_variations[&key].0).join(" / ");
            let str_len: usize = string_value.len();
            if max_sequence_length < str_len {
                max_sequence_length = str_len;
            }
        }
        for key in &original_order {
            let string_value: String = (format_variations[&key].0).join(" / ");
            let str_len: usize = string_value.len();
            let float_value: f64 = format_variations[&key].1;
            if max_sequence_length < str_len {
                max_sequence_length = str_len;
            }
            display_string.extend(
                format!(
                    "{}{} - {:.2}%\n",
                    string_value,
                    String::from_utf8(vec![b' '; max_sequence_length - str_len]).unwrap(),
                    float_value
                )
                .as_bytes(),
            )
        }
        print!("{}", String::from_utf8(display_string).unwrap());
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
                    let spinner: ProgressBar = ProgressBar::new_spinner();
                    spinner.set_style(
                        ProgressStyle::with_template(
                            "[{spinner:.white}] [{elapsed_precise}] {msg}",
                        )
                        .unwrap()
                        .tick_strings(&[
                            "▪▪▪▪▪     ",
                            " ▪▪▪▪▪    ",
                            "  ▪▪▪▪▪   ",
                            "   ▪▪▪▪▪  ",
                            "     ▪▪▪▪▪",
                            "      ▪▪▪▪",
                            "       ▪▪▪",
                            "        ▪▪",
                            "         ▪",
                            "▪         ",
                            "▪▪        ",
                            "▪▪▪       ",
                            "▪▪▪▪      ",
                        ]),
                    );
                    spinner.set_message("Finding solutions...");
                    spinner.enable_steady_tick(Duration::from_millis(100));
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
                    let freq_amount: usize = frequencies.len();
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
                        let pb: ProgressBar = ProgressBar::new(freq_amount as u64);
                        pb.set_style(
                            ProgressStyle::with_template(
                                "[{bar:.white}] [{pos} of {len}] [{elapsed_precise}] {msg}",
                            )
                            .unwrap()
                            .progress_chars("▪▪ "),
                        );
                        pb.set_message("Finding solutions...");
                        let mut odometer: Vec<usize> =
                            vec![0; (state.args.word_count - 1) as usize];
                        'main: loop {
                            pb.set_position(odometer[0] as u64);
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
