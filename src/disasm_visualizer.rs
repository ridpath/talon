// Advanced disassembler with visualization support
// Built on Capstone with enhanced output formatting

use capstone::prelude::*;
use std::fs;
use std::collections::HashMap;

pub struct DisassemblerVisualizer {
    cs: Capstone,
    annotations: HashMap<u64, String>,
}

impl DisassemblerVisualizer {
    pub fn new_x64() -> Result<Self, String> {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .detail(true)
            .build()
            .map_err(|e| format!("Capstone initialization failed: {:?}", e))?;
        
        Ok(DisassemblerVisualizer {
            cs,
            annotations: HashMap::new(),
        })
    }
    
    pub fn new_x86() -> Result<Self, String> {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode32)
            .detail(true)
            .build()
            .map_err(|e| format!("Capstone initialization failed: {:?}", e))?;
        
        Ok(DisassemblerVisualizer {
            cs,
            annotations: HashMap::new(),
        })
    }
    
    pub fn new_arm() -> Result<Self, String> {
        let cs = Capstone::new()
            .arm()
            .mode(arch::arm::ArchMode::Arm)
            .detail(true)
            .build()
            .map_err(|e| format!("Capstone initialization failed: {:?}", e))?;
        
        Ok(DisassemblerVisualizer {
            cs,
            annotations: HashMap::new(),
        })
    }
    
    pub fn annotate(&mut self, address: u64, comment: String) {
        self.annotations.insert(address, comment);
    }
    
    pub fn disassemble_bytes(&self, bytes: &[u8], start_addr: u64) -> Result<String, String> {
        let insns = self.cs.disasm_all(bytes, start_addr)
            .map_err(|e| format!("Disassembly failed: {:?}", e))?;
        
        let mut output = String::new();
        output.push_str(&format!("{:-<80}\n", ""));
        output.push_str(&format!("{:<18} {:<12} {:<30} {}\n", "ADDRESS", "BYTES", "INSTRUCTION", "COMMENT"));
        output.push_str(&format!("{:-<80}\n", ""));
        
        for insn in insns.iter() {
            let addr = insn.address();
            let bytes_str = self.format_bytes(&insn.bytes());
            let mnemonic = insn.mnemonic().unwrap_or("???");
            let op_str = insn.op_str().unwrap_or("");
            
            let comment = self.annotations.get(&addr)
                .map(|s| format!("; {}", s))
                .unwrap_or_default();
            
            output.push_str(&format!("0x{:016x}  {:<12} {:<10} {:<20} {}\n",
                addr,
                bytes_str,
                mnemonic,
                op_str,
                comment
            ));
        }
        
        output.push_str(&format!("{:-<80}\n", ""));
        
        Ok(output)
    }
    
    pub fn disassemble_file(&self, path: &str, offset: usize, length: usize, base_addr: u64) -> Result<String, String> {
        let data = fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if offset >= data.len() {
            return Err(format!("Offset {} exceeds file size {}", offset, data.len()));
        }
        
        let end = (offset + length).min(data.len());
        let slice = &data[offset..end];
        
        self.disassemble_bytes(slice, base_addr + offset as u64)
    }
    
    pub fn disassemble_function(&self, bytes: &[u8], start_addr: u64) -> Result<FunctionAnalysis, String> {
        let insns = self.cs.disasm_all(bytes, start_addr)
            .map_err(|e| format!("Disassembly failed: {:?}", e))?;
        
        let mut analysis = FunctionAnalysis {
            start_address: start_addr,
            end_address: 0,
            instruction_count: insns.len(),
            basic_blocks: Vec::new(),
            calls: Vec::new(),
            returns: Vec::new(),
            jumps: Vec::new(),
        };
        
        let mut current_block = BasicBlock {
            start: start_addr,
            end: start_addr,
            instructions: Vec::new(),
        };
        
        for insn in insns.iter() {
            let addr = insn.address();
            let mnemonic = insn.mnemonic().unwrap_or("").to_lowercase();
            
            current_block.instructions.push(format!("{} {}", 
                insn.mnemonic().unwrap_or(""),
                insn.op_str().unwrap_or("")
            ));
            current_block.end = addr;
            
            analysis.end_address = addr;
            
            if mnemonic == "call" {
                analysis.calls.push(addr);
            } else if mnemonic == "ret" || mnemonic == "retf" {
                analysis.returns.push(addr);
                analysis.basic_blocks.push(current_block.clone());
                current_block = BasicBlock {
                    start: addr + insn.bytes().len() as u64,
                    end: addr + insn.bytes().len() as u64,
                    instructions: Vec::new(),
                };
            } else if mnemonic.starts_with('j') {
                analysis.jumps.push(addr);
                analysis.basic_blocks.push(current_block.clone());
                current_block = BasicBlock {
                    start: addr + insn.bytes().len() as u64,
                    end: addr + insn.bytes().len() as u64,
                    instructions: Vec::new(),
                };
            }
        }
        
        if !current_block.instructions.is_empty() {
            analysis.basic_blocks.push(current_block);
        }
        
        Ok(analysis)
    }
    
    pub fn find_instructions(&self, bytes: &[u8], start_addr: u64, pattern: &str) -> Result<Vec<u64>, String> {
        let insns = self.cs.disasm_all(bytes, start_addr)
            .map_err(|e| format!("Disassembly failed: {:?}", e))?;
        
        let pattern_lower = pattern.to_lowercase();
        let mut matches = Vec::new();
        
        for insn in insns.iter() {
            let full_insn = format!("{} {}",
                insn.mnemonic().unwrap_or(""),
                insn.op_str().unwrap_or("")
            ).to_lowercase();
            
            if full_insn.contains(&pattern_lower) {
                matches.push(insn.address());
            }
        }
        
        Ok(matches)
    }
    
    fn format_bytes(&self, bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    pub fn generate_control_flow_graph(&self, analysis: &FunctionAnalysis) -> String {
        let mut graph = String::new();
        
        graph.push_str("digraph CFG {\n");
        graph.push_str("  node [shape=box];\n");
        
        for (i, bb) in analysis.basic_blocks.iter().enumerate() {
            let label = format!("BB{} (0x{:x}-0x{:x})", i, bb.start, bb.end);
            graph.push_str(&format!("  bb{} [label=\"{}\"];\n", i, label));
            
            if i + 1 < analysis.basic_blocks.len() {
                graph.push_str(&format!("  bb{} -> bb{};\n", i, i + 1));
            }
        }
        
        graph.push_str("}\n");
        
        graph
    }
}

#[derive(Debug, Clone)]
pub struct FunctionAnalysis {
    pub start_address: u64,
    pub end_address: u64,
    pub instruction_count: usize,
    pub basic_blocks: Vec<BasicBlock>,
    pub calls: Vec<u64>,
    pub returns: Vec<u64>,
    pub jumps: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub start: u64,
    pub end: u64,
    pub instructions: Vec<String>,
}

impl FunctionAnalysis {
    pub fn print_summary(&self) {
        println!("Function Analysis:");
        println!("  Address range: 0x{:x} - 0x{:x}", self.start_address, self.end_address);
        println!("  Instructions: {}", self.instruction_count);
        println!("  Basic blocks: {}", self.basic_blocks.len());
        println!("  Calls: {}", self.calls.len());
        println!("  Returns: {}", self.returns.len());
        println!("  Jumps: {}", self.jumps.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassembler_x64() {
        let disasm = DisassemblerVisualizer::new_x64().unwrap();
        
        let code = vec![
            0x55,                   // push rbp
            0x48, 0x89, 0xe5,       // mov rbp, rsp
            0x5d,                   // pop rbp
            0xc3,                   // ret
        ];
        
        let result = disasm.disassemble_bytes(&code, 0x400000);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("push"));
        assert!(output.contains("mov"));
        assert!(output.contains("ret"));
    }

    #[test]
    fn test_function_analysis() {
        let disasm = DisassemblerVisualizer::new_x64().unwrap();
        
        let code = vec![
            0x55,                   // push rbp
            0x48, 0x89, 0xe5,       // mov rbp, rsp
            0xe8, 0x00, 0x00, 0x00, 0x00,  // call
            0x5d,                   // pop rbp
            0xc3,                   // ret
        ];
        
        let analysis = disasm.disassemble_function(&code, 0x400000);
        assert!(analysis.is_ok());
        
        let func = analysis.unwrap();
        assert_eq!(func.calls.len(), 1);
        assert_eq!(func.returns.len(), 1);
    }

    #[test]
    fn test_find_instructions() {
        let disasm = DisassemblerVisualizer::new_x64().unwrap();
        
        let code = vec![
            0x5f,                   // pop rdi
            0x5e,                   // pop rsi
            0xc3,                   // ret
        ];
        
        let matches = disasm.find_instructions(&code, 0x400000, "pop");
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);
    }
}
