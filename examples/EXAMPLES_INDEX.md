# TALON Examples Index

Comprehensive guide to all TALON example files organized by topic.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Binary Exploitation](#binary-exploitation)
3. [Network Exploitation](#network-exploitation)
4. [Format String Attacks](#format-string-attacks)
5. [Heap Exploitation](#heap-exploitation)
6. [ROP Techniques](#rop-techniques)
7. [Shellcode Development](#shellcode-development)
8. [SSH & Remote Access](#ssh--remote-access)
9. [Debugging & Analysis](#debugging--analysis)
10. [CTF Automation](#ctf-automation)
11. [Advanced Features](#advanced-features)
12. [OpSec & Evasion](#opsec--evasion)
13. [Distributed Operations](#distributed-operations)
14. [AI Integration](#ai-integration)

---

## Getting Started

### Basic Examples
- **`00_demo_without_binary.talon`** - Introduction to TALON without requiring a vulnerable binary
- **`01_basic_overflow.talon`** - Simple buffer overflow demonstration
- **`tutorial_01_basics.talon`** - TALON fundamentals and syntax
- **`beginner_ctf_template.talon`** - Template for CTF challenges

### Tutorials
- **`tutorial_02_exploitation.talon`** - Binary exploitation basics
- **`tutorial_03_web_exploitation.talon`** - Web application attacks
- **`tutorial_04_ctf_toolkit.talon`** - CTF-specific tools and techniques

---

## Binary Exploitation

### Buffer Overflows
- **`01_basic_overflow.talon`** - Classic stack buffer overflow
- **`01_buffer_overflow_rop.talon`** - Buffer overflow with ROP chain
- **`exploit_chain_buffer_overflow.talon`** - Complete exploitation chain

### Return-Oriented Programming
- **`02_rop_libc_attack.talon`** - ret2libc attack
- **`rop_exploitation_techniques.talon`** - ROP exploitation techniques
- **`rop_dsl_showcase.talon`** - ROP DSL features
- **`ctf_ret2libc_pwn.talon`** - CTF-style ret2libc challenge
- **`ctf_blind_rop.talon`** - Blind ROP exploitation

### SROP (Sigreturn-Oriented Programming)
- **`ctf_quick_exploitation.talon`** - Quick SROP demonstration

### Comprehensive Examples
- **`complete_exploitation_workflow.talon`** - Complete modern exploitation workflow
- **`ultimate_exploit_combo.talon`** - Multi-stage attack combining multiple techniques

---

## Network Exploitation

### Connection Handling
- **`ssh_exploitation.talon`** - Comprehensive SSH exploitation examples
- **`mass_exploitation.talon`** - Concurrent connection handling

### Port Forwarding & Pivoting
- Covered in `ssh_exploitation.talon` (Examples 4, 7)

---

## Format String Attacks

- **`02_format_string_attack.talon`** - Basic format string exploitation
- **`format_string_techniques.talon`** - Format string exploitation techniques
- **`exploit_chain_format_string.talon`** - Format string exploitation chain
- **`ctf_format_string_leak_write.talon`** - CTF-style leak and write
- **`format_string_vuln.c`** - Vulnerable C program for testing

---

## Heap Exploitation

### Techniques
- **`05_heap_exploitation.talon`** - Heap exploitation fundamentals
- **`exploit_chain_heap_uaf.talon`** - Use-After-Free exploitation
- **`ctf_heap_tcache_poison.talon`** - Tcache poisoning attack
- **`heap_exploitation_techniques.rs`** - Heap exploitation techniques (Rust)

### Practice Targets
- **`heap_vuln.c`** - Vulnerable heap program for testing

---

## ROP Techniques

- **`rop_exploitation_techniques.talon`** - Complete ROP workflow
- **`rop_dsl_showcase.talon`** - ROP DSL syntax and features
- **`ctf_blind_rop.talon`** - Blind ROP (no binary access)
- **`ctf_one_gadget_pwn.talon`** - One-gadget exploitation

---

## Shellcode Development

- **`05_shellcode_injection.rs`** - Shellcode injection techniques (Rust)
- **`shellcode_generation_techniques.talon`** - Shellcode generation techniques
- **`ctf_shellcode_encoder.talon`** - Shellcode encoding/decoding
- **`polymorphic_shellcode.talon`** - Polymorphic shellcode generation

---

## SSH & Remote Access

### SSH Operations
- **`ssh_exploitation.talon`** - Comprehensive SSH examples:
  - Password authentication
  - Key-based authentication
  - Interactive PTY sessions
  - Port forwarding
  - File upload/download
  - Credential spraying
  - Tunneling
  - Constrained terminal exploitation (Bandit 26)

### OverTheWire Bandit
- **`otw/bandit/level26_*.talon`** - Multiple solutions for Bandit Level 26
- **`otw/bandit/level27_*.talon`** - Bandit Level 27 solutions
- **`otw/bandit/test_*.talon`** - Testing scripts for Bandit challenges

---

## Debugging & Analysis

### Time-Travel Debugging
- **`time_travel_debugging.talon`** - Comprehensive time-travel debugging examples:
  - Checkpoint creation and restoration
  - State rewinding
  - Multiple payload testing
  - Split-screen debugging
  - Reverse debugging
  - State diffing

### Orchestration & Analysis
- **`orchestrator_graph.talon`** - Orchestrated exploitation with dependency graphs
- **`orchestrator_parallel.talon`** - Parallel exploitation
- **`orchestrator_resilient.talon`** - Resilient exploitation with fallbacks
- **`orchestrator_timetravel.talon`** - Time-travel orchestration
- **`04_symbolic_execution.talon`** - Symbolic execution examples

---

## CTF Automation

### Challenge Templates
- **`beginner_ctf_template.talon`** - Beginner CTF template
- **`ctf_automation.talon`** - Automated CTF solving
- **`ctf_multi_stage_pwn.talon`** - Multi-stage CTF challenges

### Specific Challenge Types
- **`ctf_blind_rop.talon`** - Blind ROP challenges
- **`ctf_format_string_leak_write.talon`** - Format string challenges
- **`ctf_heap_tcache_poison.talon`** - Heap challenges
- **`ctf_kernel_exploit.talon`** - Kernel challenges
- **`ctf_one_gadget_pwn.talon`** - One-gadget challenges
- **`ctf_ret2libc_pwn.talon`** - ret2libc challenges
- **`ctf_shellcode_encoder.talon`** - Shellcode challenges
- **`new_ctf_functions_showcase.talon`** - New CTF functions

---

## Advanced Features

### Binary Patching
- **`binary_patching.talon`** - Semantic binary patching:
  - NOP out instructions
  - Replace function calls
  - Inject assembly code
  - Patch strings
  - Create code caves
  - Recalculate headers

### Symbolic Execution
- **`04_symbolic_execution.talon`** - Z3 constraint solving
- **`phase22_symbiotic_execution.talon`** - Advanced symbolic execution

### Oracle & Vulnerability Analysis
- Integrated in AI examples (see below)

---

## OpSec & Evasion

### Memory Operations
- **`memory_scrubbing.talon`** - Secure memory handling:
  - Auto-zeroing SecureString
  - Memory scrubbing
  - DPAPI integration (Windows)
  - Anti-debugging checks

### Evasion Techniques
- **`edr_bypass_syscalls.talon`** - EDR evasion with indirect syscalls
- **`polymorphic_shellcode.talon`** - Polymorphic code generation
- **`artifact_less_execution.talon`** - Artifact-less execution:
  - memfd_create (Linux)
  - Reflective DLL injection (Windows)
  - Process hollowing
  - VM/container detection

### Production Deployment
- **`production_error_obfuscation.talon`** - Cryptographic error obfuscation

---

## Distributed Operations

### Swarm Mode
- **`swarm_mass_exploit.talon`** - Mass exploitation with swarm
- **`swarm_mass_pwn.talon`** - Distributed pwn operations
- **`swarm_subnet_scan.talon`** - Distributed network scanning
- **`swarm_libc_leak.talon`** - Distributed libc detection
- **`swarm_agent_deployment.talon`** - Agent deployment workflows

### Configuration
- **`inventory.ini`** - Agent inventory (50+ example agents)
- **`swarm_agent_config.json`** - Agent configuration

---

## AI Integration

### Local LLM Integration
- **`ai_integration.talon`** - Comprehensive AI integration:
  - Binary analysis with logic flaw detection
  - Exploit generation from vulnerability reports
  - ROP chain strategy suggestions
  - Error explanation with context
  - Code review and optimization
  - Shellcode constraint solving
  - REPL inline assistance
  - Auto-fix for common mistakes

### ML-Powered Features
- **`ml_oracle_ai_integration.talon`** - ML Oracle examples:
  - Vulnerability analysis
  - Automatic exploit generation
  - LM Studio integration
  - Local GGUF support

### AI-Powered Exploitation
- **`03_ai_powered_exploitation.talon`** - AI-assisted exploitation

---

## Natural Language Examples

- **`natural_language_examples.talon`** - Natural language DSL examples
- **`phase21_meta_programming.talon`** - Meta-programming features
- **`phase22_demo.talon`** - Phase 22 feature demonstration

---

## Rust Examples

For developers extending TALON or understanding internals:

- **`01_basic_pwn.rs`** - Basic exploitation in Rust
- **`02_heap_exploitation.rs`** - Heap exploitation in Rust
- **`03_srop_exploitation.rs`** - SROP in Rust
- **`04_format_string.rs`** - Format string in Rust
- **`05_shellcode_injection.rs`** - Shellcode injection in Rust
- **`heap_exploitation_techniques.rs`** - Heap exploitation in Rust

---

## Vulnerable Test Programs

Practice targets for testing exploits:

- **`vuln.c`** - General vulnerable binary
- **`vuln_binary`** - Compiled vulnerable binary
- **`format_string_vuln`** - Format string vulnerable binary
- **`format_string_vuln.c`** - Format string source
- **`heap_vuln`** - Heap vulnerable binary
- **`heap_vuln.c`** - Heap vulnerable source
- **`Makefile`** - Build script for test programs

---

## Documentation

- **`README.md`** - Examples overview
- **`README_EXPLOITATION.md`** - Exploitation guide
- **`SHOWCASE.md`** - Feature showcase
- **`EXAMPLES_INDEX.md`** - This file

---

## Usage Tips

### Running Examples

```bash
# Run in development mode (fast startup)
talon run --dev examples/01_basic_overflow.talon

# Run with dry-run (no actual I/O)
talon run --dry-run examples/ssh_exploitation.talon

# Run with AI assistance
talon run --ai examples/rop_exploitation_techniques.talon

# Run with error explanations
talon run --explain-errors examples/ctf_automation.talon
```

### Example Workflow

1. **Learn Basics**: Start with `tutorial_01_basics.talon`
2. **Simple Exploits**: Try `01_basic_overflow.talon`
3. **Exploitation Techniques**: Progress to `rop_exploitation_techniques.talon`
4. **Real Challenges**: Use `otw/bandit/` for real SSH challenges
5. **Production**: Apply `production_error_obfuscation.talon` for deployment

### CTF Workflow

1. Start with `beginner_ctf_template.talon`
2. Use category-specific examples (heap, ROP, format string)
3. Enable AI assistance: `talon audit --ai <binary>`
4. Use time-travel debugging for complex exploits
5. Leverage swarm mode for multi-target challenges

### Red Team Workflow

1. Use `artifact_less_execution.talon` for stealth
2. Apply `edr_bypass_syscalls.talon` for evasion
3. Use `memory_scrubbing.talon` for OpSec
4. Deploy with `production_error_obfuscation.talon`
5. Scale with swarm mode examples

---

## Feature Coverage

All major TALON features are covered:

- ✓ Buffer Overflows
- ✓ ROP Chains (including Blind ROP, SROP)
- ✓ Format String Exploits
- ✓ Heap Exploitation (UAF, Tcache, Fastbin)
- ✓ Shellcode Development & Encoding
- ✓ SSH Operations & PTY Handling
- ✓ Time-Travel Debugging
- ✓ Binary Patching
- ✓ Symbolic Execution
- ✓ Oracle Vulnerability Analysis
- ✓ OpSec & Evasion (EDR Bypass, Memory Scrubbing)
- ✓ Artifact-less Execution
- ✓ Polymorphic Code Generation
- ✓ Distributed Swarm Operations
- ✓ AI Integration (LM Studio, Local GGUF)
- ✓ CTF Automation
- ✓ Production Error Obfuscation

---

## Contributing

When adding new examples:

1. **Follow naming convention**: `category_description.talon`
2. **Add descriptive header**: Explain purpose and techniques
3. **Include inline comments**: Explain each step
4. **No emoticons**: Keep professional
5. **No marketing language**: Technical only
6. **Test examples**: Verify they run without errors
7. **Update this index**: Add entry in appropriate section

---

## Validation

Examples are validated for:

- No emoticons
- No marketing language
- Descriptive headers
- Consistent style
- Syntax correctness (when compiled)

Run validation:
```bash
powershell -ExecutionPolicy Bypass -File scripts/validate_examples.ps1
```

---

## Support

For questions or issues with examples:

1. Check TALON documentation
2. Review similar examples in this index
3. Use `talon help <function>` for builtin documentation
4. Enable AI assistance: `talon --ai`

---

**Total Examples**: 65+ files covering all TALON features
**Last Updated**: 2026-02-07
