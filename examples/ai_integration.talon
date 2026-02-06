# AI Integration Example
# Demonstrates AI-assisted features throughout TALON

# 1. AI-Powered General Help in REPL
# Usage: ai("how do I build a ROP chain?")
let help_response = ai("explain how to use format string exploits")
print("[AI HELP]", help_response)

# 2. AI Error Explanation (automatic with --explain-errors flag)
# When running with: talon run --explain-errors script.talon
# Errors will automatically include AI explanations

# 3. AI ROP Chain Suggestions
# The ai_integration layer provides suggest_rop_chain() for advanced usage
# This is integrated with rop_tools for automatic strategy suggestions

# 4. AI Shellcode Optimization
# The ai_integration layer provides optimize_shellcode() for constraint-aware generation
# Usage in Python-like pseudocode:
# ai.optimize_shellcode("execve /bin/sh", "avoid: [0x00, 0x0a], max_size: 64")

# 5. AI Exploit Review
# Use talon audit --ai <binary> for AI-powered vulnerability analysis
# Or review_exploit_ai(script_path) programmatically

# 6. AI Auto-Fix
# Use talon fix --ai <script> to automatically fix common errors
# Powered by ai_integration.fix_script()

# 7. AI Documentation Generator
# Use talon document --ai <script> to generate inline documentation
# Powered by ai_integration.generate_documentation()

# 8. AI Tutorial Hints
# The talon learn command now has AI-powered hints
# Powered by ai_integration.tutorial_hint()

# Example: Using AI in exploitation workflow
print("[AI DEMO] Starting AI-assisted exploitation workflow")

# Check if AI is available
let ai_query = ai("What are the main steps in a buffer overflow exploit?")
print("[AI] Response:", ai_query)

# AI features are optional and gracefully degrade
# If LM Studio is not running or AI is disabled, features show helpful messages

# Token Budget Management
# AI integration includes automatic token budgeting (100,000 tokens/hour by default)
# Prevents excessive API calls

# Caching System
# Repeated queries are cached for 1 hour (1000 entry cache)
# Example: Same error explanation requested twice uses cached response

# Configuration
# AI backend configured via:
# 1. Environment variables: TALON_AI_BACKEND, TALON_LM_STUDIO_ENDPOINT
# 2. Config file: ~/.talon/config.toml
# 3. Default: LM Studio at http://10.5.0.2:1234/v1

# Example config.toml:
# [ai]
# backend = "lm_studio"  # or "local_gguf" or "disabled"
# endpoint = "http://10.5.0.2:1234/v1"
# model = "deepseek-coder-6.7b-instruct"
# temperature = 0.7
# max_tokens = 2048

print("[AI DEMO] AI integration features ready")
print("[AI DEMO] Use --no-ai flag to disable all AI features")
print("[AI DEMO] Use --explain-errors flag for AI-powered error explanations")
