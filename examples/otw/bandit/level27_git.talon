print("[*] Bandit Level 27 -> 28: Git Clone Challenge")

let level27_password = "upsNCc7vzaRDx6oZC6GiR6ERwe1MowGB"

print("[*] Connecting to bandit27")
let ssh = connect_ssh("bandit.labs.overthewire.org", 2220, "bandit27", level27_password)

print("[*] Cloning repo with host key bypass")
let commands = "WORKDIR=$(mktemp -d) && cd $WORKDIR && GIT_SSH_COMMAND='ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null' git clone ssh://bandit27-git@localhost:2220/home/bandit27-git/repo 2>&1 && cat repo/README && rm -rf $WORKDIR"
let output = ssh_run(ssh, commands)

print("\n=== OUTPUT ===")
print(output)
print("==============\n")
