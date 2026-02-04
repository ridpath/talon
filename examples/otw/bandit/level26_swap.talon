print("[*] Bandit Level 26 -> 27: Handling Swap File")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connecting...")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[*] Get --More-- and enter Vim")
let initial = ssh_interactive_recv(ssh, 1500)
ssh_interactive_send(ssh, "v")
let vim_start = ssh_interactive_recv(ssh, 1000)

print("[*] Press ENTER past startup")
ssh_interactive_send(ssh, "\r")
let vim_ready = ssh_interactive_recv(ssh, 500)

print("[*] Open password file")
ssh_interactive_send(ssh, ":e /etc/bandit_pass/bandit27\r")
let warning = ssh_interactive_recv(ssh, 800)

print("[*] Press ENTER past warning")
ssh_interactive_send(ssh, "\r")
let swap_msg = ssh_interactive_recv(ssh, 1000)

print("[*] Handle swap file - press space to continue")
ssh_interactive_send(ssh, " ")
let swap_options = ssh_interactive_recv(ssh, 800)

print("[*] Choose 'O' for Open Read-Only")
ssh_interactive_send(ssh, "O")
let file_content = ssh_interactive_recv(ssh, 1000)

print("\n========== FINAL OUTPUT ==========")
print(file_content)
print("==================================\n")

ssh_interactive_close(ssh)
