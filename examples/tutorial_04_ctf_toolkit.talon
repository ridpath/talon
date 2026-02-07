# ═══════════════════════════════════════════════════════════════
# TALON TUTORIAL 4: CTF COMPLETE TOOLKIT
# Everything you need to dominate CTF competitions
# ═══════════════════════════════════════════════════════════════

# PREREQUISITE: Complete tutorials 01, 02, and 03 first!

include "stdlib/quick.talon"

# LESSON 1: CTF Challenge Identification
# ────────────────────────────────────────────────────────────────

define function identify_challenge_type()
    # Automatically identify what type of CTF challenge you're dealing with
    let challenge_file = "mystery_file.bin"
    
    # Use Talon's auto-identifier
    auto_identify_challenge(challenge_file)
    
    # Manual identification
    # - PE/ELF = Binary exploitation / Reverse engineering
    # - Image files = Steganography
    # - Audio = Audio steganography
    # - ZIP/archives = Forensics
    # - HTML/PHP = Web exploitation
    # - Encrypted files = Cryptography
end

# LESSON 2: Flag Finding & Pattern Recognition
# ────────────────────────────────────────────────────────────────

define function flag_hunting()
    # Search for common flag formats
    let file = "challenge.txt"
    find_flags(file)
    
    # Common flag patterns:
    # - flag{...}
    # - FLAG{...}
    # - HTB{...}
    # - picoCTF{...}
    # - CTF{...}
    # - MD5 hashes (32 hex chars)
    # - Base64 strings
    
    # Search in binaries
    find_flags("binary_challenge")
    
    # Search in network captures
    find_flags("capture.pcap")
end

# LESSON 3: Cryptography Challenges
# ────────────────────────────────────────────────────────────────

define function crypto_challenges()
    # Step 1: Identify hash type
    let mysterious_hash = "5d41402abc4b2a76b9719d911017c592"
    identify_hash(mysterious_hash)
    
    # Step 2: Crack the hash
    crack_hash(mysterious_hash, "rockyou.txt")
    
    # Step 3: Try common ciphers
    let encrypted = "Uryyb, Jbeyq!"
    let decrypted = rot13(encrypted)  # Caesar/ROT13
    
    # Step 4: XOR brute force
    let xor_encrypted = [0x48, 0x45, 0x4C, 0x4C, 0x4F]
    for key in 0..255
        let result = xor_bytes(xor_encrypted, key)
        # Check if result makes sense
    end
    
    # Step 5: RSA attacks
    # - Small e attack
    # - Common modulus
    # - Wiener's attack
    # - Factor weak primes
end

# LESSON 4: Steganography Challenges
# ────────────────────────────────────────────────────────────────

define function stego_challenges()
    # LSB extraction from images
    let image = "suspicious_image.png"
    extract_lsb(image)
    
    # Find hidden files
    find_hidden_files(image)
    
    # EXIF data extraction
    extract_gps(image)
    
    # Audio steganography
    # - Spectral analysis
    # - LSB in audio samples
    # - Hidden messages in frequency spectrum
    
    # Check for appended files
    # Use binwalk or manual hex analysis
end

# LESSON 5: Forensics Challenges
# ────────────────────────────────────────────────────────────────

define function forensics_challenges()
    # File carving from disk images
    let disk_image = "forensics.img"
    carve_files(disk_image, "carved_output/")
    
    # Timeline analysis
    analyze_timeline("/evidence/")
    
    # String extraction
    extract_strings_min("memory_dump.raw", 6)
    
    # PCAP analysis
    # - Extract HTTP objects
    # - Follow TCP streams
    # - Look for credentials
    
    # Memory forensics
    # - Volatility framework
    # - Process listing
    # - Network connections
end

# LESSON 6: OSINT Challenges
# ────────────────────────────────────────────────────────────────

define function osint_challenges()
    # Subdomain enumeration
    enumerate_subdomains("target.com")
    
    # Email harvesting
    harvest_emails("https://target.com")
    
    # WHOIS lookup
    whois_lookup("target.com")
    
    # Username OSINT
    search_username("target_user")
    
    # GPS coordinates from images
    extract_gps("photo.jpg")
    
    # Social media investigations
    # - LinkedIn profiles
    # - Twitter handles
    # - GitHub repos
    # - Profile reuse across platforms
end

# LESSON 7: Reverse Engineering Challenges
# ────────────────────────────────────────────────────────────────

define function reverse_engineering()
    # Step 1: File analysis
    # file binary
    # strings binary | grep flag
    
    # Step 2: Disassembly
    # objdump -d binary
    # ghidra binary
    
    # Step 3: Dynamic analysis
    # gdb binary
    # ltrace binary
    # strace binary
    
    # Step 4: Decompilation
    # Use Ghidra's decompiler
    # IDA Pro / Hex-Rays
    
    # Step 5: Binary patching
    let binary = "crackme"
    let offset = 0x1234
    let nop_bytes = [0x90, 0x90, 0x90]
    patch_binary(binary, offset, nop_bytes, "patched_binary")
end

# LESSON 8: Encoding/Decoding Challenges
# ────────────────────────────────────────────────────────────────

define function encoding_challenges()
    # Try all common encodings
    let encoded_data = "SGVsbG8gV29ybGQh"
    try_all_decodings(encoded_data)
    
    # Specific decodings
    let base64_data = "VGFsb24gRFNM"
    b64_decode(base64_data)
    
    let base32_data = "KRUGKIDROVUWG2ZAMJZG653OEBTG66BANJ2W24DTEBXXMZLSE"
    quick_b32_encode(base32_data)
    
    # URL encoding
    let url_encoded = "%48%65%6C%6C%6F"
    quick_url_encode(url_encoded)
    
    # Morse code
    let morse = ".... . .-.. .-.. ---"
    # Decode morse
    
    # Binary
    let binary = "01010100 01100001 01101100 01101111 01101110"
    # Convert from binary
end

# LESSON 9: Archive & File Format Challenges
# ────────────────────────────────────────────────────────────────

define function archive_challenges()
    # Auto-extract any archive
    auto_extract("challenge.zip", "output/")
    
    # Password-protected archives
    crack_zip_password("protected.zip", "rockyou.txt")
    
    # Nested archives
    # Keep extracting until you find the flag!
    
    # Corrupted archives
    # Fix magic bytes
    # Repair CRC checksums
    
    # Hidden data in archives
    # Check for extra data after ZIP end marker
end

# LESSON 10: Binary Exploitation (PWN) Challenges
# ────────────────────────────────────────────────────────────────

define function pwn_challenges()
    # Step 1: Checksec
    # checksec binary
    
    # Step 2: Find offset
    # pattern_create 200
    # Run binary with pattern
    # pattern_offset <crash_address>
    
    # Step 3: Build exploit
    let offset = 264
    let ret_addr = 0xdeadbeef
    let payload = overflow_payload(offset, ret_addr, "shellcode")
    
    # Step 4: ROP chain
    let libc_base = 0x7ffff7a0d000
    let binsh = libc_base + 0x1b3e1a
    let rop = ret2libc(libc_base, binsh)
    
    # Step 5: Execute
    # python exploit.py
end

# LESSON 11: Complete CTF Workflow
# ────────────────────────────────────────────────────────────────

define function complete_ctf_workflow()
    # Phase 1: Initial analysis
    let challenge = "unknown_challenge"
    auto_identify_challenge(challenge)
    
    # Phase 2: Quick checks
    find_flags(challenge)
    
    # Phase 3: Category-specific approach
    # Based on challenge type:
    # - Binary → Reverse/Pwn
    # - Image → Stego
    # - Web → Web exploits
    # - Hash → Crypto
    # - Archive → Forensics
    
    # Phase 4: Exploitation
    # Use appropriate technique
    
    # Phase 5: Flag extraction
    # Clean up and submit
end

# LESSON 12: CTF Cheatsheet Usage
# ────────────────────────────────────────────────────────────────

define function use_cheatsheets()
    # Show category-specific cheatsheet
    show_ctf_cheatsheet("pwn")
    show_ctf_cheatsheet("web")
    show_ctf_cheatsheet("crypto")
    show_ctf_cheatsheet("forensics")
    
    # Quick reference for commands, techniques, and tools
end

# LESSON 13: Network Challenges (PCAP Analysis)
# ────────────────────────────────────────────────────────────────

define function network_challenges()
    # Analyze PCAP files
    # - Extract HTTP objects
    # - Follow TCP streams
    # - Find credentials
    # - Reconstruct files
    
    # Common tools:
    # - Wireshark
    # - tshark
    # - tcpdump
    # - NetworkMiner
    
    # Look for:
    # - FTP credentials (plaintext)
    # - HTTP POST data
    # - DNS queries
    # - Suspicious traffic patterns
end

# LESSON 14: Miscellaneous Challenges
# ────────────────────────────────────────────────────────────────

define function misc_challenges()
    # QR codes
    # - Scan and decode QR codes
    
    # Barcodes
    # - EAN, Code128, etc.
    
    # Esoteric programming languages
    # - Brainfuck
    # - Malbolge
    # - Whitespace
    
    # Weird formats
    # - CHIP-8
    # - Game Boy ROMs
    # - Custom file formats
end

# LESSON 15: Time Management & Strategy
# ────────────────────────────────────────────────────────────────

define function ctf_strategy()
    # 1. Read ALL challenges first
    # 2. Start with your strengths
    # 3. Pick low-hanging fruit (easy challenges)
    # 4. Don't get stuck - move on if stuck for 30+ min
    # 5. Collaborate with team
    # 6. Document everything
    # 7. Time-box each challenge
    # 8. Return to hard challenges with fresh perspective
end

# HANDS-ON CTF SIMULATION
# ────────────────────────────────────────────────────────────────

define function ctf_simulation()
    # Challenge 1: Hidden Flag
    let mystery = "ZmxhZ3tTdGVnYW5vZ3JhcGh5X0Z1bn0="
    # Decode this!
    
    # Challenge 2: Hash Crack
    let hash = "5d41402abc4b2a76b9719d911017c592"
    identify_hash(hash)
    # Crack it!
    
    # Challenge 3: Find the Flag
    let binary = "challenge.bin"
    find_flags(binary)
    
    # Challenge 4: OSINT
    # Find information about user "ctf_player_2024"
    search_username("ctf_player_2024")
end

# RECOMMENDED CTF PLATFORMS
# ────────────────────────────────────────────────────────────────
# - HackTheBox (https://hackthebox.com)
# - TryHackMe (https://tryhackme.com)
# - PicoCTF (https://picoctf.org)
# - OverTheWire (https://overthewire.org)
# - CTFtime.org (find live competitions)
# - CryptoHack (https://cryptohack.org)
# - pwnable.kr (binary exploitation)
# - RingZer0 CTF (https://ringzer0ctf.com)

# ESSENTIAL TOOLS LIST
# ────────────────────────────────────────────────────────────────
# Binary:
# - Ghidra, IDA Pro, Binary Ninja (disassemblers)
# - GDB, PWNDBG (debuggers)
# - ROPgadget, ropper (ROP tools)
# - pwntools (Python exploitation framework)
#
# Web:
# - Burp Suite, OWASP ZAP (web proxies)
# - SQLMap (SQL injection)
# - Gobuster, Dirb (directory enumeration)
# - Nikto (web scanner)
#
# Crypto:
# - John the Ripper, Hashcat (password cracking)
# - RsaCtfTool (RSA attacks)
# - CyberChef (encoding/decoding)
#
# Forensics:
# - Autopsy, FTK Imager (disk forensics)
# - Volatility (memory forensics)
# - Wireshark (network forensics)
# - Binwalk, Foremost (file carving)
#
# Stego:
# - Steghide, Stegsolve (image stego)
# - zsteg (PNG/BMP LSB)
# - Sonic Visualizer (audio spectrograms)
# - exiftool (metadata)
#
# OSINT:
# - Sherlock (username search)
# - theHarvester (email/subdomain enum)
# - Shodan (IoT/exposed services)
# - Maltego (OSINT framework)

#  PRO TIPS
# ────────────────────────────────────────────────────────────────
# 1. Always start with 'file', 'strings', 'binwalk'
# 2. Keep a collection of wordlists (rockyou.txt is essential)
# 3. Script repetitive tasks (that's what Talon is for!)
# 4. Join CTF Discord servers for hints and community
# 5. Write writeups after solving challenges (solidifies learning)
# 6. Practice regularly - consistency beats cramming
# 7. Specialize in 2-3 categories, but know basics of all
# 8. Time management is KEY in Jeopardy-style CTFs
# 9. In Attack-Defense CTFs, patch vulnerabilities first!
# 10. Have fun and never stop learning!

#  CONTINUE YOUR JOURNEY
# ────────────────────────────────────────────────────────────────
# You now have a complete CTF toolkit in Talon DSL!
# - All 10 new modules are at your fingertips
# - Web exploitation, crypto, stego, forensics, OSINT, and more
# - Production-ready code for real competitions
# - User-friendly helpers and cheatsheets
#
# Go forth and capture those flags! 
