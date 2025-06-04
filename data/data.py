import wordfreq
import struct

valid: int = 0
miss: int = 0
line_count: int = 0
mean_freq: float = 0.0
LOWER: int = ord("a")
UPPER: int = ord("z")
with open("base.bin", "wb") as BASE:
    with open("english.txt", "r") as FILE:
        data: str = FILE.read()
        for line in data.split("\n"):
            validity = True
            for c in line:
                if not (LOWER <= ord(c) <= UPPER):
                    validity = False
                    break   
            line_count += 1
            if not validity:
                miss += 1
                continue
           
            freq_data: float = wordfreq.word_frequency(line, "en", "large")
            if freq_data > 0: # Words with a frequency of 0 must be filtered out.
                valid += 1
                BASE.write(struct.pack("<d", freq_data))
                BASE.write(f"{line}\n".encode("ascii"))
                mean_freq += freq_data
            else:
                miss += 1
print(f"VALID: {valid} MISS: {miss} COUNT: {line_count} MEAN: {mean_freq / valid}")
