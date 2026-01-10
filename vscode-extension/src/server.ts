import {
    createConnection,
    TextDocuments,
    ProposedFeatures,
    InitializeParams,
    CompletionItem,
    CompletionItemKind,
    TextDocumentSyncKind,
    InitializeResult,
    Hover
} from 'vscode-languageserver/node';

import { TextDocument } from 'vscode-languageserver-textdocument';

const connection = createConnection(ProposedFeatures.all);
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

connection.onInitialize((params: InitializeParams) => {
    const result: InitializeResult = {
        capabilities: {
            textDocumentSync: TextDocumentSyncKind.Incremental,
            completionProvider: {
                resolveProvider: true,
                triggerCharacters: ['.', '(']
            },
            hoverProvider: true
        }
    };
    return result;
});

const builtinFunctions = [
    { name: 'process_attach', signature: 'process_attach(pid_or_name)', description: 'Attach to a running process by PID or name' },
    { name: 'process_detach', signature: 'process_detach(pid)', description: 'Detach from an attached process' },
    { name: 'process_suspend', signature: 'process_suspend(pid)', description: 'Suspend a running process' },
    { name: 'process_resume', signature: 'process_resume(pid)', description: 'Resume a suspended process' },
    { name: 'process_kill', signature: 'process_kill(pid)', description: 'Terminate a process' },
    { name: 'process_modules', signature: 'process_modules(pid)', description: 'List all loaded modules in a process' },
    { name: 'mem_read', signature: 'mem_read(pid, address, size)', description: 'Read memory from target process' },
    { name: 'mem_write', signature: 'mem_write(pid, address, data)', description: 'Write memory to target process' },
    { name: 'mem_scan', signature: 'mem_scan(pid, pattern)', description: 'Scan process memory for byte pattern' },
    { name: 'mem_alloc', signature: 'mem_alloc(pid, size)', description: 'Allocate memory in target process' },
    { name: 'mem_free', signature: 'mem_free(pid, address)', description: 'Free allocated memory' },
    { name: 'mem_protect', signature: 'mem_protect(pid, address, protection)', description: 'Change memory protection flags' },
    { name: 'pointer_chain', signature: 'pointer_chain(pid, offsets)', description: 'Follow multi-level pointer chain' },
    { name: 'inject_asm', signature: 'inject_asm(pid, address, asm_code)', description: 'Inject assembly code at address' },
    { name: 'anticheat_detect', signature: 'anticheat_detect()', description: 'Detect anti-cheat systems' },
    { name: 'kernel_driver_status', signature: 'kernel_driver_status(name)', description: 'Check kernel driver status' },
    { name: 'stealth_read', signature: 'stealth_read(pid, address, size)', description: 'Stealthy memory read' },
    { name: 'stealth_write', signature: 'stealth_write(pid, address, data)', description: 'Stealthy memory write' },
    { name: 'hook_detect', signature: 'hook_detect(pid, address)', description: 'Detect function hooks' },
    { name: 'hook_restore', signature: 'hook_restore(pid, address)', description: 'Restore original function bytes' },
    { name: 'debugger_evasion', signature: 'debugger_evasion()', description: 'Apply anti-debugger techniques' },
    { name: 'signature_obfuscate', signature: 'signature_obfuscate(code)', description: 'Obfuscate code signature' },
    { name: 'unity_find_objects', signature: 'unity_find_objects(class_name)', description: 'Find Unity game objects' },
    { name: 'unity_get_component', signature: 'unity_get_component(object_addr, component_name)', description: 'Get Unity component' },
    { name: 'unity_call_method', signature: 'unity_call_method(object_addr, method_name)', description: 'Call Unity method' },
    { name: 'unity_mono_dump', signature: 'unity_mono_dump(pid)', description: 'Dump Mono assemblies' },
    { name: 'unreal_find_actors', signature: 'unreal_find_actors(class_name)', description: 'Find Unreal actors' },
    { name: 'unreal_get_property', signature: 'unreal_get_property(actor_addr, property_name)', description: 'Get Unreal property value' },
    { name: 'unreal_set_property', signature: 'unreal_set_property(actor_addr, property_name, value)', description: 'Set Unreal property value' },
    { name: 'unreal_process_event', signature: 'unreal_process_event(actor_addr, event_name)', description: 'Trigger Unreal event' },
    { name: 'vtable_hook', signature: 'vtable_hook(pid, object_addr, vfunc_index)', description: 'Hook vtable function' },
    { name: 'vtable_restore', signature: 'vtable_restore(pid, object_addr)', description: 'Restore original vtable' },
    { name: 'script_engine_hook', signature: 'script_engine_hook(pid, engine_type)', description: 'Hook script engine (Lua/Python)' },
    { name: 'game_packet_capture', signature: 'game_packet_capture(port)', description: 'Capture game network packets' },
    { name: 'game_packet_inject', signature: 'game_packet_inject(port, packet_data)', description: 'Inject custom game packet' },
    { name: 'game_packet_decrypt', signature: 'game_packet_decrypt(encrypted_data)', description: 'Decrypt game packets' },
    { name: 'game_packet_encrypt', signature: 'game_packet_encrypt(plain_data)', description: 'Encrypt game packets' },
    { name: 'protocol_reverse', signature: 'protocol_reverse(packet_samples)', description: 'Reverse engineer protocol' },
    { name: 'game_server_emulate', signature: 'game_server_emulate(port)', description: 'Emulate game server' },
    { name: 'network_proxy', signature: 'network_proxy(listen_port, target_port)', description: 'Network MITM proxy' },
    { name: 'lag_exploit', signature: 'lag_exploit(delay_ms, packet_count)', description: 'Induce artificial lag' },
    { name: 'dx_hook', signature: 'dx_hook(pid)', description: 'Hook DirectX rendering' },
    { name: 'opengl_hook', signature: 'opengl_hook(pid)', description: 'Hook OpenGL rendering' },
    { name: 'vulkan_hook', signature: 'vulkan_hook(pid)', description: 'Hook Vulkan rendering' },
    { name: 'render_overlay', signature: 'render_overlay(pid, elements)', description: 'Render visual overlay' },
    { name: 'shader_inject', signature: 'shader_inject(pid, shader_code)', description: 'Inject custom shader' },
    { name: 'audio_hook', signature: 'audio_hook(pid)', description: 'Hook audio API' },
    { name: 'esp_create', signature: 'esp_create(pid, entity_list_addr)', description: 'Create ESP (wallhack)' },
    { name: 'entity_iterate', signature: 'entity_iterate(pid, entity_list_addr)', description: 'Iterate game entities' },
    { name: 'aimbot_calculate', signature: 'aimbot_calculate(camera_pos, target_pos)', description: 'Calculate aim angles' },
    { name: 'triggerbot', signature: 'triggerbot(pid, crosshair_entity_addr)', description: 'Auto-fire triggerbot' },
    { name: 'visibility_check', signature: 'visibility_check(pid, entity_addr)', description: 'Check entity visibility' },
    { name: 'trainer_create', signature: 'trainer_create(pid, cheats_map)', description: 'Create game trainer' },
    { name: 'world_to_screen', signature: 'world_to_screen(world_pos, view_matrix)', description: 'Convert 3D to 2D coords' },
    { name: 'crash_dump_analyze', signature: 'crash_dump_analyze(dump_path)', description: 'Analyze crash dumps' },
    { name: 'auto_re_pattern', signature: 'auto_re_pattern(pid, function_name)', description: 'Auto-find function patterns' },
    { name: 'data_flow_trace', signature: 'data_flow_trace(pid, variable_addr)', description: 'Trace data flow' },
    { name: 'dll_inject', signature: 'dll_inject(pid, dll_path)', description: 'Inject DLL into process' },
    { name: 'dll_hide', signature: 'dll_hide(pid, dll_name)', description: 'Hide DLL from detection' },
    { name: 'reflective_load', signature: 'reflective_load(dll_bytes)', description: 'Reflective DLL loading' },
    { name: 'persist_install', signature: 'persist_install(method, target_path)', description: 'Install persistence' },
    { name: 'persist_remove', signature: 'persist_remove(method)', description: 'Remove persistence' },
    { name: 'cyclic', signature: 'cyclic(length)', description: 'Generate cyclic pattern' },
    { name: 'cyclic_find', signature: 'cyclic_find(pattern)', description: 'Find offset in cyclic pattern' },
    { name: 'p64', signature: 'p64(value)', description: 'Pack 64-bit little-endian' },
    { name: 'p32', signature: 'p32(value)', description: 'Pack 32-bit little-endian' },
    { name: 'shellcode', signature: 'shellcode(arch, payload, ...)', description: 'Generate shellcode' },
    { name: 'rop_find', signature: 'rop_find(binary, gadget)', description: 'Find ROP gadgets' },
    { name: 'connect', signature: 'connect(host, port)', description: 'Connect to remote service' },
    { name: 'sha256', signature: 'sha256(data)', description: 'SHA-256 hash' },
    { name: 'base64_encode', signature: 'base64_encode(data)', description: 'Base64 encode' },
];

connection.onCompletion((_textDocumentPosition) => {
    return builtinFunctions.map(func => ({
        label: func.name,
        kind: CompletionItemKind.Function,
        data: func.name,
        detail: func.signature,
        documentation: func.description
    }));
});

connection.onCompletionResolve((item: CompletionItem): CompletionItem => {
    const func = builtinFunctions.find(f => f.name === item.data);
    if (func) {
        item.detail = func.signature;
        item.documentation = func.description;
    }
    return item;
});

connection.onHover((_params) => {
    return {
        contents: 'TALON built-in function'
    } as Hover;
});

documents.listen(connection);
connection.listen();
