# ═══════════════════════════════════════════════════════════════
# TALON TUTORIAL 1: BASICS
# Learn the fundamentals of Talon DSL
# ═══════════════════════════════════════════════════════════════

print("=== TALON Tutorial 1: Basics ===")
print("")

# LESSON 1: Variables and Types
# ────────────────────────────────────────────────────────────────

print("LESSON 1: Variables and Types")

# Simple variable declaration
let greeting = "Hello, Talon!"
print("  Variable declared: greeting")

# Variable with type hint (recommended)
let port: int = 8080
let target: string = "192.168.1.100"
print("  Type hints used for port and target")

# Constants (cannot be changed)
const MAX_RETRIES = 3
print("  Constant MAX_RETRIES = 3")
print("")

# LESSON 2: Control Flow
# ────────────────────────────────────────────────────────────────

print("LESSON 2: Control Flow")

# If-else statement
let status = 200

if status == 200 {
    print("  Status check passed (200)")
} else {
    print("  Status check failed")
}
print("")

# LESSON 3: Loops
# ────────────────────────────────────────────────────────────────

print("LESSON 3: Loops")
print("  For loop with range 1..5:")

# For loop with range
for i in 1..5 {
    print("    Loop iteration")
}
print("")

# LESSON 4: Data Structures
# ────────────────────────────────────────────────────────────────

print("LESSON 4: Data Structures")

# Lists
let exploit_ports = [21, 22, 23, 80, 443]
print("  List created with 5 ports")

# Note: Maps and Sets are available for advanced usage
print("  Maps and Sets available for advanced usage")
print("")

# LESSON 5: String Operations
# ────────────────────────────────────────────────────────────────

print("LESSON 5: String Operations")

# Basic strings
let target_ip = "192.168.1.100"
let target_port = "8080"
print("  String variables: target_ip and target_port")
print("")

# LESSON 6: Best Practices
# ────────────────────────────────────────────────────────────────

print("LESSON 6: Best Practices")
print("  1. Use type hints for parameters")
print("  2. Use descriptive variable names")
print("  3. Use constants for magic numbers")
print("  4. Comment your code thoroughly")
print("")

# Define constants for demonstration
const DEFAULT_TIMEOUT = 30
const MAX_PAYLOAD_SIZE = 4096
print("  Constants: DEFAULT_TIMEOUT=30, MAX_PAYLOAD_SIZE=4096")
print("")

# ═══════════════════════════════════════════════════════════════
#  CONGRATULATIONS!
# ═══════════════════════════════════════════════════════════════

print("=== CONGRATULATIONS! ===")
print("You have completed Tutorial 1: Basics")
print("")
print("Next steps:")
print("  - Try tutorial_02_exploitation.talon")
print("  - Experiment in the REPL: talon repl")
print("  - Check the documentation: talon help")
print("")
print("Tutorial 1 completed successfully!")
