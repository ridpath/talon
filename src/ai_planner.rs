use crate::campaign::{Action, ActionType, CampaignObjective, Prerequisite, Strategy};
use crate::environment_graph::{EnvironmentGraph, NodeId, NodeType};
use std::collections::HashMap;
use std::sync::Arc;

pub struct AIPlanner {
    graph: Arc<EnvironmentGraph>,
}

#[derive(Debug, Clone)]
pub struct PlanningResult {
    pub strategies: Vec<Strategy>,
    pub estimated_success: f64,
    pub estimated_duration: std::time::Duration,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl AIPlanner {
    pub fn new(graph: Arc<EnvironmentGraph>) -> Self {
        AIPlanner { graph }
    }

    pub async fn plan_campaign(
        &self,
        objective: &CampaignObjective,
        start_node: NodeId,
    ) -> Result<PlanningResult, String> {
        let goal_nodes = self.identify_goal_nodes(objective).await?;

        if goal_nodes.is_empty() {
            return Err("No viable paths to objective found".to_string());
        }

        let mut strategies = Vec::new();

        for (index, goal_node) in goal_nodes.iter().enumerate() {
            let paths = self.graph.find_paths(start_node, *goal_node, 10).await;

            if !paths.is_empty() {
                let best_path = &paths[0];
                let actions = self.path_to_actions(best_path).await?;

                let strategy = Strategy {
                    name: format!("Strategy_{}", index + 1),
                    priority: (10 - index as i32).max(1),
                    probability: best_path.success_probability,
                    prerequisites: self.extract_prerequisites(best_path).await,
                    actions,
                    fallback: None,
                    timeout: Some(best_path.estimated_time),
                };

                strategies.push(strategy);
            }
        }

        if strategies.is_empty() {
            return Err("No executable strategies found".to_string());
        }

        let avg_success: f64 =
            strategies.iter().map(|s| s.probability).sum::<f64>() / strategies.len() as f64;
        let max_duration = strategies
            .iter()
            .filter_map(|s| s.timeout)
            .max()
            .unwrap_or(std::time::Duration::from_secs(3600));

        let risk = if avg_success > 0.7 {
            RiskLevel::Low
        } else if avg_success > 0.4 {
            RiskLevel::Medium
        } else if avg_success > 0.2 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        };

        Ok(PlanningResult {
            strategies,
            estimated_success: avg_success,
            estimated_duration: max_duration,
            risk_level: risk,
        })
    }

    async fn identify_goal_nodes(
        &self,
        objective: &CampaignObjective,
    ) -> Result<Vec<NodeId>, String> {
        let nodes = self
            .graph
            .find_nodes_by_type(|node_type| match node_type {
                NodeType::Host { ip, .. } => ip.contains(&objective.target),
                NodeType::User { username, .. } => username.contains(&objective.target),
                _ => false,
            })
            .await;

        if nodes.is_empty() {
            Err(format!(
                "Target {} not found in environment",
                objective.target
            ))
        } else {
            Ok(nodes)
        }
    }

    async fn path_to_actions(
        &self,
        path: &crate::environment_graph::AttackPath,
    ) -> Result<Vec<Action>, String> {
        let mut actions = Vec::new();
        let mut action_id = 1u64;

        for edge_id in &path.edges {
            if let Some(edge) = self.graph.get_edge(*edge_id).await {
                let action_type = match edge.edge_type {
                    crate::environment_graph::EdgeType::NetworkAccess => ActionType::Scan {
                        scan_type: crate::campaign::ScanType::PortScan,
                    },
                    crate::environment_graph::EdgeType::ServiceExploits => ActionType::Exploit {
                        exploit_name: "auto_selected".to_string(),
                    },
                    crate::environment_graph::EdgeType::HasPrivilege => ActionType::PrivEsc {
                        method: "auto".to_string(),
                    },
                    _ => ActionType::Custom("analyze".to_string()),
                };

                actions.push(Action {
                    id: action_id,
                    action_type,
                    parameters: HashMap::new(),
                    expected_outcome: None,
                    retry_on_failure: true,
                });

                action_id += 1;
            }
        }

        Ok(actions)
    }

    async fn extract_prerequisites(
        &self,
        _path: &crate::environment_graph::AttackPath,
    ) -> Vec<Prerequisite> {
        vec![Prerequisite::CredentialsAvailable]
    }
}
