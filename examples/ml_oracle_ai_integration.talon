# ML Oracle - AI-Assisted Vulnerability Analysis with LM Studio Integration
# Demonstrates dual-mode support: LM Studio HTTP API or local GGUF models
# Graceful fallback: LM Studio → Local GGUF → Disable AI

# Configuration Example (~/.talon/config.toml):
# [ai]
# backend = "lm_studio"  # or "local_gguf" or "disabled"
# endpoint = "http://10.5.0.2:1234/v1"
# model = "deepseek-coder-6.7b-instruct"
# temperature = 0.7
# max_tokens = 2048

# Environment Variable Configuration:
# TALON_AI_BACKEND=lm_studio
# TALON_LM_STUDIO_ENDPOINT=http://10.5.0.2:1234/v1

# ============================================================
# Example 1: AI-Assisted Binary Audit
# ============================================================
# Command: talon audit --ai /path/to/binary
#
# The ML Oracle will:
# 1. Run heuristic vulnerability detection
# 2. Enhance findings with AI analysis
# 3. Provide exploit strategies and gadget suggestions
# 4. Assess risk and recommend techniques

# ============================================================
# Example 2: Error Explanation with AI
# ============================================================
# Command: talon explain --ai "Connection refused"
#
# The ML Oracle will provide:
# 1. Root cause analysis
# 2. Common reasons for the error
# 3. Step-by-step fix instructions
# 4. Example corrected code

# ============================================================
# Example 3: Code Review and Suggestions
# ============================================================
# Command: talon suggest --ai script.talon
#
# The ML Oracle will analyze:
# 1. Security considerations
# 2. Reliability improvements
# 3. Code quality suggestions
# 4. Performance optimizations
# 5. Edge cases to handle

# ============================================================
# Example 4: Programmatic ML Oracle Usage
# ============================================================

# Simple binary analysis example
let binary = "/path/to/vulnerable_binary"

# AI-assisted vulnerability detection would integrate here
# (Future DSL integration for direct ML Oracle access)

# ============================================================
# Example 5: LM Studio Integration Points
# ============================================================

# LM Studio must be running at http://10.5.0.2:1234 (WSL setup)
# or configured via ~/.talon/config.toml or environment variables

# Key Features:
# - Automatic LM Studio detection
# - Fallback to local GGUF models
# - Graceful degradation (disable AI if unavailable)
# - OpenAI-compatible API format
# - Context management (max 8K tokens)
# - Streaming inference for progress indication

# ============================================================
# Example 6: Recommended Models
# ============================================================

# Primary: DeepSeek-Coder 6.7B Instruct
#   - Best balance of size/performance
#   - Excellent for vulnerability analysis
#   - Q4_K_M quantization recommended

# Secondary: CodeLlama 13B Instruct
#   - Larger context window
#   - Better for complex exploit generation
#   - Q5_K_M quantization recommended

# Tertiary: Llama 3 8B Instruct
#   - General purpose
#   - Good for error explanations
#   - Q4_K_M quantization recommended

# ============================================================
# Example 7: Use Cases
# ============================================================

# Use Case 1: Binary Analysis
# Feed disassembly + heuristics → get logic flaw suggestions

# Use Case 2: Exploit Generation
# Feed vulnerability report → get .talon script skeleton

# Use Case 3: Gadget Discovery
# Feed binary sections → get ROP chain suggestions

# Use Case 4: Error Explanation
# Feed error message + source → get human-readable explanation

# Use Case 5: Code Review
# Feed .talon script → get optimization/security suggestions

# Use Case 6: Shellcode Optimization
# Feed shellcode constraints → get optimized variants

print("ML Oracle example - see comments for usage patterns")
print("Run: talon audit --ai /path/to/binary")
print("Run: talon explain --ai 'error message'")
print("Run: talon suggest --ai script.talon")
