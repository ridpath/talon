#  Format string write attack (to overwrite memory or leak)
# Assumes binary has vulnerable printf(user_input)

include "exploit/format_string.my"

# Find offset dynamically
find_format_offset("bin/format0")

# Use format string payload to overwrite memory (example only)
format_string_exploit("bin/format0")
