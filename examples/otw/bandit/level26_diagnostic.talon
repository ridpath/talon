print("[*] Level 26 Diagnostic - Testing Interactive Recv Timeout")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connecting with 3x20 PTY...")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
print("[+] Connected")

print("[*] Starting interactive shell...")
ssh_interactive_start(ssh)
print("[+] Interactive started")

print("[*] Testing recv with 1 second timeout...")
let output1 = ssh_interactive_recv(ssh, 1000)
print("[+] Recv returned! Length:", len(output1))
print("Output preview (first 200 chars):", substr(output1, 0, 200))

print("[*] Done - testing if timeout actually works")
ssh_interactive_close(ssh)
print("[+] Closed successfully")
