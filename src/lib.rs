#![allow(dead_code)]

pub mod ast;
pub mod build_cache;
pub mod codegen;
pub mod error_context;
pub mod interpreter;
#[cfg(feature = "llvm")]
pub mod llvm_codegen;
pub mod parser;
pub mod parser_utils;
pub mod wasm_codegen;

pub mod binary_analyzer;
pub mod binary_patch;
pub mod cyclic_tools;
pub mod elf_tools;
pub mod encoding_tools;
pub mod fmtstr_tools;
pub mod format_string;
pub mod heap_grooming;
pub mod heap_tools;
pub mod packing_tools;
pub mod rop_gadget_finder;
pub mod rop_tools;
pub mod shellcode_encoders;
pub mod shellcode_library;
pub mod srop_tools;

pub mod advanced_fuzzer;
pub mod ai_exploit_gen;
pub mod auto_offset;
pub mod binary_similarity;
pub mod crypto_tools;
pub mod cve_scanner;
pub mod cyclic_pattern;
pub mod diff_fuzzer;
pub mod disasm_visualizer;
pub mod doc_generator;
pub mod exploit_chaining;
pub mod flag_tools;
pub mod gdb_mi;
pub mod gdb_parser;
pub mod gdb_tools;
pub mod interactive_io;
pub mod interactive_shell;
pub mod kernel_exploiter;
pub mod pty;
pub mod libc_database;
pub mod libc_db;
pub mod parallel_exploit;
pub mod quick_mode;
pub mod quick_pwn;
pub mod repl;
pub mod runtime_safety;
pub mod shellcode_db;
pub mod socket_tools;
pub mod session_state;
pub mod split_screen_debugger;
pub mod ssh_bridge;
#[cfg(feature = "symbolic-execution")]
pub mod symbolic_engine;
pub mod time_travel;
pub mod web_tools;

pub mod ctf_helpers;
pub mod ctf_quick_helpers;
pub mod helpers;
pub mod mitigation_detector;
pub mod oracle;
pub mod output_utils;
pub mod registry;

// OpSec & EDR Evasion (syscalls only on Windows with feature flag)
pub mod opsec;

// Forensics & Anti-Sandbox (Phase 5.5)
pub mod forensics;
