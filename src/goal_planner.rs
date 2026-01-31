use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Deserialize, Serialize};
use crate::ast::{Command, Expr, Literal};
use crate::z3_solver::{Z3Solver, Z3Constraint, Z3Type};
use crate::rop_gadget_finder::{ROPGadgetFinder, GadgetCategory, Architecture};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub goal_type: String,
    pub target_address: Option<u64>,
    pub target_value: Option<u64>,
    pub constraints: Vec<String>,
    pub available_primitives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    pub name: String,
    pub capability: String,
    pub cost: u32,
    pub prerequisites: Vec<String>,
    pub effects: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ActionNode {
    pub action: String,
    pub cost: u32,
    pub children: Vec<usize>,
}

pub struct GoalPlanner {
    primitives: HashMap<String, Primitive>,
    knowledge_base: HashMap<String, Vec<String>>,
    gadget_finder: Option<ROPGadgetFinder>,
    binary_path: Option<String>,
}

impl GoalPlanner {
    pub fn new() -> Self {
        let mut planner = GoalPlanner {
            primitives: HashMap::new(),
            knowledge_base: HashMap::new(),
            gadget_finder: None,
            binary_path: None,
        };
        planner.initialize_primitives();
        planner
    }

    pub fn set_binary(&mut self, binary_path: String) -> Result<(), String> {
        log::info!("Goal planner analyzing binary: {}", binary_path);
        let mut finder = ROPGadgetFinder::new(Architecture::X64)?;
        finder.analyze_file(&binary_path)?;
        self.gadget_finder = Some(finder);
        self.binary_path = Some(binary_path);
        Ok(())
    }

    fn initialize_primitives(&mut self) {
        self.primitives.insert("write4".to_string(), Primitive {
            name: "write4".to_string(),
            capability: "write_4_bytes".to_string(),
            cost: 1,
            prerequisites: vec!["control_flow".to_string()],
            effects: vec!["arbitrary_write".to_string()],
        });

        self.primitives.insert("read8".to_string(), Primitive {
            name: "read8".to_string(),
            capability: "read_8_bytes".to_string(),
            cost: 1,
            prerequisites: vec![],
            effects: vec!["information_leak".to_string()],
        });

        self.primitives.insert("stack_pivot".to_string(), Primitive {
            name: "stack_pivot".to_string(),
            capability: "control_stack_pointer".to_string(),
            cost: 2,
            prerequisites: vec!["buffer_overflow".to_string()],
            effects: vec!["control_flow".to_string()],
        });

        self.primitives.insert("rop_gadget".to_string(), Primitive {
            name: "rop_gadget".to_string(),
            capability: "execute_gadget".to_string(),
            cost: 1,
            prerequisites: vec!["control_flow".to_string()],
            effects: vec!["arbitrary_code_execution".to_string()],
        });

        self.primitives.insert("format_string".to_string(), Primitive {
            name: "format_string".to_string(),
            capability: "format_string_exploit".to_string(),
            cost: 1,
            prerequisites: vec![],
            effects: vec!["arbitrary_write".to_string(), "information_leak".to_string()],
        });

        self.primitives.insert("heap_spray".to_string(), Primitive {
            name: "heap_spray".to_string(),
            capability: "spray_heap".to_string(),
            cost: 3,
            prerequisites: vec![],
            effects: vec!["controlled_memory_layout".to_string()],
        });

        self.primitives.insert("uaf_trigger".to_string(), Primitive {
            name: "uaf_trigger".to_string(),
            capability: "use_after_free".to_string(),
            cost: 2,
            prerequisites: vec!["controlled_memory_layout".to_string()],
            effects: vec!["arbitrary_write".to_string()],
        });
    }

    pub async fn synthesize_plan(&self, goal: &Goal) -> Result<Vec<String>, String> {
        let target_capability = match goal.goal_type.as_str() {
            "arbitrary_write" => "arbitrary_write",
            "code_execution" => "arbitrary_code_execution",
            "information_leak" => "information_leak",
            _ => return Err(format!("Unknown goal type: {}", goal.goal_type)),
        };

        let plan = self.backward_search(target_capability, &goal.constraints)?;
        
        if plan.is_empty() {
            return Err(format!("No plan found for goal: {}", goal.goal_type));
        }

        Ok(plan)
    }

    fn backward_search(&self, target: &str, constraints: &[String]) -> Result<Vec<String>, String> {
        let mut plan = Vec::new();
        let mut current_goals = VecDeque::new();
        let mut satisfied = HashSet::new();
        
        current_goals.push_back(target.to_string());

        while let Some(goal) = current_goals.pop_front() {
            if satisfied.contains(&goal) {
                continue;
            }

            let applicable_primitives: Vec<&Primitive> = self.primitives
                .values()
                .filter(|p| p.effects.contains(&goal) && self.meets_constraints(p, constraints))
                .collect();

            if let Some(best_primitive) = applicable_primitives.iter().min_by_key(|p| p.cost) {
                plan.push(best_primitive.name.clone());
                satisfied.insert(goal.clone());

                for prereq in &best_primitive.prerequisites {
                    if !satisfied.contains(prereq) {
                        current_goals.push_back(prereq.clone());
                    }
                }
            } else if goal == target {
                return Err(format!("Cannot satisfy goal: {} with given constraints", goal));
            }
        }

        plan.reverse();
        Ok(plan)
    }

    fn meets_constraints(&self, primitive: &Primitive, constraints: &[String]) -> bool {
        for constraint in constraints {
            match constraint.as_str() {
                "no_null_bytes" => {
                    if primitive.name.contains("write4") {
                        return false;
                    }
                }
                "must_preserve_rdx" => {
                    if primitive.name == "rop_gadget" {
                        return false;
                    }
                }
                "nx_enabled" => {
                    if primitive.name.contains("shellcode") {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }

    pub async fn generate_exploit_code(&self, plan: &[String], goal: &Goal) -> Result<Vec<Command>, String> {
        let mut commands = Vec::new();

        let finder = self.gadget_finder.as_ref()
            .ok_or_else(|| "No binary loaded. Call set_binary() first".to_string())?;

        for step in plan {
            match step.as_str() {
                "format_string" => {
                    commands.extend(self.generate_format_string_commands(goal)?);
                }
                "stack_pivot" => {
                    commands.extend(self.generate_stack_pivot_commands(goal, finder)?);
                }
                "write4" => {
                    commands.extend(self.generate_write4_commands(goal, finder)?);
                }
                "rop_gadget" => {
                    commands.extend(self.generate_rop_commands(goal, finder)?);
                }
                "heap_spray" => {
                    commands.extend(self.generate_heap_spray_commands(goal)?);
                }
                "uaf_trigger" => {
                    commands.extend(self.generate_uaf_commands(goal)?);
                }
                _ => {
                    log::warn!("Unknown exploit step: {}", step);
                }
            }
        }

        log::info!("Generated {} commands from plan with {} steps", commands.len(), plan.len());
        Ok(commands)
    }

    fn generate_format_string_commands(&self, goal: &Goal) -> Result<Vec<Command>, String> {
        let target = goal.target_address.unwrap_or(0xdeadbeef);
        let value = goal.target_value.unwrap_or(0xcafebabe);
        
        let mut solver = Z3Solver::new();
        solver.add_variable("offset".to_string(), Z3Type::Integer, 32);
        for constraint in &goal.constraints {
            if constraint == "no_null_bytes" {
                solver.add_constraint(Z3Constraint::NoNullBytes("offset".to_string()));
            }
        }

        let commands = vec![
            Command::VarDecl {
                name: "target_addr".to_string(),
                value: Expr::Literal(Literal::Number(target as i64)),
            },
            Command::VarDecl {
                name: "target_value".to_string(),
                value: Expr::Literal(Literal::Number(value as i64)),
            },
        ];

        log::info!("Generated format string exploit for address 0x{:x}", target);
        Ok(commands)
    }

    fn generate_stack_pivot_commands(&self, _goal: &Goal, finder: &ROPGadgetFinder) -> Result<Vec<Command>, String> {
        let pivot_gadgets = finder.find_gadgets_by_category(GadgetCategory::StackPivot);
        
        if pivot_gadgets.is_empty() {
            return Err("No stack pivot gadgets found in binary".to_string());
        }

        let gadget = &pivot_gadgets[0];
        log::info!("Using stack pivot gadget at 0x{:x}: {:?}", gadget.address, gadget.instructions);

        Ok(vec![
            Command::VarDecl {
                name: "pivot_gadget".to_string(),
                value: Expr::Literal(Literal::Number(gadget.address as i64)),
            },
        ])
    }

    fn generate_write4_commands(&self, goal: &Goal, finder: &ROPGadgetFinder) -> Result<Vec<Command>, String> {
        let target = goal.target_address.unwrap_or(0xdeadbeef);
        let value = goal.target_value.unwrap_or(0xcafebabe);

        let load_gadgets = finder.find_gadgets_by_category(GadgetCategory::LoadRegister);
        let store_gadgets = finder.find_gadgets_by_category(GadgetCategory::StoreMemory);

        if load_gadgets.is_empty() || store_gadgets.is_empty() {
            return Err("Insufficient gadgets for write4 primitive".to_string());
        }

        log::info!("Building write4 chain with {} load and {} store gadgets", 
                  load_gadgets.len(), store_gadgets.len());

        Ok(vec![
            Command::VarDecl {
                name: "target_addr".to_string(),
                value: Expr::Literal(Literal::Number(target as i64)),
            },
            Command::VarDecl {
                name: "target_value".to_string(),
                value: Expr::Literal(Literal::Number(value as i64)),
            },
            Command::VarDecl {
                name: "load_gadget".to_string(),
                value: Expr::Literal(Literal::Number(load_gadgets[0].address as i64)),
            },
            Command::VarDecl {
                name: "store_gadget".to_string(),
                value: Expr::Literal(Literal::Number(store_gadgets[0].address as i64)),
            },
        ])
    }

    fn generate_rop_commands(&self, _goal: &Goal, finder: &ROPGadgetFinder) -> Result<Vec<Command>, String> {
        let control_flow = finder.find_gadgets_by_category(GadgetCategory::ControlFlow);
        
        if control_flow.is_empty() {
            return Err("No control flow gadgets found".to_string());
        }

        log::info!("Building ROP chain with {} control flow gadgets", control_flow.len());

        Ok(vec![
            Command::VarDecl {
                name: "rop_gadget".to_string(),
                value: Expr::Literal(Literal::Number(control_flow[0].address as i64)),
            },
        ])
    }

    fn generate_heap_spray_commands(&self, _goal: &Goal) -> Result<Vec<Command>, String> {
        Ok(vec![
            Command::VarDecl {
                name: "spray_count".to_string(),
                value: Expr::Literal(Literal::Number(1000)),
            },
            Command::VarDecl {
                name: "spray_size".to_string(),
                value: Expr::Literal(Literal::Number(256)),
            },
        ])
    }

    fn generate_uaf_commands(&self, _goal: &Goal) -> Result<Vec<Command>, String> {
        Ok(vec![
            Command::VarDecl {
                name: "chunk_size".to_string(),
                value: Expr::Literal(Literal::Number(256)),
            },
        ])
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.primitives.insert(primitive.name.clone(), primitive);
    }

    pub fn add_knowledge(&mut self, key: String, techniques: Vec<String>) {
        self.knowledge_base.insert(key, techniques);
    }
}
