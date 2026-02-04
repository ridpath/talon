print("[*] Level 26 - No Sleep Version")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[1] Connect & start")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)
ssh_interactive_start(ssh)

print("[2] Get initial --More--")
let initial = ssh_interactive_recv(ssh, 1500)
print("[+] Got", len(initial), "bytes")

print("[3] Send 'v' for Vim")
ssh_interactive_send(ssh, "v")

print("[4] Receive Vim output (1 sec timeout)")
let vim_out = ssh_interactive_recv(ssh, 1000)
print("[+] Vim output:", len(vim_out), "bytes")

if len(vim_out) > 100
    print("First 500 chars:")
    print(vim_out)
end

ssh_interactive_close(ssh)
print("[+] Done")
