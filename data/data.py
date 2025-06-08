import spacy
from wordfreq import word_frequency
import json
from typing import List, Dict
from tqdm import tqdm

INPUT_FILE: str = "english.txt"
OUTPUT_FILE: str = "data.json"
LANGUAGE_CODE: str = "en"
MODEL: str = "en_core_web_lg" 
nlp = spacy.load(MODEL, disable=["parser", "ner"])
data = {}

print(f"Reading words from {INPUT_FILE}...")
all_raw_words: List[str] = []
with open(INPUT_FILE, "r", encoding="utf-8") as f:
    for line in f:
        all_raw_words.append(line.strip().lower())
unique_words: List[str] = sorted(list(set(
    word for word in all_raw_words if word and word.isalpha()
)))
print(f"Found {len(unique_words)} unique alphabetic words.")
if not unique_words:
    print("No valid words found to process.")
    exit()
single_token_words_with_isolated_pos: Dict[str, str] = {}
docs = nlp.pipe(unique_words, batch_size=512)

for word, doc in tqdm(zip(unique_words, docs), total=len(unique_words), desc="Initial POS"):
    if len(doc) == 1: 
        pos = doc[0].pos_
        if pos not in ["X", "SPACE", "PUNCT"]:
            single_token_words_with_isolated_pos[word] = pos
print(f"Found {len(single_token_words_with_isolated_pos)} valid single-token words.")
if not single_token_words_with_isolated_pos:
    print("No valid single-token words after initial spaCy processing.")
    exit()
print("Processing words to get frequency and store canonical POS...")
processed_word_count = 0
for word, isolated_pos in tqdm(single_token_words_with_isolated_pos.items(), desc="Main Processing"):
    freq = word_frequency(word, LANGUAGE_CODE, wordlist='large') # 'large' can be slow; consider 'best'
    if freq == 0:
        continue
    data[word] = {
        "frequency": freq,
        "tag": isolated_pos, 
    }
    processed_word_count += 1
print(f"\nProcessed {processed_word_count} words and added to data.")
print(f"Saving data to {OUTPUT_FILE}...")
with open(OUTPUT_FILE, "w", encoding="utf-8") as O_FILE:
    json.dump(data, O_FILE, indent=2, ensure_ascii=False)
print("DONE.")

