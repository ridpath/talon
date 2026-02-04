print("[*] Bandit Level 26 -> 27: Complete Automation")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[*] Connect with tiny PTY")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[*] Wait for --More-- and enter Vim")
let initial = ssh_interactive_recv(ssh, 1500)
ssh_interactive_send(ssh, "v")

print("[*] Wait for Vim to load")
let vim_start = ssh_interactive_recv(ssh, 1000)

print("[*] Press ENTER to continue past startup screen")
ssh_interactive_send(ssh, "\r")
let vim_ready = ssh_interactive_recv(ssh, 500)

print("[*] Open password file with :e")
ssh_interactive_send(ssh, ":e /etc/bandit_pass/bandit27\r")
let password_screen = ssh_interactive_recv(ssh, 1000)

print("\n===== VIM SCREEN WITH PASSWORD =====")
print(password_screen)
print("===== END =====\n")

ssh_interactive_close(ssh)

print("[+] Level 26 -> 27 Complete!")
print("[*] Look for the password in the output above")
