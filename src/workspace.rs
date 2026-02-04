use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct Workspace {
    name: String,
    created_at: String,
    challenges: HashMap<String, Challenge>,
    notes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Challenge {
    name: String,
    category: String,
    points: u32,
    status: String,
    files: Vec<String>,
    notes: Vec<String>,
    solution: Option<String>,
}

pub struct WorkspaceManager;

impl WorkspaceManager {
    pub fn init(name: &str) -> Result<(), String> {
        let workspace_dir = Self::get_workspace_dir(name)?;

        if workspace_dir.exists() {
            return Err(format!("Workspace '{}' already exists", name));
        }

        fs::create_dir_all(&workspace_dir)
            .map_err(|e| format!("Failed to create workspace directory: {}", e))?;

        let workspace = Workspace {
            name: name.to_string(),
            created_at: chrono::Local::now().to_string(),
            challenges: HashMap::new(),
            notes: Vec::new(),
        };

        Self::save_workspace(&workspace)?;

        println!("Workspace '{}' created successfully", name);
        println!("Location: {}", workspace_dir.display());

        Ok(())
    }

    pub fn add(workspace_name: &str, challenge_name: &str) -> Result<(), String> {
        let mut workspace = Self::load_workspace(workspace_name)?;

        let challenge = Challenge {
            name: challenge_name.to_string(),
            category: "Unknown".to_string(),
            points: 0,
            status: "not_started".to_string(),
            files: Vec::new(),
            notes: Vec::new(),
            solution: None,
        };

        workspace
            .challenges
            .insert(challenge_name.to_string(), challenge);
        Self::save_workspace(&workspace)?;

        let challenge_dir = Self::get_workspace_dir(workspace_name)?.join(challenge_name);
        fs::create_dir_all(&challenge_dir)
            .map_err(|e| format!("Failed to create challenge directory: {}", e))?;

        println!(
            "Challenge '{}' added to workspace '{}'",
            challenge_name, workspace_name
        );

        Ok(())
    }

    pub fn list(workspace_name: &str) -> Result<(), String> {
        let workspace = Self::load_workspace(workspace_name)?;

        println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
        println!("║ Workspace: {}", workspace.name);
        println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

        if workspace.challenges.is_empty() {
            println!("No challenges added yet.");
            println!("Use 'talon workspace add <workspace> <challenge>' to add challenges.\n");
            return Ok(());
        }

        println!("Challenges:");
        for (name, challenge) in &workspace.challenges {
            let status_icon = match challenge.status.as_str() {
                "solved" => "[OK]",
                "in_progress" => "●",
                "blocked" => "[ERROR]",
                _ => "○",
            };

            println!(
                "  {} {} - {} ({} points) [{}]",
                status_icon, name, challenge.category, challenge.points, challenge.status
            );

            if !challenge.files.is_empty() {
                println!("      Files: {}", challenge.files.join(", "));
            }
        }

        if !workspace.notes.is_empty() {
            println!("\nNotes:");
            for (i, note) in workspace.notes.iter().enumerate() {
                println!("  {}. {}", i + 1, note);
            }
        }

        println!();
        Ok(())
    }

    pub fn sync(workspace_name: &str) -> Result<(), String> {
        let workspace = Self::load_workspace(workspace_name)?;

        let workspace_dir = Self::get_workspace_dir(workspace_name)?;

        for (challenge_name, mut challenge) in workspace.challenges.clone() {
            let challenge_dir = workspace_dir.join(&challenge_name);

            if challenge_dir.exists() {
                let entries = fs::read_dir(&challenge_dir)
                    .map_err(|e| format!("Failed to read challenge directory: {}", e))?;

                let mut files = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(filename) = entry.file_name().to_str() {
                            if !filename.starts_with('.') {
                                files.push(filename.to_string());
                            }
                        }
                    }
                }

                challenge.files = files;
            }
        }

        Self::save_workspace(&workspace)?;
        println!("Workspace '{}' synced successfully", workspace_name);

        Ok(())
    }

    pub fn set_status(
        workspace_name: &str,
        challenge_name: &str,
        status: &str,
    ) -> Result<(), String> {
        let mut workspace = Self::load_workspace(workspace_name)?;

        if let Some(challenge) = workspace.challenges.get_mut(challenge_name) {
            challenge.status = status.to_string();
            Self::save_workspace(&workspace)?;
            println!("Challenge '{}' status set to: {}", challenge_name, status);
            Ok(())
        } else {
            Err(format!(
                "Challenge '{}' not found in workspace",
                challenge_name
            ))
        }
    }

    pub fn add_note(workspace_name: &str, note: &str) -> Result<(), String> {
        let mut workspace = Self::load_workspace(workspace_name)?;
        workspace.notes.push(note.to_string());
        Self::save_workspace(&workspace)?;
        println!("Note added to workspace '{}'", workspace_name);
        Ok(())
    }

    pub fn add_solution(
        workspace_name: &str,
        challenge_name: &str,
        solution_path: &str,
    ) -> Result<(), String> {
        let mut workspace = Self::load_workspace(workspace_name)?;

        if let Some(challenge) = workspace.challenges.get_mut(challenge_name) {
            let solution_content = fs::read_to_string(solution_path)
                .map_err(|e| format!("Failed to read solution file: {}", e))?;

            challenge.solution = Some(solution_content);
            challenge.status = "solved".to_string();

            Self::save_workspace(&workspace)?;
            println!("Solution added for challenge '{}'", challenge_name);
            Ok(())
        } else {
            Err(format!(
                "Challenge '{}' not found in workspace",
                challenge_name
            ))
        }
    }

    fn get_workspace_dir(name: &str) -> Result<PathBuf, String> {
        use directories::BaseDirs;

        if let Some(base_dirs) = BaseDirs::new() {
            let workspaces_root = base_dirs.home_dir().join(".talon_workspaces");
            Ok(workspaces_root.join(name))
        } else {
            Err("Failed to determine home directory".to_string())
        }
    }

    fn get_workspace_file(name: &str) -> Result<PathBuf, String> {
        Ok(Self::get_workspace_dir(name)?.join("workspace.json"))
    }

    fn load_workspace(name: &str) -> Result<Workspace, String> {
        let workspace_file = Self::get_workspace_file(name)?;

        if !workspace_file.exists() {
            return Err(format!("Workspace '{}' not found", name));
        }

        let content = fs::read_to_string(&workspace_file)
            .map_err(|e| format!("Failed to read workspace file: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse workspace file: {}", e))
    }

    fn save_workspace(workspace: &Workspace) -> Result<(), String> {
        let workspace_file = Self::get_workspace_file(&workspace.name)?;

        let content = serde_json::to_string_pretty(workspace)
            .map_err(|e| format!("Failed to serialize workspace: {}", e))?;

        fs::write(&workspace_file, content)
            .map_err(|e| format!("Failed to write workspace file: {}", e))?;

        Ok(())
    }

    pub fn list_all() -> Result<(), String> {
        use directories::BaseDirs;

        if let Some(base_dirs) = BaseDirs::new() {
            let workspaces_root = base_dirs.home_dir().join(".talon_workspaces");

            if !workspaces_root.exists() {
                println!("No workspaces found.");
                println!("Use 'talon workspace init <name>' to create a workspace.\n");
                return Ok(());
            }

            let entries = fs::read_dir(&workspaces_root)
                .map_err(|e| format!("Failed to read workspaces directory: {}", e))?;

            println!("\nAvailable workspaces:");
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.path().is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            if let Ok(workspace) = Self::load_workspace(name) {
                                let solved = workspace
                                    .challenges
                                    .values()
                                    .filter(|c| c.status == "solved")
                                    .count();
                                let total = workspace.challenges.len();

                                println!("  {} - {}/{} solved", name, solved, total);
                            }
                        }
                    }
                }
            }
            println!();

            Ok(())
        } else {
            Err("Failed to determine home directory".to_string())
        }
    }
}
