use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::interpreter::Value;

pub trait TalonPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn author(&self) -> &str;
    
    fn init(&mut self) -> Result<(), String>;
    fn execute(&self, command: &str, args: HashMap<String, Value>) -> Result<Value, String>;
    fn commands(&self) -> Vec<PluginCommand>;
    fn help(&self, command: &str) -> Option<String>;
}

#[derive(Debug, Clone)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub examples: Vec<String>,
    pub required_args: Vec<String>,
    pub optional_args: Vec<String>,
}

impl PluginCommand {
    pub fn new(name: &str, description: &str, usage: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            usage: usage.to_string(),
            examples: Vec::new(),
            required_args: Vec::new(),
            optional_args: Vec::new(),
        }
    }
    
    pub fn with_example(mut self, example: &str) -> Self {
        self.examples.push(example.to_string());
        self
    }
    
    pub fn with_required_arg(mut self, arg: &str) -> Self {
        self.required_args.push(arg.to_string());
        self
    }
    
    pub fn with_optional_arg(mut self, arg: &str) -> Self {
        self.optional_args.push(arg.to_string());
        self
    }
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn TalonPlugin>>,
    plugin_paths: Vec<PathBuf>,
}

impl PluginManager {
    pub fn new() -> Self {
        let mut manager = Self {
            plugins: HashMap::new(),
            plugin_paths: Vec::new(),
        };
        
        manager.init_default_paths();
        manager
    }
    
    fn init_default_paths(&mut self) {
        if let Ok(home_env) = std::env::var("HOME") {
            let home = PathBuf::from(home_env);
            self.plugin_paths.push(home.join(".talon/plugins"));
        } else if let Ok(home_env) = std::env::var("USERPROFILE") {
            let home = PathBuf::from(home_env);
            self.plugin_paths.push(home.join(".talon/plugins"));
        }
        
        self.plugin_paths.push(PathBuf::from("./talon_plugins"));
        
        if let Ok(custom_path) = std::env::var("TALON_PLUGIN_PATH") {
            for path in custom_path.split(':') {
                self.plugin_paths.push(PathBuf::from(path));
            }
        }
    }
    
    pub fn register_plugin(&mut self, plugin: Box<dyn TalonPlugin>) -> Result<(), String> {
        let name = plugin.name().to_string();
        
        if self.plugins.contains_key(&name) {
            return Err(format!("Plugin '{}' is already registered", name));
        }
        
        self.plugins.insert(name.clone(), plugin);
        
        if let Some(plugin) = self.plugins.get_mut(&name) {
            plugin.init()?;
        }
        
        Ok(())
    }
    
    pub fn load_script_plugin(&mut self, path: &Path) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("Plugin file not found: {}", path.display()));
        }
        
        let script_content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read plugin: {}", e))?;
        
        let plugin = ScriptPlugin::from_file(path, &script_content)?;
        self.register_plugin(Box::new(plugin))
    }
    
    pub fn discover_plugins(&mut self) -> Result<Vec<String>, String> {
        let mut discovered = Vec::new();
        
        for search_path in &self.plugin_paths.clone() {
            if !search_path.exists() {
                continue;
            }
            
            if let Ok(entries) = fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    
                    if path.extension().and_then(|s| s.to_str()) == Some("tal") {
                        match self.load_script_plugin(&path) {
                            Ok(_) => {
                                discovered.push(path.display().to_string());
                            }
                            Err(e) => {
                                eprintln!("Warning: Failed to load plugin {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(discovered)
    }
    
    pub fn execute(&self, plugin_name: &str, command: &str, args: HashMap<String, Value>) -> Result<Value, String> {
        let plugin = self.plugins.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;
        
        plugin.execute(command, args)
    }
    
    pub fn list_plugins(&self) -> Vec<(&str, &str, &str)> {
        self.plugins.iter()
            .map(|(name, plugin)| (name.as_str(), plugin.version(), plugin.description()))
            .collect()
    }
    
    pub fn get_plugin(&self, name: &str) -> Option<&Box<dyn TalonPlugin>> {
        self.plugins.get(name)
    }
    
    pub fn get_all_commands(&self) -> Vec<(String, Vec<PluginCommand>)> {
        self.plugins.iter()
            .map(|(name, plugin)| (name.clone(), plugin.commands()))
            .collect()
    }
    
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), String> {
        self.plugins.remove(name)
            .map(|_| ())
            .ok_or_else(|| format!("Plugin '{}' not found", name))
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ScriptPlugin {
    name: String,
    version: String,
    description: String,
    author: String,
    script_content: String,
    commands: Vec<PluginCommand>,
}

impl ScriptPlugin {
    pub fn from_file(path: &Path, content: &str) -> Result<Self, String> {
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let metadata = Self::parse_metadata(content);
        
        Ok(Self {
            name: metadata.get("name").unwrap_or(&name).to_string(),
            version: metadata.get("version").unwrap_or(&"0.1.0".to_string()).to_string(),
            description: metadata.get("description").unwrap_or(&"No description".to_string()).to_string(),
            author: metadata.get("author").unwrap_or(&"Unknown".to_string()).to_string(),
            script_content: content.to_string(),
            commands: Vec::new(),
        })
    }
    
    fn parse_metadata(content: &str) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(meta_line) = trimmed.strip_prefix("//!") {
                if let Some((key, value)) = meta_line.trim().split_once(':') {
                    metadata.insert(key.trim().to_lowercase(), value.trim().to_string());
                }
            }
        }
        
        metadata
    }
}

impl TalonPlugin for ScriptPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn author(&self) -> &str {
        &self.author
    }
    
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    
    fn execute(&self, _command: &str, _args: HashMap<String, Value>) -> Result<Value, String> {
        Err("Script plugin execution requires interpreter integration".to_string())
    }
    
    fn commands(&self) -> Vec<PluginCommand> {
        self.commands.clone()
    }
    
    fn help(&self, command: &str) -> Option<String> {
        self.commands.iter()
            .find(|cmd| cmd.name == command)
            .map(|cmd| format!("{}\n\nUsage: {}\n\n{}", cmd.description, cmd.usage, 
                               if cmd.examples.is_empty() { String::new() } 
                               else { format!("Examples:\n{}", cmd.examples.join("\n")) }))
    }
}

pub fn create_example_plugin_template() -> String {
    r#"//! name: example_plugin
//! version: 1.0.0
//! description: Example Talon plugin demonstrating plugin capabilities
//! author: Your Name

define function plugin_init() {
    print("Example plugin initialized")
    return "OK"
}

define function custom_scan(target: string) {
    print("Scanning target: " + target)
    
    return {
        "status": "success",
        "results": ["finding1", "finding2"]
    }
}

define function custom_exploit(target: string, payload: string) {
    print("Exploiting target: " + target)
    print("Using payload: " + payload)
    
    return {
        "exploited": true,
        "shell": "interactive"
    }
}
"#.to_string()
}

pub fn print_plugin_documentation() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                        TALON PLUGIN SYSTEM                                ║
╚═══════════════════════════════════════════════════════════════════════════╝

OVERVIEW
The Talon plugin system allows you to extend the language with custom modules
written in Talon itself. Plugins can add new commands, utilities, and workflows
to your exploitation toolkit.

PLUGIN LOCATIONS
1. ~/.talon/plugins/          - User plugins
2. ./talon_plugins/            - Project-specific plugins
3. $TALON_PLUGIN_PATH          - Custom paths (colon-separated)

CREATING A PLUGIN
Create a .tal file with metadata comments at the top:

//! name: my_plugin
//! version: 1.0.0
//! description: What this plugin does
//! author: Your Name

define function plugin_init() {{
    // Initialization code
    return "OK"
}}

define function my_command(arg1: string) {{
    // Your plugin logic here
    return result
}}

LOADING PLUGINS
# Auto-discover and load from default paths
talon plugin discover

# Load specific plugin
talon plugin load path/to/plugin.tal

# List loaded plugins
talon plugin list

# Show plugin info
talon plugin info <name>

# Unload plugin
talon plugin unload <name>

USING PLUGINS
Once loaded, plugin functions are available in the global namespace:

let result = my_command("argument")
print(result)

BEST PRACTICES
- Keep plugins focused on a single task or domain
- Document your functions with comments
- Provide meaningful error messages
- Test plugins before distribution
- Version your plugins semantically

EXAMPLE PLUGINS
- web_scanner.tal       - Custom web vulnerability scanner
- crypto_breaker.tal    - Specialized cryptography attacks
- report_generator.tal  - Automated report generation
- custom_payloads.tal   - Domain-specific payload library

For more information: talon man plugins
"#);
}
