print("[*] Level 26 - Analyzing Initial Output")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connecting with 3x20 PTY...")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)

print("[*] Starting interactive...")
ssh_interactive_start(ssh)

print("[*] Getting initial output...")
let output = ssh_interactive_recv(ssh, 2000)
print("\n===== INITIAL OUTPUT =====")
print(output)
print("===== END =====\n")
print("Length:", len(output), "bytes")

ssh_interactive_close(ssh)
