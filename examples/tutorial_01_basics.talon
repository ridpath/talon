# ═══════════════════════════════════════════════════════════════
# TALON TUTORIAL 1: BASICS
# Learn the fundamentals of Talon DSL
# ═══════════════════════════════════════════════════════════════

# LESSON 1: Variables and Types
# ────────────────────────────────────────────────────────────────

# Simple variable declaration
let greeting = "Hello, Talon!"

# Variable with type hint (recommended)
let port: int = 8080
let target: string = "192.168.1.100"

# Constants (cannot be changed)
const MAX_RETRIES = 3

# LESSON 2: Functions
# ────────────────────────────────────────────────────────────────

# Define a simple function
define function greet(name) {
    return "Hello, {name}!"
}

# Call the function
let message = greet("Hacker")

# Function with multiple parameters and type hints
define function connect_to_target(ip: string, port: int) {
    connect to ip on port port
    return "Connected!"
}

# LESSON 3: Control Flow
# ────────────────────────────────────────────────────────────────

# If-else statement
let status = 200

if status == 200 {
    let result = "Success"
} else {
    let result = "Failed"
}

# For loop with range
for i in 1..10 {
    # This will iterate from 1 to 9
}

# For loop with list
let ports = [80, 443, 8080, 8443]
for port in ports {
    connect_to_target("localhost", port)
}

# LESSON 4: Data Structures
# ────────────────────────────────────────────────────────────────

# Lists
let exploit_ports = [21, 22, 23, 80, 443]

# Note: Maps and Sets are available for advanced usage
# Consult documentation for syntax details

# LESSON 5: String Operations
# ────────────────────────────────────────────────────────────────

# Basic strings
let target_ip = "192.168.1.100"
let target_port = "8080"
let conn_str = "Connecting to " + target_ip + ":" + target_port

# Note: String concatenation uses the + operator

# LESSON 6: Comments
# ────────────────────────────────────────────────────────────────

# This is a single-line comment

# TIP: Use descriptive comments for better code readability
# Success case
# Error case
# Important note
# Security consideration

# LESSON 7: Best Practices
# ────────────────────────────────────────────────────────────────

# 1. Always use type hints for function parameters
define function safe_connect(ip: string, port: int): string {
    # Connection logic would go here
    return "Connected"
}

# 2. Use descriptive variable names
let target_ip = "192.168.1.100"  # Good
let x = "192.168.1.100"          # Bad

# 3. Comment your code  
# Check if port is open before exploiting
# let conn = connect(target_ip, 22)

# 4. Use constants for magic numbers
const DEFAULT_TIMEOUT = 30
const MAX_PAYLOAD_SIZE = 4096

# LESSON 8: Error Handling
# ────────────────────────────────────────────────────────────────

# Try-catch for error handling (conceptual)
# Error handling syntax may vary
print("Error handling example completed")

# ═══════════════════════════════════════════════════════════════
#  CONGRATULATIONS!
# You've completed Tutorial 1: Basics
# 
# Next steps:
#   - Try tutorial_02_exploitation.talon
#   - Experiment in the REPL: talon repl
#   - Check the cheatsheet: talon cheatsheet
# ═══════════════════════════════════════════════════════════════
