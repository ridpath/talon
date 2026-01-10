# TALON Examples

This directory contains comprehensive examples demonstrating TALON's capabilities for exploit development, binary analysis, and security research.

## Examples Overview

### Basic Exploitation

1. **01_buffer_overflow_rop.talon** - Classic buffer overflow with ROP chain
   - Demonstrates cyclic pattern generation
   - Libc base address leaking
   - ROP chain construction
   - Shell spawning

2. **02_format_string_attack.talon** - Format string arbitrary write
   - Format string offset detection
   - GOT overwrite technique
   - Multi-byte payload construction
   - Function pointer hijacking

3. **05_heap_exploitation.talon** - Modern heap exploitation
   - Tcache poisoning technique
   - Heap grooming strategies
   - Use-after-free exploitation
   - Arbitrary allocation attacks

### Advanced Features

4. **03_ai_powered_exploitation.talon** - AI-driven exploit generation
   - Automated vulnerability analysis
   - AI exploit suggestion system
   - Code generation workflow
   - Integration with OpenAI/Anthropic

5. **04_symbolic_execution.talon** - Automated path finding
   - Z3 constraint solving
   - Automatic input generation
   - Path exploration strategies
   - CrackMe solving

6. **06_ctf_automation.talon** - Full CTF challenge automation
   - Binary download and analysis
   - Local testing workflow
   - Remote exploitation
   - Flag extraction and submission

## Usage

### Running Examples Locally

```bash
# Run a specific example
talon run examples/01_buffer_overflow_rop.talon

# With custom parameters
talon run examples/03_ai_powered_exploitation.talon --binary ./custom_target

# Interactive mode
talon --interactive examples/04_symbolic_execution.talon
```

### Using AI Features

```bash
# Get AI-powered exploit suggestions
talon suggest ./binary

# Generate exploit code from suggestion #1
talon suggest ./binary --generate 1

# Use OpenAI for enhanced analysis
talon suggest ./binary --ai sk-YOUR-API-KEY

# Use Anthropic Claude
talon suggest ./binary --ai YOUR-ANTHROPIC-KEY --provider anthropic
```

### Modifying Examples

All examples are designed to be educational and modifiable. Key sections to customize:

- **Target configuration**: Change `target_host`, `target_port`, `binary_path`
- **Exploit parameters**: Modify offsets, addresses, payload sizes
- **Attack strategies**: Swap ROP chains, shellcode, techniques

## Best Practices

1. **Always test locally first** - Use local binaries before attacking remote servers
2. **Understand protections** - Run `checksec` to identify mitigations
3. **Use AI suggestions** - Leverage TALON's AI to discover attack vectors
4. **Iterate and debug** - Use TALON's interactive mode for troubleshooting

## Learning Path

### Beginner
1. Start with `01_buffer_overflow_rop.talon`
2. Understand `02_format_string_attack.talon`
3. Try `06_ctf_automation.talon` on easy CTF challenges

### Intermediate
1. Study `05_heap_exploitation.talon` for modern techniques
2. Experiment with `03_ai_powered_exploitation.talon`
3. Combine techniques from multiple examples

### Advanced
1. Master `04_symbolic_execution.talon` for complex challenges
2. Create custom exploitation workflows
3. Contribute new examples to the repository

## Requirements

- TALON framework installed
- Target binaries (provided or user-supplied)
- Network access for remote examples
- OpenAI/Anthropic API key for AI features (optional)

## Contributing

To add new examples:
1. Follow the existing naming convention (##_description.talon)
2. Include comprehensive comments
3. Add entry to this README
4. Test thoroughly before submitting

## Security Notice

These examples are for educational and authorized security testing only. Always:
- Obtain proper authorization before testing
- Use in controlled environments
- Follow responsible disclosure practices
- Comply with applicable laws and regulations


