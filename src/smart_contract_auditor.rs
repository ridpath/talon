// ═══════════════════════════════════════════════════════════════════════════
// ADVANCED SMART CONTRACT AUDITING & SECURITY ANALYSIS FRAMEWORK
// Comprehensive vulnerability detection and exploit generation for Solidity contracts
// ═══════════════════════════════════════════════════════════════════════════

#![allow(clippy::needless_range_loop)]

use serde::{Deserialize, Serialize};
use std::fs;

// ═══════════════════════════════════════════════════════════════════════════
// VULNERABILITY DETECTION ENGINE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilityType {
    // Classic Vulnerabilities
    Reentrancy,
    IntegerOverflow,
    IntegerUnderflow,
    UnprotectedSelfDestruct,
    UnprotectedEther,

    // DeFi-Specific
    FlashLoanAttack,
    FrontRunning,
    BackRunning,
    Sandwiching,
    OracleManipulation,
    PriceManipulation,

    // Access Control
    UnprotectedFunction,
    MissingAccessControl,
    WeakRandomness,
    TxOriginAuthentication,
    DelegateCallInjection,

    // Gas & Performance
    UnboundedLoop,
    GasGriefing,
    StorageCollision,

    // Advanced
    TimestampDependence,
    BlockhashWeakness,
    SignatureReplay,
    ERC20IssuesApproveRace,
    UncheckedCall,

    // MEV
    MEVVulnerable,
    SlippageTooHigh,
    NoDeadline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub vuln_type: VulnerabilityType,
    pub severity: Severity,
    pub location: CodeLocation,
    pub description: String,
    pub recommendation: String,
    pub exploitability_score: u8, // 0-100
    pub poc_code: Option<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Critical, // Immediate funds at risk
    High,     // Significant risk, exploitable
    Medium,   // Potential risk, complex exploit
    Low,      // Minor issue
    Info,     // Informational
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub function: Option<String>,
    pub snippet: String,
}

pub struct SmartContractAuditor {
    contract_path: String,
    source_code: String,
    bytecode: Option<Vec<u8>>,
    vulnerabilities: Vec<Vulnerability>,
    contract_abi: Option<String>,
}

impl SmartContractAuditor {
    pub fn new(contract_path: String) -> Result<Self, String> {
        let source_code = fs::read_to_string(&contract_path)
            .map_err(|e| format!("Failed to read contract: {}", e))?;

        println!("[AUDIT] Initializing Smart Contract Security Auditor");
        println!("[AUDIT] Contract: {}", contract_path);
        println!("[AUDIT] Source: {} lines", source_code.lines().count());

        Ok(SmartContractAuditor {
            contract_path,
            source_code,
            bytecode: None,
            vulnerabilities: Vec::new(),
            contract_abi: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMPREHENSIVE VULNERABILITY SCANNING
    // ═══════════════════════════════════════════════════════════════════════

    pub fn scan_all_vulnerabilities(&mut self) -> Result<AuditReport, String> {
        println!("[AUDIT] Starting comprehensive vulnerability scan...");

        self.detect_reentrancy()?;
        self.detect_integer_issues()?;
        self.detect_access_control()?;
        self.detect_unchecked_calls()?;
        self.detect_timestamp_dependence()?;
        self.detect_tx_origin()?;
        self.detect_delegatecall_issues()?;
        self.detect_selfdestruct_issues()?;
        self.detect_weak_randomness()?;
        self.detect_unbounded_loops()?;
        self.detect_storage_collision()?;
        self.detect_signature_replay()?;
        self.detect_erc20_issues()?;
        self.detect_defi_specific()?;
        self.detect_mev_vulnerabilities()?;
        self.detect_flashloan_attacks()?;
        self.detect_oracle_manipulation()?;
        self.detect_front_running()?;

        println!(
            "[AUDIT] [OK] Scan complete: {} vulnerabilities found",
            self.vulnerabilities.len()
        );

        Ok(self.generate_report())
    }

    fn detect_reentrancy(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for reentrancy vulnerabilities...");

        let lines: Vec<&str> = self.source_code.lines().collect();
        let mut state_changes_after_call = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            // Detect external calls
            if line.contains(".call{")
                || line.contains(".call(")
                || line.contains(".transfer(")
                || line.contains(".send(")
            {
                // Check if state changes occur after the call
                for future_idx in (idx + 1)..lines.len().min(idx + 20) {
                    let future_line = lines[future_idx];

                    if future_line.contains("=") && !future_line.trim().starts_with("//") {
                        // State change after external call - reentrancy risk!

                        if !future_line.contains("require") && !future_line.contains("assert") {
                            state_changes_after_call.push((idx, future_idx));

                            self.vulnerabilities.push(Vulnerability {
                                vuln_type: VulnerabilityType::Reentrancy,
                                severity: Severity::Critical,
                                location: CodeLocation {
                                    file: self.contract_path.clone(),
                                    line: idx + 1,
                                    column: 0,
                                    function: self.extract_function_name(&lines, idx),
                                    snippet: line.trim().to_string(),
                                },
                                description: format!(
                                    "Reentrancy vulnerability: External call at line {} followed by state change at line {}. \
                                    An attacker can recursively call this function before state updates complete.",
                                    idx + 1, future_idx + 1
                                ),
                                recommendation: "Apply checks-effects-interactions pattern. Move all state changes before external calls, or use ReentrancyGuard modifier.".to_string(),
                                exploitability_score: 95,
                                poc_code: Some(self.generate_reentrancy_exploit(idx)),
                                references: vec![
                                    "https://consensys.github.io/smart-contract-best-practices/attacks/reentrancy/".to_string()
                                ],
                            });
                        }
                    }
                }
            }
        }

        println!(
            "[AUDIT]   → Found {} reentrancy issues",
            state_changes_after_call.len()
        );
        Ok(())
    }

    fn detect_integer_issues(&mut self) -> Result<(), String> {
        println!("[AUDIT]  Scanning for integer overflow/underflow...");

        let lines: Vec<&str> = self.source_code.lines().collect();
        let mut unchecked_math = 0;

        // Check Solidity version
        let has_solidity_08 = self.source_code.contains("pragma solidity ^0.8")
            || self.source_code.contains("pragma solidity >=0.8");

        if !has_solidity_08 {
            for (idx, line) in lines.iter().enumerate() {
                if (line.contains(" + ")
                    || line.contains(" - ")
                    || line.contains(" * ")
                    || line.contains(" / "))
                    && !line.contains("SafeMath")
                    && !line.trim().starts_with("//")
                {
                    unchecked_math += 1;

                    self.vulnerabilities.push(Vulnerability {
                        vuln_type: VulnerabilityType::IntegerOverflow,
                        severity: Severity::High,
                        location: CodeLocation {
                            file: self.contract_path.clone(),
                            line: idx + 1,
                            column: 0,
                            function: self.extract_function_name(&lines, idx),
                            snippet: line.trim().to_string(),
                        },
                        description: format!(
                            "Integer overflow/underflow risk at line {}. Arithmetic operation without SafeMath in Solidity <0.8",
                            idx + 1
                        ),
                        recommendation: "Use SafeMath library or upgrade to Solidity ^0.8.0 which has built-in overflow protection.".to_string(),
                        exploitability_score: 85,
                        poc_code: Some(self.generate_overflow_exploit()),
                        references: vec!["https://github.com/ethereum/solidity/issues/796".to_string()],
                    });
                }
            }
        }

        println!(
            "[AUDIT]   → Found {} unchecked arithmetic operations",
            unchecked_math
        );
        Ok(())
    }

    fn detect_access_control(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for access control issues...");

        let lines: Vec<&str> = self.source_code.lines().collect();
        let mut unprotected_functions = 0;

        for (idx, line) in lines.iter().enumerate() {
            // Check for public/external functions
            if line.contains("function") && (line.contains("public") || line.contains("external")) {
                let is_protected = self.function_has_modifier(&lines, idx);
                let is_view_pure = line.contains("view") || line.contains("pure");
                let is_constructor = line.contains("constructor");

                if !is_protected && !is_view_pure && !is_constructor {
                    unprotected_functions += 1;

                    self.vulnerabilities.push(Vulnerability {
                        vuln_type: VulnerabilityType::MissingAccessControl,
                        severity: Severity::High,
                        location: CodeLocation {
                            file: self.contract_path.clone(),
                            line: idx + 1,
                            column: 0,
                            function: self.extract_function_name(&lines, idx),
                            snippet: line.trim().to_string(),
                        },
                        description: format!(
                            "Unprotected function at line {}. No access control modifier detected (onlyOwner, etc.)",
                            idx + 1
                        ),
                        recommendation: "Add appropriate access control modifiers (onlyOwner, onlyAdmin, etc.) or use OpenZeppelin's AccessControl.".to_string(),
                        exploitability_score: 90,
                        poc_code: None,
                        references: vec!["https://docs.openzeppelin.com/contracts/4.x/access-control".to_string()],
                    });
                }
            }
        }

        println!(
            "[AUDIT]   → Found {} unprotected functions",
            unprotected_functions
        );
        Ok(())
    }

    fn detect_unchecked_calls(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for unchecked external calls...");

        let lines: Vec<&str> = self.source_code.lines().collect();
        let mut unchecked_calls = 0;

        for (idx, line) in lines.iter().enumerate() {
            if (line.contains(".call(") || line.contains(".send(") || line.contains(".transfer("))
                && !line.trim().starts_with("//")
            {
                // Check if return value is checked
                let is_checked = line.contains("require(")
                    || line.contains("if (")
                    || line.contains("assert(")
                    || line.contains("success");

                if !is_checked && line.contains(".call(") {
                    unchecked_calls += 1;

                    self.vulnerabilities.push(Vulnerability {
                        vuln_type: VulnerabilityType::UncheckedCall,
                        severity: Severity::Medium,
                        location: CodeLocation {
                            file: self.contract_path.clone(),
                            line: idx + 1,
                            column: 0,
                            function: self.extract_function_name(&lines, idx),
                            snippet: line.trim().to_string(),
                        },
                        description: format!(
                            "Unchecked .call() at line {}. Return value not verified, call failure will be silently ignored.",
                            idx + 1
                        ),
                        recommendation: "Always check return value: (bool success, ) = addr.call(...); require(success, \"Call failed\");".to_string(),
                        exploitability_score: 70,
                        poc_code: None,
                        references: vec!["https://swcregistry.io/docs/SWC-104".to_string()],
                    });
                }
            }
        }

        println!(
            "[AUDIT]   → Found {} unchecked external calls",
            unchecked_calls
        );
        Ok(())
    }

    fn detect_timestamp_dependence(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for timestamp dependence...");

        if self.source_code.contains("block.timestamp") || self.source_code.contains("now") {
            let lines: Vec<&str> = self.source_code.lines().collect();

            for (idx, line) in lines.iter().enumerate() {
                if (line.contains("block.timestamp") || line.contains("now"))
                    && (line.contains("==") || line.contains("<") || line.contains(">"))
                    && !line.trim().starts_with("//")
                {
                    self.vulnerabilities.push(Vulnerability {
                        vuln_type: VulnerabilityType::TimestampDependence,
                        severity: Severity::Low,
                        location: CodeLocation {
                            file: self.contract_path.clone(),
                            line: idx + 1,
                            column: 0,
                            function: self.extract_function_name(&lines, idx),
                            snippet: line.trim().to_string(),
                        },
                        description: "Timestamp dependence detected. Miners can manipulate block.timestamp within ~15 seconds.".to_string(),
                        recommendation: "Avoid using timestamps for critical logic. Use block numbers or external oracles.".to_string(),
                        exploitability_score: 40,
                        poc_code: None,
                        references: vec!["https://swcregistry.io/docs/SWC-116".to_string()],
                    });
                }
            }
        }

        Ok(())
    }

    fn detect_tx_origin(&mut self) -> Result<(), String> {
        println!("[AUDIT]  Scanning for tx.origin usage...");

        if self.source_code.contains("tx.origin") {
            let lines: Vec<&str> = self.source_code.lines().collect();

            for (idx, line) in lines.iter().enumerate() {
                if line.contains("tx.origin") && !line.trim().starts_with("//") {
                    self.vulnerabilities.push(Vulnerability {
                        vuln_type: VulnerabilityType::TxOriginAuthentication,
                        severity: Severity::High,
                        location: CodeLocation {
                            file: self.contract_path.clone(),
                            line: idx + 1,
                            column: 0,
                            function: self.extract_function_name(&lines, idx),
                            snippet: line.trim().to_string(),
                        },
                        description: "tx.origin used for authentication. Vulnerable to phishing attacks via malicious contracts.".to_string(),
                        recommendation: "Use msg.sender instead of tx.origin for authentication.".to_string(),
                        exploitability_score: 85,
                        poc_code: Some(self.generate_txorigin_exploit()),
                        references: vec!["https://swcregistry.io/docs/SWC-115".to_string()],
                    });
                }
            }
        }

        Ok(())
    }

    fn detect_delegatecall_issues(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for delegatecall vulnerabilities...");

        if self.source_code.contains("delegatecall") {
            let lines: Vec<&str> = self.source_code.lines().collect();

            for (idx, line) in lines.iter().enumerate() {
                if line.contains("delegatecall") && !line.trim().starts_with("//") {
                    let is_controlled = line.contains("msg.sender")
                        || line.contains("input")
                        || line.contains("_target");

                    self.vulnerabilities.push(Vulnerability {
                        vuln_type: VulnerabilityType::DelegateCallInjection,
                        severity: if is_controlled { Severity::Critical } else { Severity::High },
                        location: CodeLocation {
                            file: self.contract_path.clone(),
                            line: idx + 1,
                            column: 0,
                            function: self.extract_function_name(&lines, idx),
                            snippet: line.trim().to_string(),
                        },
                        description: "Delegatecall to user-controlled address allows arbitrary code execution in contract's context.".to_string(),
                        recommendation: "Never delegatecall to user-supplied addresses. Whitelist allowed contract addresses.".to_string(),
                        exploitability_score: 98,
                        poc_code: Some(self.generate_delegatecall_exploit()),
                        references: vec!["https://swcregistry.io/docs/SWC-112".to_string()],
                    });
                }
            }
        }

        Ok(())
    }

    fn detect_selfdestruct_issues(&mut self) -> Result<(), String> {
        println!("[AUDIT]  Scanning for selfdestruct issues...");

        if self.source_code.contains("selfdestruct") || self.source_code.contains("suicide") {
            self.vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::UnprotectedSelfDestruct,
                severity: Severity::Critical,
                location: CodeLocation {
                    file: self.contract_path.clone(),
                    line: 0,
                    column: 0,
                    function: None,
                    snippet: "selfdestruct found".to_string(),
                },
                description: "Contract contains selfdestruct. If unprotected, attacker can destroy contract and steal funds.".to_string(),
                recommendation: "Protect selfdestruct with multi-sig or remove entirely. Consider using upgrade patterns instead.".to_string(),
                exploitability_score: 95,
                poc_code: None,
                references: vec!["https://swcregistry.io/docs/SWC-106".to_string()],
            });
        }

        Ok(())
    }

    fn detect_weak_randomness(&mut self) -> Result<(), String> {
        println!("[AUDIT]  Scanning for weak randomness...");

        let weak_sources = [
            "block.timestamp",
            "block.number",
            "blockhash",
            "block.difficulty",
        ];

        for source in &weak_sources {
            if self.source_code.contains(source) && self.source_code.contains("random") {
                self.vulnerabilities.push(Vulnerability {
                    vuln_type: VulnerabilityType::WeakRandomness,
                    severity: Severity::High,
                    location: CodeLocation {
                        file: self.contract_path.clone(),
                        line: 0,
                        column: 0,
                        function: None,
                        snippet: format!("Using {} for randomness", source),
                    },
                    description: format!("Weak randomness: Using {} for random number generation. Miners can predict/manipulate.", source),
                    recommendation: "Use Chainlink VRF or commit-reveal schemes for secure randomness.".to_string(),
                    exploitability_score: 80,
                    poc_code: None,
                    references: vec!["https://github.com/smartcontractkit/chainlink".to_string()],
                });
            }
        }

        Ok(())
    }

    fn detect_unbounded_loops(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for unbounded loops...");

        let lines: Vec<&str> = self.source_code.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            if line.contains("for") && (line.contains(".length") || line.contains("array")) {
                self.vulnerabilities.push(Vulnerability {
                    vuln_type: VulnerabilityType::UnboundedLoop,
                    severity: Severity::Medium,
                    location: CodeLocation {
                        file: self.contract_path.clone(),
                        line: idx + 1,
                        column: 0,
                        function: self.extract_function_name(&lines, idx),
                        snippet: line.trim().to_string(),
                    },
                    description: "Unbounded loop over array. Can cause out-of-gas errors if array grows too large.".to_string(),
                    recommendation: "Implement pagination, limit array size, or use pull-over-push pattern.".to_string(),
                    exploitability_score: 60,
                    poc_code: None,
                    references: vec!["https://consensys.github.io/smart-contract-best-practices/attacks/denial-of-service/".to_string()],
                });
            }
        }

        Ok(())
    }

    fn detect_storage_collision(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for storage collision risks...");

        if self.source_code.contains("delegatecall")
            && (self.source_code.contains("contract") && self.source_code.contains("is"))
        {
            self.vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::StorageCollision,
                severity: Severity::Medium,
                location: CodeLocation {
                    file: self.contract_path.clone(),
                    line: 0,
                    column: 0,
                    function: None,
                    snippet: "delegatecall + inheritance detected".to_string(),
                },
                description: "Storage collision risk with delegatecall and inheritance. Variable slots may conflict.".to_string(),
                recommendation: "Use EIP-1967 storage slots or carefully manage storage layout in proxy patterns.".to_string(),
                exploitability_score: 70,
                poc_code: None,
                references: vec!["https://eips.ethereum.org/EIPS/eip-1967".to_string()],
            });
        }

        Ok(())
    }

    fn detect_signature_replay(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for signature replay vulnerabilities...");

        if self.source_code.contains("ecrecover") || self.source_code.contains("signature") {
            let has_nonce = self.source_code.contains("nonce");
            let has_chainid =
                self.source_code.contains("chainid") || self.source_code.contains("block.chainid");

            if !has_nonce || !has_chainid {
                self.vulnerabilities.push(Vulnerability {
                    vuln_type: VulnerabilityType::SignatureReplay,
                    severity: Severity::High,
                    location: CodeLocation {
                        file: self.contract_path.clone(),
                        line: 0,
                        column: 0,
                        function: None,
                        snippet: "signature verification found".to_string(),
                    },
                    description: "Signature replay vulnerability. Missing nonce or chainId in signature verification.".to_string(),
                    recommendation: "Include nonce and chainId in signed messages. Follow EIP-712 standard.".to_string(),
                    exploitability_score: 85,
                    poc_code: None,
                    references: vec!["https://eips.ethereum.org/EIPS/eip-712".to_string()],
                });
            }
        }

        Ok(())
    }

    fn detect_erc20_issues(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for ERC20-specific issues...");

        // Check for approve race condition
        if self.source_code.contains("function approve")
            && !self.source_code.contains("increaseAllowance") {
                self.vulnerabilities.push(Vulnerability {
                    vuln_type: VulnerabilityType::ERC20IssuesApproveRace,
                    severity: Severity::Medium,
                    location: CodeLocation {
                        file: self.contract_path.clone(),
                        line: 0,
                        column: 0,
                        function: Some("approve".to_string()),
                        snippet: "ERC20 approve function".to_string(),
                    },
                    description: "ERC20 approve race condition. Changing allowance from non-zero to non-zero is unsafe.".to_string(),
                    recommendation: "Implement increaseAllowance/decreaseAllowance or require approve to zero first.".to_string(),
                    exploitability_score: 65,
                    poc_code: None,
                    references: vec!["https://docs.google.com/document/d/1YLPtQxZu1UAvO9cZ1O2RPXBbT0mooh4DYKjA_jp-RLM".to_string()],
                });
            }

        Ok(())
    }

    fn detect_defi_specific(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for DeFi-specific vulnerabilities...");

        // Check for price manipulation via direct balance checks
        if self.source_code.contains("balanceOf(address(this))")
            && (self.source_code.contains("swap") || self.source_code.contains("price"))
        {
            self.vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::PriceManipulation,
                severity: Severity::Critical,
                location: CodeLocation {
                    file: self.contract_path.clone(),
                    line: 0,
                    column: 0,
                    function: None,
                    snippet: "balanceOf used for pricing".to_string(),
                },
                description: "Price calculation based on token balance. Vulnerable to flash loan price manipulation.".to_string(),
                recommendation: "Use time-weighted average price (TWAP) oracles like Uniswap V2/V3 TWAP or Chainlink.".to_string(),
                exploitability_score: 95,
                poc_code: Some(self.generate_flashloan_exploit_template()),
                references: vec!["https://docs.uniswap.org/concepts/protocol/oracle".to_string()],
            });
        }

        Ok(())
    }

    fn detect_mev_vulnerabilities(&mut self) -> Result<(), String> {
        println!("[AUDIT]  Scanning for MEV vulnerabilities...");

        // Check for missing deadline protection
        if (self.source_code.contains("swap") || self.source_code.contains("trade"))
            && !self.source_code.contains("deadline")
        {
            self.vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::NoDeadline,
                severity: Severity::High,
                location: CodeLocation {
                    file: self.contract_path.clone(),
                    line: 0,
                    column: 0,
                    function: None,
                    snippet: "swap without deadline".to_string(),
                },
                description: "Missing deadline parameter. Transaction can be held and executed at unfavorable price.".to_string(),
                recommendation: "Add deadline parameter to all swap functions and validate block.timestamp <= deadline.".to_string(),
                exploitability_score: 80,
                poc_code: None,
                references: vec!["https://www.mev.wiki/".to_string()],
            });
        }

        // Check for high slippage tolerance
        if self.source_code.contains("slippage") {
            // Simplified check - in real implementation would parse actual values
            self.vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::SlippageTooHigh,
                severity: Severity::Medium,
                location: CodeLocation {
                    file: self.contract_path.clone(),
                    line: 0,
                    column: 0,
                    function: None,
                    snippet: "slippage parameter found".to_string(),
                },
                description: "Review slippage tolerance. High slippage exposes users to sandwich attacks.".to_string(),
                recommendation: "Use reasonable slippage (0.5-1% for liquid pairs). Consider MEV-resistant DEX designs.".to_string(),
                exploitability_score: 70,
                poc_code: Some(self.generate_sandwich_attack_template()),
                references: vec!["https://medium.com/coinmonks/defi-sandwich-attack-explain-776f6f43b2fd".to_string()],
            });
        }

        Ok(())
    }

    fn detect_flashloan_attacks(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for flashloan attack vectors...");

        if self.source_code.contains("borrow") || self.source_code.contains("flashLoan") {
            self.vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::FlashLoanAttack,
                severity: Severity::Info,
                location: CodeLocation {
                    file: self.contract_path.clone(),
                    line: 0,
                    column: 0,
                    function: None,
                    snippet: "flash loan functionality".to_string(),
                },
                description: "Flash loan functionality detected. Ensure proper validation of borrowed amounts and state.".to_string(),
                recommendation: "Implement checks to prevent flash loan attacks: verify balances before/after, use TWAP oracles.".to_string(),
                exploitability_score: 75,
                poc_code: None,
                references: vec!["https://aave.com/flash-loans".to_string()],
            });
        }

        Ok(())
    }

    fn detect_oracle_manipulation(&mut self) -> Result<(), String> {
        println!("[AUDIT]  Scanning for oracle manipulation risks...");

        if self.source_code.contains("oracle") || self.source_code.contains("getPrice") {
            let uses_single_source = !self.source_code.contains("median")
                && !self.source_code.contains("average")
                && !self.source_code.contains("multiple");

            if uses_single_source {
                self.vulnerabilities.push(Vulnerability {
                    vuln_type: VulnerabilityType::OracleManipulation,
                    severity: Severity::High,
                    location: CodeLocation {
                        file: self.contract_path.clone(),
                        line: 0,
                        column: 0,
                        function: None,
                        snippet: "oracle usage detected".to_string(),
                    },
                    description: "Single oracle source detected. Vulnerable to oracle manipulation or failure.".to_string(),
                    recommendation: "Use multiple oracle sources and calculate median/average. Consider Chainlink or Band Protocol.".to_string(),
                    exploitability_score: 85,
                    poc_code: None,
                    references: vec!["https://chain.link/education/defi-oracles".to_string()],
                });
            }
        }

        Ok(())
    }

    fn detect_front_running(&mut self) -> Result<(), String> {
        println!("[AUDIT] Scanning for front-running vulnerabilities...");

        // Check for commit-reveal pattern
        if self.source_code.contains("reveal") && !self.source_code.contains("commit") {
            self.vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::FrontRunning,
                severity: Severity::Medium,
                location: CodeLocation {
                    file: self.contract_path.clone(),
                    line: 0,
                    column: 0,
                    function: None,
                    snippet: "reveal without commit".to_string(),
                },
                description: "Reveal function without proper commit phase. Vulnerable to front-running.".to_string(),
                recommendation: "Implement commit-reveal scheme: users commit hash(data+secret), then reveal data+secret later.".to_string(),
                exploitability_score: 70,
                poc_code: None,
                references: vec!["https://ethereum.stackexchange.com/questions/191/how-can-i-securely-generate-a-random-number-in-my-smart-contract".to_string()],
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXPLOIT GENERATION
    // ═══════════════════════════════════════════════════════════════════════

    fn generate_reentrancy_exploit(&self, _line: usize) -> String {
        r#"
// Reentrancy Exploit POC
contract ReentrancyAttack {
    TargetContract target;

    constructor(address _target) {
        target = TargetContract(_target);
    }

    function attack() external payable {
        target.vulnerableFunction{value: msg.value}();
    }

    receive() external payable {
        if (address(target).balance >= 1 ether) {
            target.vulnerableFunction(); // Recursive call
        }
    }
}
        "#.to_string()
    }

    fn generate_overflow_exploit(&self) -> String {
        r#"
// Integer Overflow Exploit POC
function exploit() public {
    uint256 maxInt = type(uint256).max;
    // This will overflow to 0 in Solidity <0.8 without SafeMath
    uint256 result = maxInt + 1;
    // Attacker can manipulate balances, timestamps, etc.
}
        "#
        .to_string()
    }

    fn generate_txorigin_exploit(&self) -> String {
        r#"
// tx.origin Phishing Attack
contract PhishingAttack {
    VulnerableContract target;

    constructor(address _target) {
        target = VulnerableContract(_target);
    }

    function phish() external {
        // Victim calls this, their tx.origin is checked by vulnerable contract
        target.withdrawAll();
        // Funds go to attacker's contract
    }
}
        "#
        .to_string()
    }

    fn generate_delegatecall_exploit(&self) -> String {
        r#"
// Delegatecall Takeover Exploit
contract DelegatecallAttack {
    address public owner; // Storage slot 0

    function pwn() public {
        owner = msg.sender; // Overwrites victim's storage slot 0
    }
}

// Attacker calls: victim.delegatecall(address(attackContract), "pwn()")
// This overwrites victim's owner variable
        "#
        .to_string()
    }

    fn generate_flashloan_exploit_template(&self) -> String {
        r#"
// Flash Loan Price Manipulation Attack
contract FlashLoanAttack {
    function executeAttack() external {
        // 1. Borrow massive amount via flash loan
        uint256 loanAmount = 1000000 * 1e18;
        flashLoanProvider.flashLoan(loanAmount, address(this));
    }

    function onFlashLoan(uint256 amount) external {
        // 2. Manipulate price by dumping tokens
        uniswapPair.swap(amount, 0, address(this), "");

        // 3. Exploit vulnerable contract at manipulated price
        vulnerableContract.buyAtOraclePrice();

        // 4. Restore price
        uniswapPair.swap(0, amount, address(this), "");

        // 5. Repay flash loan + profit
        token.transfer(msg.sender, amount);
    }
}
        "#
        .to_string()
    }

    fn generate_sandwich_attack_template(&self) -> String {
        r#"
// MEV Sandwich Attack
contract SandwichAttack {
    function frontrun(address victim, uint256 amount) external {
        // 1. Detect victim's pending transaction
        // 2. Submit transaction with higher gas to execute first
        dex.swap(token0, token1, amount);
    }

    function backrun(address victim) external {
        // 3. After victim's transaction, reverse the trade
        dex.swap(token1, token0, balance);
        // Profit from price impact
    }
}
        "#
        .to_string()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER FUNCTIONS
    // ═══════════════════════════════════════════════════════════════════════

    fn extract_function_name(&self, lines: &[&str], line_idx: usize) -> Option<String> {
        // Look backwards from line_idx to find function declaration
        for i in (0..=line_idx).rev() {
            if lines[i].contains("function") {
                let parts: Vec<&str> = lines[i].split_whitespace().collect();
                if let Some(pos) = parts.iter().position(|&x| x == "function") {
                    if pos + 1 < parts.len() {
                        let name = parts[pos + 1].trim_end_matches('(');
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }

    fn function_has_modifier(&self, lines: &[&str], line_idx: usize) -> bool {
        // Check if function has modifiers like onlyOwner, onlyAdmin, etc.
        for i in line_idx..(line_idx + 5).min(lines.len()) {
            let line = lines[i];
            if line.contains("only")
                || line.contains("require(msg.sender")
                || line.contains("AccessControl")
                || line.contains("Ownable")
            {
                return true;
            }
            // Stop at opening brace
            if line.contains("{") {
                break;
            }
        }
        false
    }

    fn generate_report(&self) -> AuditReport {
        let critical = self
            .vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, Severity::Critical))
            .count();
        let high = self
            .vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, Severity::High))
            .count();
        let medium = self
            .vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, Severity::Medium))
            .count();
        let low = self
            .vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, Severity::Low))
            .count();
        let info = self
            .vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, Severity::Info))
            .count();

        let risk_score = (critical * 100 + high * 50 + medium * 25 + low * 10) as f64
            / (self.vulnerabilities.len().max(1) as f64);

        AuditReport {
            contract_name: self.contract_path.clone(),
            total_vulnerabilities: self.vulnerabilities.len(),
            critical_count: critical,
            high_count: high,
            medium_count: medium,
            low_count: low,
            info_count: info,
            vulnerabilities: self.vulnerabilities.clone(),
            risk_score,
            recommendations: self.generate_recommendations(),
        }
    }

    fn generate_recommendations(&self) -> Vec<String> {
        vec![
            "Implement comprehensive test coverage including edge cases".to_string(),
            "Use formal verification tools (Certora, K Framework)".to_string(),
            "Deploy on testnet and run extensive tests before mainnet".to_string(),
            "Consider bug bounty program (Immunefi, HackerOne)".to_string(),
            "Implement circuit breakers and pausable patterns".to_string(),
            "Use OpenZeppelin's audited contract libraries".to_string(),
            "Implement multi-sig for privileged operations".to_string(),
            "Monitor contract for unusual activity post-deployment".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub contract_name: String,
    pub total_vulnerabilities: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub vulnerabilities: Vec<Vulnerability>,
    pub risk_score: f64,
    pub recommendations: Vec<String>,
}

impl AuditReport {
    pub fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║          SMART CONTRACT AUDIT REPORT                          ║");
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║ Contract: {:<51} ║", self.contract_name);
        println!("║ Total Issues: {:<47} ║", self.total_vulnerabilities);
        println!("║                                                               ║");
        println!("║ Severity Breakdown:                                           ║");
        println!("║   Critical: {:<49} ║", self.critical_count);
        println!("║   High: {:<53} ║", self.high_count);
        println!("║   Medium: {:<51} ║", self.medium_count);
        println!("║   Low: {:<54} ║", self.low_count);
        println!("║   INFO: {:<52} ║", self.info_count);
        println!("║                                                               ║");
        println!(
            "║ Risk Score: {:<6.2} / 100                                      ║",
            self.risk_score
        );
        println!("╚═══════════════════════════════════════════════════════════════╝\n");

        if self.critical_count > 0 || self.high_count > 0 {
            println!("WARNING: CRITICAL ISSUES FOUND - DO NOT DEPLOY TO MAINNET\n");
        }

        for (idx, vuln) in self.vulnerabilities.iter().enumerate() {
            let prefix = match vuln.severity {
                Severity::Critical => "[CRITICAL]",
                Severity::High => "[HIGH]",
                Severity::Medium => "[MEDIUM]",
                Severity::Low => "[LOW]",
                Severity::Info => "[INFO]",
            };

            println!(
                "\n{}═══════════════════════════════════════════════════════════════",
                prefix
            );
            println!("{} Issue #{}: {:?}", prefix, idx + 1, vuln.vuln_type);
            println!(
                "{}═══════════════════════════════════════════════════════════════",
                prefix
            );
            println!("Location: {}:{}", vuln.location.file, vuln.location.line);
            if let Some(func) = &vuln.location.function {
                println!("Function: {}", func);
            }
            println!("Description: {}", vuln.description);
            println!("Recommendation: {}", vuln.recommendation);
            println!("Exploitability: {}/100", vuln.exploitability_score);

            if let Some(poc) = &vuln.poc_code {
                println!("\nProof of Concept:\n{}", poc);
            }

            println!("References:");
            for ref_url in &vuln.references {
                println!("   {}", ref_url);
            }
        }

        println!("\n\nRECOMMENDATIONS:");
        for (idx, rec) in self.recommendations.iter().enumerate() {
            println!("  {}. {}", idx + 1, rec);
        }
    }
}
