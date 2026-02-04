print("[*] Verifying Level 27 Password")

let level27_password = "upsNCc7vzaRDx6oZC6GiR6ERwe1MowGB"

print("[*] Attempting to connect to bandit27...")
let ssh = connect_ssh("bandit.labs.overthewire.org", 2220, "bandit27", level27_password)

if ssh != null
    print("[+] SUCCESS! Level 27 password is correct")
    
    print("[*] Checking for git repo cloning requirement")
    let ls_output = ssh_run(ssh, "ls -la")
    print(ls_output)
    
    print("\n[+] Level 26 -> 27 is already solved!")
    print("[*] Password: " + level27_password)
else
    print("[-] Password incorrect, need to solve Level 26")
end
