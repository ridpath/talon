use walrus::{
    FunctionBuilder, MemoryId, Module, ValType, DataKind,
};
use crate::ast::Command;

/// Emit WebAssembly module based on parsed DSL
pub fn emit_wasm(commands: &[Command], output_path: &str) {
    let mut module = Module::default();

    // 1. Add memory segment
    let memory_id: MemoryId = module.memories.add_local(false, 1, None);
    module.exports.add("memory", memory_id);

    // 2. Import `print` from host env (used for logging from wasm)
    let print_ty = module.types.add(&[ValType::I32], &[]);
    let (print_fn, _) = module.add_import_func("host", "print", print_ty);

    // 3. Create main function
    let mut main_fn = FunctionBuilder::new(&mut module.types, &[], &[]);
    let mut builder = main_fn.func_body();

    let mut offset = 0u32;

    for cmd in commands {
        match cmd {
            Command::Connect { ip, port } => {
                let msg = format!("Connecting to {}:{}\\n", ip, port);
                offset += emit_string(&mut module, memory_id, offset, &msg);
                builder.i32_const(offset as i32);
                builder.call(print_fn);
            }

            Command::Sleep(secs) => {
                let msg = format!("Sleep for {} seconds\\n", secs);
                offset += emit_string(&mut module, memory_id, offset, &msg);
                builder.i32_const(offset as i32);
                builder.call(print_fn);
            }

            Command::Download { url, path } => {
                let msg = format!("Download from {} → {}\\n", url, path);
                offset += emit_string(&mut module, memory_id, offset, &msg);
                builder.i32_const(offset as i32);
                builder.call(print_fn);
            }

            Command::Beacon { url, interval } => {
                let msg = format!("Beacon to {} every {}s\\n", url, interval);
                offset += emit_string(&mut module, memory_id, offset, &msg);
                builder.i32_const(offset as i32);
                builder.call(print_fn);
            }

            _ => {
                let msg = format!("WARNING: [WASM] Unhandled: {:?}\\n", cmd);
                offset += emit_string(&mut module, memory_id, offset, &msg);
                builder.i32_const(offset as i32);
                builder.call(print_fn);
            }
        }
    }

    let main_id = main_fn.finish(vec![], &mut module.funcs);
    module.exports.add("main", main_id);

    module
        .emit_wasm_file(output_path)
        .expect("[ERROR] Failed to write WASM file");

    println!("[OK] [WASM] Output written to {}", output_path);
}

/// Emit a string to memory
fn emit_string(
    module: &mut Module,
    mem_id: MemoryId,
    offset: u32,
    text: &str,
) -> u32 {
    let bytes = text.as_bytes();
    let len = bytes.len() as u32;

    module.data.add(
        DataKind::Active(walrus::ActiveData {
            memory: mem_id,
            location: walrus::ActiveDataLocation::Absolute(offset),
        }),
        bytes.to_vec(),
    );

    len + 4 // Alignment buffer
}

