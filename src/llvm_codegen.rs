use crate::ast::{Command, Control};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    targets::{FileType, InitializationConfig, Target, TargetMachine},
    types::BasicTypeEnum,
    values::{BasicValueEnum, FunctionValue},
    IntPredicate, OptimizationLevel,
};
use std::path::Path;

pub fn emit_executable(commands: &[Command], output_path: &str) {
    // Initialize LLVM targets
    Target::initialize_all(&InitializationConfig::default());

    // LLVM core setup
    let context = Context::create();
    let module = context.create_module("talon");
    let builder = context.create_builder();
    let void_type = context.void_type();
    let i32_type = context.i32_type();
    let i8_ptr = context.i8_type().ptr_type(Default::default());

    // Declare C standard functions
    let printf_ty = i32_type.fn_type(&[i8_ptr.into()], true);
    let printf = module.add_function("printf", printf_ty, None);

    let sleep_ty = i32_type.fn_type(&[i32_type.into()], false);
    let sleep_fn = module.add_function("sleep", sleep_ty, None);

    // Entry: main function
    let main_fn = module.add_function("main", void_type.fn_type(&[], false), None);
    let entry_bb = context.append_basic_block(main_fn, "entry");
    builder.position_at_end(entry_bb);

    // Emit user commands
    compile_commands(&context, &module, &builder, commands, printf, sleep_fn);

    builder.build_return(None);

    // Emit object file
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let machine = target
        .create_target_machine(
            &triple,
            "x86-64",
            "",
            OptimizationLevel::Aggressive,
            inkwell::targets::RelocMode::Default,
            inkwell::targets::CodeModel::Default,
        )
        .unwrap();

    machine
        .write_to_file(&module, FileType::Object, Path::new(output_path))
        .expect("LLVM object file write failed");

    println!("[OK] [LLVM] Wrote object file to {}", output_path);
}

fn compile_commands(
    context: &Context,
    module: &Module,
    builder: &Builder,
    commands: &[Command],
    printf: FunctionValue,
    sleep_fn: FunctionValue,
) {
    let i32_type = context.i32_type();

    for cmd in commands {
        match cmd {
            Command::Sleep(secs) => {
                let secs_val = i32_type.const_int(*secs as u64, false);
                builder.build_call(sleep_fn, &[secs_val.into()], "sleep");
            }

            Command::Connect { ip, port } => {
                let msg = format!("🔌 Connecting to {}:{}\\n", ip, port);
                let ptr = builder.build_global_string_ptr(&msg, "connect_msg");
                builder.build_call(printf, &[ptr.as_pointer_value().into()], "printf");
            }

            Command::Download { url, path } => {
                let msg = format!("📥 Download: {} → {}\\n", url, path);
                let ptr = builder.build_global_string_ptr(&msg, "download_msg");
                builder.build_call(printf, &[ptr.as_pointer_value().into()], "printf");
            }

            Command::Beacon { url, interval } => {
                let msg = format!("📡 Beacon to {} every {}s\\n", url, interval);
                let str_ptr = builder.build_global_string_ptr(&msg, "beacon_msg");

                let func = builder.get_insert_block().unwrap().get_parent().unwrap();
                let loop_bb = context.append_basic_block(func, "beacon_loop");
                let after_bb = context.append_basic_block(func, "after_beacon");

                builder.build_unconditional_branch(loop_bb);
                builder.position_at_end(loop_bb);

                builder.build_call(printf, &[str_ptr.as_pointer_value().into()], "printf");
                builder.build_call(
                    sleep_fn,
                    &[i32_type.const_int(*interval, false).into()],
                    "sleep",
                );
                builder.build_unconditional_branch(loop_bb);

                builder.position_at_end(after_bb);
            }

            Command::Control(ctrl) => match ctrl {
                Control::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    // NOTE: This is a stub. You may compile `condition` expressions later.
                    let msg = format!("🔀 IF CONDITION: {}\\n", condition);
                    let ptr = builder.build_global_string_ptr(&msg, "if_msg");
                    builder.build_call(printf, &[ptr.as_pointer_value().into()], "printf");

                    compile_commands(context, module, builder, then_body, printf, sleep_fn);

                    if !else_body.is_empty() {
                        compile_commands(context, module, builder, else_body, printf, sleep_fn);
                    }
                }

                Control::For {
                    var,
                    iterable,
                    body,
                } => {
                    let func = builder.get_insert_block().unwrap().get_parent().unwrap();
                    let loop_bb = context.append_basic_block(func, "for_loop");
                    let after_bb = context.append_basic_block(func, "after_loop");

                    let i32_type = context.i32_type();
                    let var_ptr = builder.build_alloca(i32_type, &format!("{}_ptr", var));
                    let base_val = i32_type.const_int(0, false);
                    builder.build_store(var_ptr, base_val);
                    builder.build_unconditional_branch(loop_bb);

                    builder.position_at_end(loop_bb);
                    let curr_val = builder.build_load(var_ptr, "curr").into_int_value();
                    let limit_val = i32_type.const_int(10, false); // Placeholder

                    let cond =
                        builder.build_int_compare(IntPredicate::ULT, curr_val, limit_val, "cond");
                    let body_bb = context.append_basic_block(func, "for_body");
                    let inc_bb = context.append_basic_block(func, "for_inc");

                    builder.build_conditional_branch(cond, body_bb, after_bb);

                    builder.position_at_end(body_bb);
                    compile_commands(context, module, builder, body, printf, sleep_fn);
                    builder.build_unconditional_branch(inc_bb);

                    builder.position_at_end(inc_bb);
                    let inc_val =
                        builder.build_int_add(curr_val, i32_type.const_int(1, false), "next");
                    builder.build_store(var_ptr, inc_val);
                    builder.build_unconditional_branch(loop_bb);

                    builder.position_at_end(after_bb);
                }
            },

            _ => {
                let stub = format!("⚠️ LLVM: Unhandled command: {:?}\\n", cmd);
                let ptr = builder.build_global_string_ptr(&stub, "stub");
                builder.build_call(printf, &[ptr.as_pointer_value().into()], "printf");
            }
        }
    }
}
