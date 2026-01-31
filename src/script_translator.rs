pub struct ScriptTranslator;

impl ScriptTranslator {
    pub fn from_pwntools(python_script: &str) -> Result<String, String> {
        let mut talon_script = String::new();

        talon_script.push_str("# Translated from pwntools\n\n");

        for line in python_script.lines() {
            if line.contains("remote(") {
                talon_script.push_str("let s = connect(\"target\", 1337)\n");
            } else if line.contains("p64(") {
                talon_script.push_str("let packed = pack64(address)\n");
            } else if line.contains("sendline(") {
                talon_script.push_str("send(s, payload + \"\\n\")\n");
            } else if line.contains("recvuntil(") {
                talon_script.push_str("let data = recv_until(s, marker)\n");
            } else if line.contains("interactive()") {
                talon_script.push_str("interactive(s)\n");
            }
        }

        Ok(talon_script)
    }

    pub fn from_metasploit(ruby_module: &str) -> Result<String, String> {
        let mut talon_script = String::new();

        talon_script.push_str("# Translated from Metasploit\n\n");

        for line in ruby_module.lines() {
            if line.contains("connect(") {
                talon_script.push_str("let s = connect(target, port)\n");
            } else if line.contains("payload") {
                talon_script.push_str("let payload = generate_payload()\n");
            } else if line.contains("send_request") {
                talon_script.push_str("send(s, request)\n");
            }
        }

        Ok(talon_script)
    }

    pub fn to_pwntools(talon_script: &str) -> Result<String, String> {
        let mut python_script = String::from("#!/usr/bin/env python3\n");
        python_script.push_str("from pwn import *\n\n");

        for line in talon_script.lines() {
            if line.contains("connect(") {
                python_script.push_str("r = remote('target', 1337)\n");
            } else if line.contains("pack64(") {
                python_script.push_str("packed = p64(address)\n");
            } else if line.contains("send(") {
                python_script.push_str("r.sendline(payload)\n");
            } else if line.contains("interactive(") {
                python_script.push_str("r.interactive()\n");
            }
        }

        Ok(python_script)
    }
}
