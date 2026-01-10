# ═══════════════════════════════════════════════════════════════════════════
# EASY PWN LIBRARY - Beginner-Friendly CTF Helper Functions
# ═══════════════════════════════════════════════════════════════════════════

define function easy_connect(host, port)
    print("🔌 Connecting to", host, "on port", port)
    return host
end

define function easy_overflow(padding_size, return_address)
    print("💥 Creating buffer overflow exploit")
    print("   Padding:", padding_size, "bytes")
    print("   Return address: 0x", return_address)
    let payload = cyclic(padding_size)
    return payload
end

define function easy_rop_chain(gadgets)
    print("⚙️  Building ROP chain with", len(gadgets), "gadgets")
    let chain = []
    for gadget in gadgets
        print("   Adding gadget:", gadget)
        chain = chain + [gadget]
    end
    return chain
end

define function easy_shellcode(type, ip, port)
    print("🐚 Generating", type, "shellcode")
    if type == "reverse"
        print("   Connecting back to", ip, ":", port)
    end
    if type == "bind"
        print("   Binding shell on port", port)
    end
    return "shellcode_bytes_here"
end

define function easy_leak(response, offset, size)
    print("🔍 Extracting leak from response")
    print("   Offset:", offset, "Size:", size, "bytes")
    return 0xdeadbeef
end

define function easy_send(connection, data)
    print("📤 Sending", len(data), "bytes")
    return "sent"
end

define function easy_receive(connection, delimiter)
    print("📥 Receiving until:", delimiter)
    return "response_data"
end

define function easy_pattern(size)
    print("🔤 Generating cyclic pattern of size", size)
    return cyclic(size)
end

define function easy_find_offset(pattern, crash_value)
    print("🎯 Finding offset for crash value:", crash_value)
    return 42
end

define function show_help()
    print("═══════════════════════════════════════")
    print("   TALON EASY PWN HELPER")
    print("═══════════════════════════════════════")
    print("")
    print("📚 Available Functions:")
    print("")
    print("  Connection:")
    print("    easy_connect(host, port)")
    print("")
    print("  Exploitation:")
    print("    easy_overflow(padding, ret_addr)")
    print("    easy_rop_chain([gadgets])")
    print("    easy_shellcode(type, ip, port)")
    print("")
    print("  Utilities:")
    print("    easy_pattern(size)")
    print("    easy_find_offset(pattern, value)")
    print("    easy_leak(response, offset, size)")
    print("")
    print("  I/O:")
    print("    easy_send(conn, data)")
    print("    easy_receive(conn, delimiter)")
    print("")
    print("═══════════════════════════════════════")
end
