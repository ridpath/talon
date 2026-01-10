use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Campaign {
    pub name: String,
    pub objective: CampaignObjective,
    pub starting_point: String,
    pub constraints: Vec<CampaignConstraint>,
    pub strategies: Vec<Strategy>,
    pub state: Arc<RwLock<CampaignState>>,
    pub execution_graph: Arc<RwLock<ExecutionGraph>>,
}

#[derive(Debug, Clone)]
pub struct CampaignObjective {
    pub goal_type: ObjectiveType,
    pub target: String,
    pub success_criteria: Vec<SuccessCriterion>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectiveType {
    GetShell,
    ElevatePrivileges,
    LateralMovement,
    DataExfiltration,
    Persistence,
    DomainAdmin,
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum SuccessCriterion {
    ShellAccess { target: String },
    PrivilegeLevel { level: String },
    FileAccess { path: String },
    NetworkAccess { network: String },
    Custom { condition: String },
}

#[derive(Debug, Clone)]
pub enum CampaignConstraint {
    AvoidDetection,
    MaxTime(Duration),
    NoDestructive,
    StealthMode,
    MaxRetries(usize),
    RequireStability,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub priority: i32,
    pub probability: f64,
    pub prerequisites: Vec<Prerequisite>,
    pub actions: Vec<Action>,
    pub fallback: Option<Box<Strategy>>,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum Prerequisite {
    TargetReachable { host: String },
    ServiceRunning { service: String, port: u16 },
    VulnerabilityPresent { cve: String },
    CredentialsAvailable,
    ToolAvailable { tool: String },
    Custom { check: String },
}

#[derive(Debug, Clone)]
pub struct Action {
    pub id: u64,
    pub action_type: ActionType,
    pub parameters: HashMap<String, String>,
    pub expected_outcome: Option<String>,
    pub retry_on_failure: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    Scan { scan_type: ScanType },
    Exploit { exploit_name: String },
    Enumerate { target: String },
    PrivEsc { method: String },
    LateralMove { method: String },
    Persistence { method: String },
    Cleanup,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScanType {
    PortScan,
    ServiceFingerprint,
    VulnerabilityScan,
    CredentialEnumeration,
    NetworkDiscovery,
}

#[derive(Debug, Clone)]
pub struct CampaignState {
    pub status: CampaignStatus,
    pub current_strategy: Option<String>,
    pub completed_actions: Vec<u64>,
    pub failed_actions: Vec<(u64, String)>,
    pub discovered_assets: Vec<Asset>,
    pub acquired_credentials: Vec<Credential>,
    pub open_sessions: Vec<Session>,
    pub metrics: CampaignMetrics,
    pub started_at: Option<Instant>,
    pub ended_at: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CampaignStatus {
    NotStarted,
    Planning,
    Executing,
    Paused,
    Succeeded,
    Failed(String),
    Aborted,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub asset_type: AssetType,
    pub identifier: String,
    pub properties: HashMap<String, String>,
    pub discovered_at: Instant,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssetType {
    Host { ip: String, hostname: Option<String> },
    Service { host: String, port: u16, protocol: String },
    Vulnerability { cve: String, severity: String },
    Account { username: String, domain: Option<String> },
    File { path: String },
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Credential {
    pub username: String,
    pub credential_type: CredentialType,
    pub domain: Option<String>,
    pub acquired_from: String,
    pub acquired_at: Instant,
}

#[derive(Debug, Clone)]
pub enum CredentialType {
    Password(String),
    Hash { hash_type: String, value: String },
    Token(String),
    Certificate { cert: String, key: String },
    ApiKey(String),
}

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: u64,
    pub target: String,
    pub session_type: SessionType,
    pub established_at: Instant,
    pub privilege_level: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionType {
    Shell,
    Meterpreter,
    RDP,
    SSH,
    WinRM,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct CampaignMetrics {
    pub actions_attempted: usize,
    pub actions_succeeded: usize,
    pub actions_failed: usize,
    pub hosts_discovered: usize,
    pub vulnerabilities_found: usize,
    pub credentials_acquired: usize,
    pub detection_events: usize,
}

#[derive(Debug, Clone)]
pub struct ExecutionGraph {
    pub nodes: HashMap<u64, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub current_node: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: u64,
    pub action: Action,
    pub status: NodeStatus,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: u64,
    pub to: u64,
    pub condition: Option<EdgeCondition>,
}

#[derive(Debug, Clone)]
pub enum EdgeCondition {
    OnSuccess,
    OnFailure,
    Always,
    Custom(String),
}

impl Campaign {
    pub fn new(name: String, objective: CampaignObjective, starting_point: String) -> Self {
        Campaign {
            name,
            objective,
            starting_point,
            constraints: Vec::new(),
            strategies: Vec::new(),
            state: Arc::new(RwLock::new(CampaignState {
                status: CampaignStatus::NotStarted,
                current_strategy: None,
                completed_actions: Vec::new(),
                failed_actions: Vec::new(),
                discovered_assets: Vec::new(),
                acquired_credentials: Vec::new(),
                open_sessions: Vec::new(),
                metrics: CampaignMetrics {
                    actions_attempted: 0,
                    actions_succeeded: 0,
                    actions_failed: 0,
                    hosts_discovered: 0,
                    vulnerabilities_found: 0,
                    credentials_acquired: 0,
                    detection_events: 0,
                },
                started_at: None,
                ended_at: None,
            })),
            execution_graph: Arc::new(RwLock::new(ExecutionGraph {
                nodes: HashMap::new(),
                edges: Vec::new(),
                current_node: None,
            })),
        }
    }

    pub fn with_constraint(mut self, constraint: CampaignConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn with_strategy(mut self, strategy: Strategy) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub async fn execute(&self) -> Result<CampaignResult, String> {
        {
            let mut state = self.state.write().await;
            state.status = CampaignStatus::Planning;
            state.started_at = Some(Instant::now());
        }

        self.plan_execution().await?;

        {
            let mut state = self.state.write().await;
            state.status = CampaignStatus::Executing;
        }

        let result = self.execute_strategies().await;

        {
            let mut state = self.state.write().await;
            state.ended_at = Some(Instant::now());
            
            match &result {
                Ok(_) => state.status = CampaignStatus::Succeeded,
                Err(e) => state.status = CampaignStatus::Failed(e.clone()),
            }
        }

        result
    }

    async fn plan_execution(&self) -> Result<(), String> {
        let mut sorted_strategies = self.strategies.clone();
        sorted_strategies.sort_by(|a, b| b.priority.cmp(&a.priority));

        for strategy in &sorted_strategies {
            if self.check_prerequisites(&strategy.prerequisites).await? {
                let mut state = self.state.write().await;
                state.current_strategy = Some(strategy.name.clone());
                return Ok(());
            }
        }

        Err("No viable strategy found".to_string())
    }

    async fn check_prerequisites(&self, prerequisites: &[Prerequisite]) -> Result<bool, String> {
        for prereq in prerequisites {
            match prereq {
                Prerequisite::TargetReachable { host } => {
                    if !self.check_reachability(host).await? {
                        return Ok(false);
                    }
                }
                Prerequisite::ServiceRunning { service, port } => {
                    if !self.check_service(service, *port).await? {
                        return Ok(false);
                    }
                }
                Prerequisite::VulnerabilityPresent { cve } => {
                    if !self.check_vulnerability(cve).await? {
                        return Ok(false);
                    }
                }
                Prerequisite::CredentialsAvailable => {
                    let state = self.state.read().await;
                    if state.acquired_credentials.is_empty() {
                        return Ok(false);
                    }
                }
                Prerequisite::ToolAvailable { tool } => {
                    if !self.check_tool(tool).await? {
                        return Ok(false);
                    }
                }
                Prerequisite::Custom { check: _ } => {
                }
            }
        }
        Ok(true)
    }

    async fn check_reachability(&self, _host: &str) -> Result<bool, String> {
        Ok(true)
    }

    async fn check_service(&self, _service: &str, _port: u16) -> Result<bool, String> {
        Ok(true)
    }

    async fn check_vulnerability(&self, _cve: &str) -> Result<bool, String> {
        Ok(true)
    }

    async fn check_tool(&self, _tool: &str) -> Result<bool, String> {
        Ok(true)
    }

    async fn execute_strategies(&self) -> Result<CampaignResult, String> {
        for strategy in &self.strategies {
            if self.check_prerequisites(&strategy.prerequisites).await? {
                match self.execute_strategy(strategy).await {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        if let Some(fallback) = &strategy.fallback {
                            match self.execute_strategy(fallback).await {
                                Ok(result) => return Ok(result),
                                Err(fallback_err) => {
                                    return Err(format!("Primary failed: {}, Fallback failed: {}", e, fallback_err));
                                }
                            }
                        } else {
                            continue;
                        }
                    }
                }
            }
        }

        Err("All strategies failed".to_string())
    }

    async fn execute_strategy(&self, strategy: &Strategy) -> Result<CampaignResult, String> {
        {
            let mut state = self.state.write().await;
            state.current_strategy = Some(strategy.name.clone());
        }

        for action in &strategy.actions {
            let action_result = self.execute_action(action).await;

            let mut state = self.state.write().await;
            state.metrics.actions_attempted += 1;

            match action_result {
                Ok(_) => {
                    state.completed_actions.push(action.id);
                    state.metrics.actions_succeeded += 1;
                }
                Err(e) => {
                    state.failed_actions.push((action.id, e.clone()));
                    state.metrics.actions_failed += 1;

                    if !action.retry_on_failure {
                        return Err(format!("Action {} failed: {}", action.id, e));
                    }
                }
            }
        }

        self.check_objective_completion().await
    }

    async fn execute_action(&self, action: &Action) -> Result<(), String> {
        match &action.action_type {
            ActionType::Scan { scan_type } => self.execute_scan(scan_type, &action.parameters).await,
            ActionType::Exploit { exploit_name } => self.execute_exploit(exploit_name, &action.parameters).await,
            ActionType::Enumerate { target } => self.execute_enumeration(target).await,
            ActionType::PrivEsc { method } => self.execute_privesc(method).await,
            ActionType::LateralMove { method } => self.execute_lateral_move(method).await,
            ActionType::Persistence { method } => self.execute_persistence(method).await,
            ActionType::Cleanup => self.execute_cleanup().await,
            ActionType::Custom(_) => Ok(()),
        }
    }

    async fn execute_scan(&self, _scan_type: &ScanType, _params: &HashMap<String, String>) -> Result<(), String> {
        Ok(())
    }

    async fn execute_exploit(&self, _exploit: &str, _params: &HashMap<String, String>) -> Result<(), String> {
        Ok(())
    }

    async fn execute_enumeration(&self, _target: &str) -> Result<(), String> {
        Ok(())
    }

    async fn execute_privesc(&self, _method: &str) -> Result<(), String> {
        Ok(())
    }

    async fn execute_lateral_move(&self, _method: &str) -> Result<(), String> {
        Ok(())
    }

    async fn execute_persistence(&self, _method: &str) -> Result<(), String> {
        Ok(())
    }

    async fn execute_cleanup(&self) -> Result<(), String> {
        Ok(())
    }

    async fn check_objective_completion(&self) -> Result<CampaignResult, String> {
        let state = self.state.read().await;

        for criterion in &self.objective.success_criteria {
            match criterion {
                SuccessCriterion::ShellAccess { target } => {
                    if !state.open_sessions.iter().any(|s| &s.target == target) {
                        return Err("Objective not met: Shell access not achieved".to_string());
                    }
                }
                SuccessCriterion::PrivilegeLevel { level } => {
                    if !state.open_sessions.iter().any(|s| &s.privilege_level == level) {
                        return Err("Objective not met: Required privilege level not achieved".to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(CampaignResult {
            success: true,
            objective_met: true,
            actions_executed: state.completed_actions.len(),
            duration: state.started_at.map(|start| {
                state.ended_at.unwrap_or_else(Instant::now).duration_since(start)
            }),
            assets_discovered: state.discovered_assets.len(),
            sessions_opened: state.open_sessions.len(),
        })
    }

    pub async fn pause(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.status = CampaignStatus::Paused;
        Ok(())
    }

    pub async fn abort(&self) -> Result<(), String> {
        let mut state = self.state.write().await;
        state.status = CampaignStatus::Aborted;
        state.ended_at = Some(Instant::now());
        Ok(())
    }

    pub async fn get_status(&self) -> CampaignStatus {
        let state = self.state.read().await;
        state.status.clone()
    }

    pub async fn get_metrics(&self) -> CampaignMetrics {
        let state = self.state.read().await;
        state.metrics.clone()
    }
}

#[derive(Debug, Clone)]
pub struct CampaignResult {
    pub success: bool,
    pub objective_met: bool,
    pub actions_executed: usize,
    pub duration: Option<Duration>,
    pub assets_discovered: usize,
    pub sessions_opened: usize,
}

pub struct CampaignBuilder {
    name: String,
    objective: Option<CampaignObjective>,
    starting_point: Option<String>,
    constraints: Vec<CampaignConstraint>,
    strategies: Vec<Strategy>,
}

impl CampaignBuilder {
    pub fn new(name: String) -> Self {
        CampaignBuilder {
            name,
            objective: None,
            starting_point: None,
            constraints: Vec::new(),
            strategies: Vec::new(),
        }
    }

    pub fn with_objective(mut self, objective: CampaignObjective) -> Self {
        self.objective = Some(objective);
        self
    }

    pub fn with_starting_point(mut self, starting_point: String) -> Self {
        self.starting_point = Some(starting_point);
        self
    }

    pub fn add_constraint(mut self, constraint: CampaignConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    pub fn add_strategy(mut self, strategy: Strategy) -> Self {
        self.strategies.push(strategy);
        self
    }

    pub fn build(self) -> Result<Campaign, String> {
        let objective = self.objective.ok_or("Objective is required")?;
        let starting_point = self.starting_point.ok_or("Starting point is required")?;

        let mut campaign = Campaign::new(self.name, objective, starting_point);
        
        for constraint in self.constraints {
            campaign = campaign.with_constraint(constraint);
        }
        
        for strategy in self.strategies {
            campaign = campaign.with_strategy(strategy);
        }

        Ok(campaign)
    }
}
