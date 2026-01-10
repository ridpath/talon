pub struct CompletionGenerator;

impl CompletionGenerator {
    pub fn generate(shell: &str) -> Result<String, String> {
        match shell {
            "bash" => Ok(BASH_COMPLETION.to_string()),
            "zsh" => Ok(ZSH_COMPLETION.to_string()),
            "fish" => Ok(FISH_COMPLETION.to_string()),
            "powershell" => Ok(POWERSHELL_COMPLETION.to_string()),
            _ => Err(format!("Unsupported shell: {}", shell)),
        }
    }
    
    pub fn install(shell: &str) -> Result<(), String> {
        let completion = Self::generate(shell)?;
        
        let filename = match shell {
            "bash" => "talon.bash",
            "zsh" => "_talon",
            "fish" => "talon.fish",
            "powershell" => "talon.ps1",
            _ => return Err("Unsupported shell".to_string()),
        };
        
        std::fs::write(filename, completion)
            .map_err(|e| format!("Failed to write completion file: {}", e))?;
        
        println!("Generated completion file: {}", filename);
        println!("Install instructions:");
        
        match shell {
            "bash" => println!("  sudo cp {} /etc/bash_completion.d/", filename),
            "zsh" => println!("  sudo cp {} /usr/local/share/zsh/site-functions/", filename),
            "fish" => println!("  cp {} ~/.config/fish/completions/", filename),
            "powershell" => println!("  Add to PowerShell profile"),
            _ => {}
        }
        
        Ok(())
    }
}

const BASH_COMPLETION: &str = r#"#!/bin/bash
# Bash completion for talon

_talon_completions() {
    local cur prev commands template_types
    
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    commands="run repl new build wasm analyze db config man completion ctf diff_fuzz taint_analysis auto_rop heap_exploit kernel_exploit scan_cve find_similar_to chain safety help version"
    template_types="buffer-overflow rop format-string heap kernel ret2libc use-after-free race-condition shellcode web-sqli smart-contract basic"
    db_commands="search list show type platform"
    config_commands="init show edit"
    ctf_commands="new_session add_challenge set_connection add_note set_status submit_flag save_session load_session show_stats list_challenges"
    
    case "${prev}" in
        talon)
            COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
            return 0
            ;;
        new)
            COMPREPLY=( $(compgen -W "${template_types}" -- ${cur}) )
            return 0
            ;;
        db)
            COMPREPLY=( $(compgen -W "${db_commands}" -- ${cur}) )
            return 0
            ;;
        config)
            COMPREPLY=( $(compgen -W "${config_commands}" -- ${cur}) )
            return 0
            ;;
        ctf)
            COMPREPLY=( $(compgen -W "${ctf_commands}" -- ${cur}) )
            return 0
            ;;
        completion)
            COMPREPLY=( $(compgen -W "bash zsh fish powershell" -- ${cur}) )
            return 0
            ;;
        run|build|wasm|analyze)
            COMPREPLY=( $(compgen -f -X '!*.tal' -- ${cur}) )
            return 0
            ;;
    esac
    
    COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
    return 0
}

complete -F _talon_completions talon
"#;

const ZSH_COMPLETION: &str = r#"#compdef talon

_talon() {
    local -a commands template_types db_cmds config_cmds
    
    commands=(
        'run:Execute a Talon script'
        'repl:Start interactive REPL'
        'new:Generate exploit template'
        'build:Compile to native binary'
        'wasm:Compile to WebAssembly'
        'analyze:Analyze binary for vulnerabilities'
        'db:Query exploit database'
        'config:Manage configuration'
        'man:Display manual pages'
        'completion:Generate shell completions'
        'ctf:CTF session management'
        'diff_fuzz:Differential fuzzing'
        'taint_analysis:Taint analysis for info leak detection'
        'auto_rop:Automated ROP chain generation'
        'heap_exploit:Modern heap exploitation (glibc 2.35+)'
        'kernel_exploit:Automated kernel exploitation with CVE detection'
        'scan_cve:CVE scanner with exploit-db.com integration'
        'find_similar_to:Binary similarity analysis with function embedding'
        'chain:Multi-stage exploit chaining and orchestration'
        'safety:Runtime safety and resource management'
        'help:Show help information'
        'version:Display version'
    )
    
    template_types=(
        'buffer-overflow:Stack buffer overflow'
        'rop:Return-Oriented Programming'
        'format-string:Format string vulnerability'
        'heap:Heap exploitation'
        'kernel:Kernel privilege escalation'
        'ret2libc:Return to libc'
        'use-after-free:UAF exploitation'
        'race-condition:TOCTOU exploitation'
        'shellcode:Custom shellcode'
        'web-sqli:SQL injection'
        'smart-contract:Smart contract audit'
        'basic:Basic script'
    )
    
    db_cmds=(
        'search:Search exploits'
        'list:List all exploits'
        'show:Show exploit details'
        'type:Filter by type'
        'platform:Filter by platform'
    )
    
    config_cmds=(
        'init:Create default config'
        'show:Display config'
        'edit:Edit config file'
    )
    
    ctf_cmds=(
        'new_session:Create new CTF session'
        'add_challenge:Add challenge to session'
        'set_connection:Set challenge connection'
        'add_note:Add challenge note'
        'set_status:Update challenge status'
        'submit_flag:Submit flag'
        'save_session:Save session to file'
        'load_session:Load session from file'
        'show_stats:Show session statistics'
        'list_challenges:List all challenges'
    )
    
    case $words[2] in
        new)
            _describe 'template type' template_types
            ;;
        db)
            _describe 'db command' db_cmds
            ;;
        config)
            _describe 'config command' config_cmds
            ;;
        ctf)
            _describe 'ctf command' ctf_cmds
            ;;
        completion)
            compadd bash zsh fish powershell
            ;;
        run|build|wasm|analyze)
            _files -g '*.tal'
            ;;
        *)
            _describe 'command' commands
            ;;
    esac
}

_talon "$@"
"#;

const FISH_COMPLETION: &str = r#"# Fish completion for talon

# Main commands
complete -c talon -f
complete -c talon -n "__fish_use_subcommand" -a "run" -d "Execute Talon script"
complete -c talon -n "__fish_use_subcommand" -a "repl" -d "Start interactive REPL"
complete -c talon -n "__fish_use_subcommand" -a "new" -d "Generate exploit template"
complete -c talon -n "__fish_use_subcommand" -a "build" -d "Compile to native binary"
complete -c talon -n "__fish_use_subcommand" -a "wasm" -d "Compile to WebAssembly"
complete -c talon -n "__fish_use_subcommand" -a "analyze" -d "Analyze binary"
complete -c talon -n "__fish_use_subcommand" -a "db" -d "Query exploit database"
complete -c talon -n "__fish_use_subcommand" -a "config" -d "Manage configuration"
complete -c talon -n "__fish_use_subcommand" -a "man" -d "Display manual"
complete -c talon -n "__fish_use_subcommand" -a "completion" -d "Generate completions"
complete -c talon -n "__fish_use_subcommand" -a "ctf" -d "CTF session management"
complete -c talon -n "__fish_use_subcommand" -a "diff_fuzz" -d "Differential fuzzing"
complete -c talon -n "__fish_use_subcommand" -a "taint_analysis" -d "Taint analysis"
complete -c talon -n "__fish_use_subcommand" -a "auto_rop" -d "Automated ROP"
complete -c talon -n "__fish_use_subcommand" -a "heap_exploit" -d "Modern heap exploitation"
complete -c talon -n "__fish_use_subcommand" -a "kernel_exploit" -d "Kernel exploitation automation"
complete -c talon -n "__fish_use_subcommand" -a "scan_cve" -d "CVE scanner with impact assessment"
complete -c talon -n "__fish_use_subcommand" -a "find_similar_to" -d "Binary similarity analysis"
complete -c talon -n "__fish_use_subcommand" -a "chain" -d "Multi-stage exploit chaining"
complete -c talon -n "__fish_use_subcommand" -a "safety" -d "Runtime safety and resource management"
complete -c talon -n "__fish_use_subcommand" -a "help" -d "Show help"
complete -c talon -n "__fish_use_subcommand" -a "version" -d "Show version"

# Template types for 'new' command
complete -c talon -n "__fish_seen_subcommand_from new" -a "buffer-overflow" -d "Stack buffer overflow"
complete -c talon -n "__fish_seen_subcommand_from new" -a "rop" -d "Return-Oriented Programming"
complete -c talon -n "__fish_seen_subcommand_from new" -a "format-string" -d "Format string vulnerability"
complete -c talon -n "__fish_seen_subcommand_from new" -a "heap" -d "Heap exploitation"
complete -c talon -n "__fish_seen_subcommand_from new" -a "kernel" -d "Kernel privilege escalation"
complete -c talon -n "__fish_seen_subcommand_from new" -a "ret2libc" -d "Return to libc"
complete -c talon -n "__fish_seen_subcommand_from new" -a "use-after-free" -d "UAF exploitation"
complete -c talon -n "__fish_seen_subcommand_from new" -a "race-condition" -d "Race condition"
complete -c talon -n "__fish_seen_subcommand_from new" -a "shellcode" -d "Custom shellcode"
complete -c talon -n "__fish_seen_subcommand_from new" -a "web-sqli" -d "SQL injection"
complete -c talon -n "__fish_seen_subcommand_from new" -a "smart-contract" -d "Smart contract audit"
complete -c talon -n "__fish_seen_subcommand_from new" -a "basic" -d "Basic script"

# DB subcommands
complete -c talon -n "__fish_seen_subcommand_from db" -a "search" -d "Search exploits"
complete -c talon -n "__fish_seen_subcommand_from db" -a "list" -d "List all exploits"
complete -c talon -n "__fish_seen_subcommand_from db" -a "show" -d "Show exploit details"
complete -c talon -n "__fish_seen_subcommand_from db" -a "type" -d "Filter by type"
complete -c talon -n "__fish_seen_subcommand_from db" -a "platform" -d "Filter by platform"

# Config subcommands
complete -c talon -n "__fish_seen_subcommand_from config" -a "init" -d "Create default config"
complete -c talon -n "__fish_seen_subcommand_from config" -a "show" -d "Display config"
complete -c talon -n "__fish_seen_subcommand_from config" -a "edit" -d "Edit config file"

# CTF subcommands
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "new_session" -d "Create new CTF session"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "add_challenge" -d "Add challenge to session"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "set_connection" -d "Set challenge connection"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "add_note" -d "Add challenge note"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "set_status" -d "Update challenge status"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "submit_flag" -d "Submit flag"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "save_session" -d "Save session to file"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "load_session" -d "Load session from file"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "show_stats" -d "Show session statistics"
complete -c talon -n "__fish_seen_subcommand_from ctf" -a "list_challenges" -d "List all challenges"

# Completion shells
complete -c talon -n "__fish_seen_subcommand_from completion" -a "bash zsh fish powershell"

# File completions for script commands
complete -c talon -n "__fish_seen_subcommand_from run build wasm analyze" -a "(__fish_complete_suffix .tal)"

# Options
complete -c talon -s h -l help -d "Show help"
complete -c talon -s v -l version -d "Show version"
complete -c talon -s V -l verbose -d "Verbose output"
complete -c talon -s q -l quiet -d "Quiet mode"
complete -c talon -l no-color -d "Disable colors"
complete -c talon -l config -d "Config file path" -r
"#;

const POWERSHELL_COMPLETION: &str = r#"# PowerShell completion for talon

Register-ArgumentCompleter -Native -CommandName talon -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    
    $commands = @{
        'run' = 'Execute a Talon script'
        'repl' = 'Start interactive REPL'
        'new' = 'Generate exploit template'
        'build' = 'Compile to native binary'
        'ctf' = 'CTF session management'
        'diff_fuzz' = 'Differential fuzzing'
        'taint_analysis' = 'Taint analysis for info leak detection'
        'auto_rop' = 'Automated ROP chain generation'
        'heap_exploit' = 'Modern heap exploitation (glibc 2.35+)'
        'kernel_exploit' = 'Automated kernel exploitation with CVE detection'
        'scan_cve' = 'CVE scanner with exploit-db.com integration'
        'find_similar_to' = 'Binary similarity analysis with function embedding'
        'wasm' = 'Compile to WebAssembly'
        'analyze' = 'Analyze binary for vulnerabilities'
        'db' = 'Query exploit database'
        'config' = 'Manage configuration'
        'man' = 'Display manual pages'
        'completion' = 'Generate shell completions'
        'help' = 'Show help information'
        'version' = 'Display version'
    }
    
    $templateTypes = @{
        'buffer-overflow' = 'Stack buffer overflow'
        'rop' = 'Return-Oriented Programming'
        'format-string' = 'Format string vulnerability'
        'heap' = 'Heap exploitation'
        'kernel' = 'Kernel privilege escalation'
        'ret2libc' = 'Return to libc'
        'use-after-free' = 'UAF exploitation'
        'race-condition' = 'TOCTOU exploitation'
        'shellcode' = 'Custom shellcode'
        'web-sqli' = 'SQL injection'
        'smart-contract' = 'Smart contract audit'
        'basic' = 'Basic script structure'
    }
    
    $parts = $commandAst.ToString().Split(' ')
    
    if ($parts.Length -eq 2) {
        $commands.Keys | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $commands[$_])
        }
    }
    elseif ($parts[1] -eq 'new' -and $parts.Length -eq 3) {
        $templateTypes.Keys | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $templateTypes[$_])
        }
    }
    elseif ($parts[1] -eq 'db') {
        @('search', 'list', 'show', 'type', 'platform') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    }
    elseif ($parts[1] -eq 'config') {
        @('init', 'show', 'edit') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    }
    elseif ($parts[1] -eq 'completion') {
        @('bash', 'zsh', 'fish', 'powershell') | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
        }
    }
    elseif ($parts[1] -match '^(run|build|wasm|analyze)$') {
        Get-ChildItem -Filter "*.tal" | Where-Object { $_.Name -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_.Name, $_.Name, 'ParameterValue', $_.Name)
        }
    }
}
"#;
