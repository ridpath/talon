# 🔬 Fuzzes the given input file 10 times with random mutations
# Goal: trigger crashes or edge cases in binaries
# Output: crash files saved as `crash_*.bin` if the program crashes

include "fuzzing/fuzzer.my"

# 🔁 Fuzz this sample input file
fuzz_file("samples/input.png")
