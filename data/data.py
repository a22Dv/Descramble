import wordfreq
import struct

data = wordfreq.get_frequency_dict("en")
raw_bytes: list[bytes] = []
for k, v in data.items():
    v_b = struct.pack("<d", v)
    str_b = f"{k}\n".encode("utf-8")
    raw_bytes.append(v_b + str_b)
with open("data.bin", "wb") as DATA:
    DATA.write(b"".join(raw_bytes))
    


