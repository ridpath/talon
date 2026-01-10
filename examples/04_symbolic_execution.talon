# Symbolic Execution with Z3 Solver
# Automatically find input that reaches a specific code path

print("[*] Symbolic Execution Example")
print("=" * 60)

let binary = "./crackme"
let target_address = 0x401337  # Address of win() function

print("[+] Binary: " + binary)
print("[+] Target address: " + hex(target_address))

# Step 1: Initialize symbolic execution engine
print("\n[*] Step 1: Initializing symbolic execution...")
symbolic {
    binary: binary
    entry: "main"
    avoid: [0x401234, 0x401300]  # Addresses to avoid (fail paths)
    find: target_address         # Address to reach (win path)
    timeout: 30                   # seconds
}

print("    [+] Symbolic engine initialized")

# Step 2: Define symbolic variables
print("\n[*] Step 2: Creating symbolic input...")
let input_length = 32
let symbolic_input = symbolic_bytes(input_length)

print("    [+] Created " + str(input_length) + " bytes of symbolic input")

# Step 3: Execute symbolically
print("\n[*] Step 3: Exploring paths...")
let result = symbolic_execute(binary, symbolic_input)

if result.found
    print("    [+] Solution found!")
    print("    Explored " + str(result.paths_explored) + " paths")
    print("    Time: " + str(result.execution_time) + "s")
    
    # Step 4: Get concrete input that reaches target
    print("\n[*] Step 4: Extracting solution...")
    let solution = result.concrete_input
    
    print("    Solution (hex): " + hexlify(solution))
    print("    Solution (ascii): " + ascii_safe(solution))
    
    # Step 5: Verify solution
    print("\n[*] Step 5: Verifying solution...")
    let verify_session = run_binary(binary)
    send(verify_session, solution)
    let output = recv(verify_session, 1024)
    
    if "win" in output or "flag" in output
        print("    [+] SUCCESS! Solution verified!")
        print("    Output: " + output)
    else
        print("    [-] Verification failed")
    end
else
    print("    [-] No solution found")
    print("    Explored " + str(result.paths_explored) + " paths")
    print("    Constraints: " + str(result.unsolvable_constraints))
end

print("\n[+] Symbolic execution complete!")
