print("[*] Bandit Level 27 -> 28: Git Clone Challenge")
print("[*] Note: Direct localhost:2220 cloning is blocked by server")

let level27_password = "upsNCc7vzaRDx6oZC6GiR6ERwe1MowGB"

print("\n[*] Connecting to bandit27...")
let ssh = connect_ssh("bandit.labs.overthewire.org", 2220, "bandit27", level27_password)

print("[*] Checking if repo exists locally...")
let check = ssh_run(ssh, "ls -la /home/bandit27-git/repo 2>&1")
print(check)

print("\n[*] Attempting git clone (expecting server block)...")
let workdir_cmd = "WORKDIR=$(mktemp -d) && echo $WORKDIR"
let workdir = ssh_run(ssh, workdir_cmd)
print("[+] Working directory:", workdir)

let clone_cmd = "cd " + trim(workdir) + " && GIT_SSH_COMMAND='ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null' git clone ssh://bandit27-git@localhost:2220/home/bandit27-git/repo 2>&1"
let clone_result = ssh_run(ssh, clone_cmd)

print("\n===== CLONE RESULT =====")
print(clone_result)
print("========================\n")

if contains(clone_result, "blocked")
    print("[-] Server blocks localhost connections as expected")
    print("[*] Solution: Use port forwarding from external machine")
    print("[*] Command: ssh -L 2221:localhost:2220 bandit27@bandit.labs.overthewire.org")
    print("[*] Then: GIT_SSH_COMMAND='ssh -p 2221' git clone ssh://bandit27-git@localhost:2221/home/bandit27-git/repo")
else
    print("[*] Checking for password in README...")
    let cat_cmd = "cd " + trim(workdir) + "/repo && cat README 2>&1"
    let readme = ssh_run(ssh, cat_cmd)
    print("\n===== README =====")
    print(readme)
    print("==================\n")
    
    let cleanup = ssh_run(ssh, "rm -rf " + trim(workdir))
end
