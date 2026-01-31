use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotebookEntry {
    entry_type: String,
    timestamp: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Notebook {
    title: String,
    created_at: String,
    entries: Vec<NotebookEntry>,
    variables: HashMap<String, String>,
}

impl Notebook {
    pub fn new(title: &str) -> Self {
        Notebook {
            title: title.to_string(),
            created_at: chrono::Local::now().to_string(),
            entries: Vec::new(),
            variables: HashMap::new(),
        }
    }

    pub fn add_note(&mut self, note: &str) {
        self.entries.push(NotebookEntry {
            entry_type: "note".to_string(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            content: note.to_string(),
        });
    }

    pub fn add_code(&mut self, code: &str) {
        self.entries.push(NotebookEntry {
            entry_type: "code".to_string(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            content: code.to_string(),
        });
    }

    pub fn add_finding(&mut self, finding: &str) {
        self.entries.push(NotebookEntry {
            entry_type: "finding".to_string(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            content: finding.to_string(),
        });
    }

    pub fn add_result(&mut self, result: &str) {
        self.entries.push(NotebookEntry {
            entry_type: "result".to_string(),
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            content: result.to_string(),
        });
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    pub fn export_markdown(&self, filename: &str) -> Result<(), String> {
        let mut markdown = String::new();

        markdown.push_str(&format!("# {}\n\n", self.title));
        markdown.push_str(&format!("**Created:** {}\n\n", self.created_at));
        markdown.push_str("---\n\n");

        for entry in &self.entries {
            match entry.entry_type.as_str() {
                "note" => {
                    markdown.push_str(&format!("### Note - {}\n\n", entry.timestamp));
                    markdown.push_str(&format!("{}\n\n", entry.content));
                }
                "finding" => {
                    markdown.push_str(&format!("### Finding - {}\n\n", entry.timestamp));
                    markdown.push_str(&format!("> {}\n\n", entry.content));
                }
                "code" => {
                    markdown.push_str(&format!("### Code - {}\n\n", entry.timestamp));
                    markdown.push_str("```talon\n");
                    markdown.push_str(&format!("{}\n", entry.content));
                    markdown.push_str("```\n\n");
                }
                "result" => {
                    markdown.push_str(&format!("### Result - {}\n\n", entry.timestamp));
                    markdown.push_str("```\n");
                    markdown.push_str(&format!("{}\n", entry.content));
                    markdown.push_str("```\n\n");
                }
                _ => {}
            }
        }

        if !self.variables.is_empty() {
            markdown.push_str("## Variables\n\n");
            markdown.push_str("| Name | Value |\n");
            markdown.push_str("|------|-------|\n");
            for (name, value) in &self.variables {
                markdown.push_str(&format!("| {} | {} |\n", name, value));
            }
            markdown.push('\n');
        }

        fs::write(filename, markdown)
            .map_err(|e| format!("Failed to write markdown file: {}", e))?;

        println!("Notebook exported to: {}", filename);
        Ok(())
    }

    pub fn save(&self, filename: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize notebook: {}", e))?;

        fs::write(filename, json).map_err(|e| format!("Failed to write notebook file: {}", e))?;

        println!("Notebook saved to: {}", filename);
        Ok(())
    }

    pub fn load(filename: &str) -> Result<Self, String> {
        let content = fs::read_to_string(filename)
            .map_err(|e| format!("Failed to read notebook file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse notebook file: {}", e))
    }
}

pub struct NotebookManager;

impl NotebookManager {
    pub fn start(title: &str) -> Notebook {
        println!("Starting notebook: {}", title);
        Notebook::new(title)
    }

    pub fn execute_with_notebook(notebook: &mut Notebook, code: &str) -> Result<String, String> {
        notebook.add_code(code);

        let cmds = crate::parser::parse_script(code).map_err(|e| format!("Parse error: {}", e))?;

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(crate::interpreter::interpret(&cmds))
            .map_err(|e| format!("Execution error: {}", e))?;

        let result = "Executed successfully";
        notebook.add_result(result);

        Ok(result.to_string())
    }
}
