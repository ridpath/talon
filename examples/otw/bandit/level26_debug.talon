print("[*] Level 26 Debug - Step by step")

let pass = "s0773xxkk0MXfdqOfPRVr9L3jJBUOgCZ"

print("[1] Connect")
let ssh = connect_ssh_pty("bandit.labs.overthewire.org", 2220, "bandit26", pass, 3, 20)

print("[2] Start interactive")
ssh_interactive_start(ssh)

print("[3] Wait for --More--")
let initial = ssh_interactive_recv(ssh, 1500)
print("[+] Got", len(initial), "bytes, contains --More--:", contains(initial, "--More--"))

print("[4] Send 'v' to enter Vim")
ssh_interactive_send(ssh, "v")

print("[5] Wait 500ms for Vim to start")
sleep(500)

print("[6] Try to recv with SHORT timeout (500ms)")
let vim_out = ssh_interactive_recv(ssh, 500)
print("[+] Received:", len(vim_out), "bytes")

if len(vim_out) > 0
    print("Output:", vim_out)
else
    print("[!] No output from Vim - it may not have started")
end

ssh_interactive_close(ssh)
