use crate::ast::{Command, Control, ShellcodeSpec};

/// Recursively print nodes in Graphviz DOT format
fn print_node(cmd: &Command, parent_id: usize, counter: &mut usize) {
    *counter += 1;
    let current_id = *counter;

    let label = match cmd {
        Command::GenerateShellcode(ShellcodeSpec { os, payload_type, .. }) => {
            format!("shellcode\\nOS: {}\\nType: {}", os, payload_type)
        }
        Command::DefineFunction(func_def) => format!("function {}", func_def.name),
        Command::CallFunction { name, .. } => format!("call {}", name),
        Command::VarDecl { name, .. } => format!("let {}", name),
        Command::Assignment { name, .. } => format!("assign {}", name),
        Command::Control(Control::If { .. }) => "if".to_string(),
        Command::Control(Control::For { .. }) => "for".to_string(),
        _ => format!("{:?}", cmd).split_once('(').map(|(a, _)| a).unwrap_or("cmd").to_string(),
    };

    println!("  node{} [label=\"{}\"];", current_id, label);
    println!("  node{} -> node{};", parent_id, current_id);

    // Recurse if nested children
    match cmd {
        Command::DefineFunction(func_def) => {
            for sub in &func_def.body {
                print_node(sub, current_id, counter);
            }
        }
        Command::Control(Control::If { then_body, else_body, .. }) => {
            for sub in then_body {
                print_node(sub, current_id, counter);
            }
            for sub in else_body {
                print_node(sub, current_id, counter);
            }
        }
        Command::Control(Control::For { body, .. }) => {
            for sub in body {
                print_node(sub, current_id, counter);
            }
        }
        _ => {}
    }
}

/// Entry: Visualize AST in Graphviz DOT format
pub fn visualize(commands: &[Command]) {
    println!("digraph AST {{");
    println!("  node0 [label=\"ROOT\"];");

    let mut counter = 0;

    for cmd in commands {
        print_node(cmd, 0, &mut counter);
    }

    println!("}}");
}
