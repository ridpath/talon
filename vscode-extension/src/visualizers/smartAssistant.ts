import * as vscode from 'vscode';

interface Message {
    role: 'user' | 'assistant';
    content: string;
    code?: string;
}

export class SmartAssistant {
    private panel: vscode.WebviewPanel | undefined;
    private conversation: Message[] = [];
    private knowledgeBase: Map<string, string> = new Map();

    constructor() {
        this.initializeKnowledgeBase();
    }

    private initializeKnowledgeBase() {
        this.knowledgeBase.set('buffer overflow', 
            'let offset = 264  // Find with cyclic pattern\nlet payload = cyclic(offset)\npayload = payload + p64(RIP_ADDRESS)');
        
        this.knowledgeBase.set('rop chain', 
            'let gadgets = rop_find("./binary", "pop rdi")\nlet rop = p64(gadgets[0]) + p64(ARG) + p64(SYSTEM_ADDR)');
        
        this.knowledgeBase.set('unity hack', 
            'let proc = process_attach("Game.exe")\nlet pid = proc["pid"]\nlet players = unity_find_objects("Player")\nlet health = unity_get_component(players[0]["address"], "Health")\nmem_write(pid, health["address"] + 0x10, p32(9999))');
        
        this.knowledgeBase.set('process attach', 
            'let proc = process_attach("TARGET")\nlet pid = proc["pid"]\nlet modules = process_modules(pid)\nlet base = modules[0]["base"]');
        
        this.knowledgeBase.set('memory leak', 
            'let leaked = u64(mem_read(pid, LEAK_ADDR, 8))\nlet libc_base = leaked - KNOWN_OFFSET\nprint("Libc base:", hex(libc_base))');
        
        this.knowledgeBase.set('shellcode', 
            'let sc = shellcode("x64", "execve", "/bin/sh")\nlet nop_sled = bytes("\\x90") * 100\nlet payload = nop_sled + sc');
        
        this.knowledgeBase.set('anti-cheat bypass', 
            'let acs = anticheat_detect()\nif len(acs) > 0\n    let evasions = debugger_evasion()\n    let data = stealth_read(pid, addr, size)\nend');
        
        this.knowledgeBase.set('esp aimbot', 
            'esp_create(pid, entity_list_addr)\nlet entities = entity_iterate(pid, entity_list_addr)\nlet angles = aimbot_calculate(camera_pos, target_pos)');
    }

    public show(context: vscode.ExtensionContext) {
        if (this.panel) {
            this.panel.reveal(vscode.ViewColumn.Two);
            return;
        }

        this.panel = vscode.window.createWebviewPanel(
            'talonSmartAssistant',
            'TALON Smart Assistant',
            vscode.ViewColumn.Two,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [context.extensionUri]
            }
        );

        this.panel.webview.html = this.getWebviewContent();

        this.panel.webview.onDidReceiveMessage(
            message => {
                switch (message.command) {
                    case 'sendMessage':
                        this.processMessage(message.text);
                        break;
                    case 'insertCode':
                        this.insertCode(message.code);
                        break;
                    case 'clearChat':
                        this.conversation = [];
                        this.updateChat();
                        break;
                }
            },
            undefined,
            context.subscriptions
        );

        this.panel.onDidDispose(() => {
            this.panel = undefined;
        });
    }

    private async processMessage(userMessage: string) {
        this.conversation.push({ role: 'user', content: userMessage });
        this.updateChat();

        const response = await this.generateResponse(userMessage);
        this.conversation.push(response);
        this.updateChat();
    }

    private async generateResponse(userMessage: string): Promise<Message> {
        const messageLower = userMessage.toLowerCase();
        
        const contexts = [
            { keywords: ['buffer overflow', 'bof', 'overflow', 'cyclic'], key: 'buffer overflow' },
            { keywords: ['rop', 'rop chain', 'gadget', 'return oriented'], key: 'rop chain' },
            { keywords: ['unity', 'game object', 'component'], key: 'unity hack' },
            { keywords: ['process', 'attach', 'pid'], key: 'process attach' },
            { keywords: ['leak', 'libc', 'aslr'], key: 'memory leak' },
            { keywords: ['shellcode', 'payload', 'shell'], key: 'shellcode' },
            { keywords: ['anti-cheat', 'anticheat', 'eac', 'battleye'], key: 'anti-cheat bypass' },
            { keywords: ['esp', 'aimbot', 'wallhack', 'aim'], key: 'esp aimbot' }
        ];

        let matchedContext: string | undefined;
        let matchScore = 0;

        for (const context of contexts) {
            const score = context.keywords.filter(kw => messageLower.includes(kw)).length;
            if (score > matchScore) {
                matchScore = score;
                matchedContext = context.key;
            }
        }

        if (matchedContext && this.knowledgeBase.has(matchedContext)) {
            const code = this.knowledgeBase.get(matchedContext)!;
            let explanation = this.getExplanation(matchedContext, userMessage);
            
            return {
                role: 'assistant',
                content: explanation,
                code: code
            };
        }

        return {
            role: 'assistant',
            content: this.getGeneralResponse(userMessage)
        };
    }

    private getExplanation(context: string, userMessage: string): string {
        const explanations: { [key: string]: string } = {
            'buffer overflow': `I'll help you build a buffer overflow exploit. Here's a complete TALON script:

1. Generate a cyclic pattern to find the offset
2. Create payload with the offset
3. Overwrite the return address

This code finds the exact offset, creates the overflow payload, and overwrites RIP/EIP with your desired address.`,

            'rop chain': `I'll help you build a ROP chain to bypass NX/DEP. Here's how:

1. Find ROP gadgets in the binary
2. Chain gadgets to set up function arguments
3. Call your target function (e.g., system)

This code searches for gadgets, builds the chain, and calls the function with your arguments.`,

            'unity hack': `I'll help you hack Unity games. Here's a complete workflow:

1. Attach to the game process
2. Find Unity GameObjects by class name
3. Get components and modify values

This code finds player objects, gets the health component, and sets health to maximum.`,

            'process attach': `I'll help you attach to a process and enumerate modules:

1. Attach by name or PID
2. List all loaded modules
3. Get base addresses for further exploitation

This gives you all the information needed to start memory manipulation.`,

            'memory leak': `I'll help you leak memory addresses to bypass ASLR:

1. Read 8 bytes from a known location
2. Unpack as 64-bit address
3. Calculate base address using known offset

This is crucial for bypassing ASLR protection.`,

            'shellcode': `I'll help you generate and deliver shellcode:

1. Generate shellcode for your target architecture
2. Add NOP sled for reliability
3. Combine into final payload

This creates a reliable shellcode delivery mechanism.`,

            'anti-cheat bypass': `I'll help you bypass anti-cheat systems:

1. Detect which anti-cheat is running
2. Apply appropriate evasion techniques
3. Use stealth memory operations

This code detects anti-cheat and applies countermeasures.`,

            'esp aimbot': `I'll help you create ESP and aimbot for games:

1. Create ESP overlay
2. Iterate through entities
3. Calculate aim angles to target

This gives you full ESP and aimbot functionality.`
        };

        return explanations[context] || 'Here\'s the code you need:';
    }

    private getGeneralResponse(userMessage: string): string {
        const messageLower = userMessage.toLowerCase();

        if (messageLower.includes('help') || messageLower.includes('what can you do')) {
            return `I'm your TALON exploit development assistant! I can help you with:

- Buffer Overflow exploits (cyclic patterns, offset finding)
- ROP chain construction (gadget finding, chain building)
- Unity/Unreal game hacking (object finding, component manipulation)
- Process manipulation (attach, memory read/write, scanning)
- Shellcode generation (various architectures)
- Anti-cheat bypass techniques
- ESP and aimbot development
- Memory leaking and ASLR bypass

Just describe what you want to do, and I'll generate the TALON code for you!

Examples:
- "Build a buffer overflow exploit with ROP chain"
- "Hack Unity game health values"
- "Create ESP for FPS game"`;
        }

        if (messageLower.includes('example') || messageLower.includes('show me')) {
            return `Here are some example requests:

**Binary Exploitation:**
- "I have a buffer overflow at offset 264"
- "Build ROP chain to call system"
- "Leak libc address"

**Game Hacking:**
- "Hack Unity player health"
- "Create ESP for CS:GO"
- "Bypass EasyAntiCheat"

**Process Manipulation:**
- "Attach to game.exe and scan memory"
- "Write value to specific address"
- "Find all instances of a pattern"

Try asking me one of these!`;
        }

        if (messageLower.includes('nx') || messageLower.includes('dep') || messageLower.includes('aslr') || messageLower.includes('pie')) {
            return `I can help you bypass binary protections:

**NX/DEP:** Use ROP chains instead of shellcode
**ASLR:** Leak addresses to calculate base addresses
**PIE:** Leak any code pointer to find base
**Stack Canary:** Leak canary value before overflow

Ask me specifically about the protection you need to bypass!`;
        }

        return `I'm not sure exactly what you need. Could you be more specific?

Try asking:
- "How do I build a buffer overflow exploit?"
- "Help me hack a Unity game"
- "Create an aimbot for FPS games"
- "Bypass anti-cheat systems"

Or type "help" to see all my capabilities!`;
    }

    private insertCode(code: string) {
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document.languageId === 'talon') {
            const position = editor.selection.active;
            editor.edit(editBuilder => {
                editBuilder.insert(position, code + '\n');
            });
            vscode.window.showInformationMessage('Code inserted into editor');
        } else {
            vscode.window.showWarningMessage('Please open a .talon file first');
        }
    }

    private updateChat() {
        if (this.panel) {
            this.panel.webview.postMessage({
                command: 'updateChat',
                messages: this.conversation
            });
        }
    }

    private getWebviewContent(): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TALON Smart Assistant</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: #1e1e1e;
            color: #d4d4d4;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 20px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.3);
        }
        .header h1 {
            font-size: 20px;
            font-weight: 600;
            margin-bottom: 5px;
        }
        .header p {
            opacity: 0.9;
            font-size: 13px;
        }
        .chat-container {
            flex: 1;
            overflow-y: auto;
            padding: 20px;
            scroll-behavior: smooth;
        }
        .message {
            margin-bottom: 20px;
            animation: fadeIn 0.3s ease-in;
        }
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }
        .message.user .message-content {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            margin-left: auto;
            margin-right: 0;
        }
        .message.assistant .message-content {
            background: #252526;
            border-left: 3px solid #569cd6;
        }
        .message-content {
            max-width: 80%;
            padding: 15px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.2);
        }
        .message-header {
            font-size: 12px;
            font-weight: 600;
            margin-bottom: 8px;
            opacity: 0.8;
        }
        .message-text {
            line-height: 1.6;
            white-space: pre-wrap;
            font-size: 14px;
        }
        .code-block {
            background: #1e1e1e;
            border: 1px solid #3e3e42;
            border-radius: 6px;
            padding: 15px;
            margin-top: 12px;
            font-family: 'Consolas', monospace;
            font-size: 13px;
            position: relative;
        }
        .code-block pre {
            margin: 0;
            color: #d4d4d4;
            overflow-x: auto;
        }
        .code-actions {
            margin-top: 10px;
            display: flex;
            gap: 10px;
        }
        .code-btn {
            background: #569cd6;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            font-weight: 600;
            transition: all 0.2s;
        }
        .code-btn:hover {
            background: #4fc3f7;
            transform: translateY(-1px);
        }
        .input-container {
            background: #252526;
            padding: 20px;
            border-top: 1px solid #3e3e42;
            box-shadow: 0 -2px 8px rgba(0,0,0,0.2);
        }
        .input-wrapper {
            display: flex;
            gap: 10px;
        }
        #messageInput {
            flex: 1;
            background: #333;
            border: 1px solid #555;
            color: #d4d4d4;
            padding: 12px;
            border-radius: 6px;
            font-size: 14px;
            font-family: 'Segoe UI', sans-serif;
        }
        #messageInput:focus {
            outline: none;
            border-color: #569cd6;
            box-shadow: 0 0 0 2px rgba(86,156,214,0.2);
        }
        .send-btn {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 600;
            transition: all 0.2s;
        }
        .send-btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102,126,234,0.4);
        }
        .send-btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }
        .quick-actions {
            display: flex;
            gap: 10px;
            margin-bottom: 15px;
            flex-wrap: wrap;
        }
        .quick-action {
            background: #333;
            border: 1px solid #555;
            color: #d4d4d4;
            padding: 8px 12px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            transition: all 0.2s;
        }
        .quick-action:hover {
            background: #404040;
            border-color: #569cd6;
        }
        .empty-state {
            text-align: center;
            padding: 60px 20px;
            opacity: 0.6;
        }
        .empty-state h2 {
            font-size: 18px;
            margin-bottom: 10px;
        }
        .empty-state p {
            font-size: 14px;
            line-height: 1.6;
        }
        .suggestions {
            margin-top: 20px;
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 10px;
        }
        .suggestion-card {
            background: #252526;
            padding: 15px;
            border-radius: 6px;
            border-left: 3px solid #569cd6;
            cursor: pointer;
            transition: all 0.2s;
        }
        .suggestion-card:hover {
            background: #2d2d30;
            transform: translateX(4px);
        }
        .suggestion-card h3 {
            font-size: 13px;
            margin-bottom: 5px;
            color: #569cd6;
        }
        .suggestion-card p {
            font-size: 12px;
            opacity: 0.8;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>TALON Smart Assistant</h1>
        <p>Context-aware exploit development help</p>
    </div>

    <div class="chat-container" id="chatContainer">
        <div class="empty-state">
            <h2>How can I help you today?</h2>
            <p>I can generate complete TALON exploits based on your description.</p>
            
            <div class="suggestions">
                <div class="suggestion-card" onclick="sendQuickMessage('Build a buffer overflow exploit with NX enabled')">
                    <h3>Buffer Overflow</h3>
                    <p>Generate cyclic pattern and ROP chain</p>
                </div>
                <div class="suggestion-card" onclick="sendQuickMessage('Hack Unity game player health')">
                    <h3>Unity Game Hack</h3>
                    <p>Find and modify game objects</p>
                </div>
                <div class="suggestion-card" onclick="sendQuickMessage('Create ESP for FPS game')">
                    <h3>ESP/Aimbot</h3>
                    <p>Build wallhack and aimbot</p>
                </div>
                <div class="suggestion-card" onclick="sendQuickMessage('Bypass EasyAntiCheat')">
                    <h3>Anti-Cheat Bypass</h3>
                    <p>Evade detection systems</p>
                </div>
            </div>
        </div>
    </div>

    <div class="input-container">
        <div class="quick-actions">
            <button class="quick-action" onclick="sendQuickMessage('help')">Help</button>
            <button class="quick-action" onclick="sendQuickMessage('show examples')">Examples</button>
            <button class="quick-action" onclick="clearChat()">Clear Chat</button>
        </div>
        <div class="input-wrapper">
            <input type="text" id="messageInput" placeholder="Describe your exploit scenario..." onkeypress="handleKeyPress(event)">
            <button class="send-btn" onclick="sendMessage()">Send</button>
        </div>
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        let messages = [];

        function sendMessage() {
            const input = document.getElementById('messageInput');
            const text = input.value.trim();
            if (!text) return;

            vscode.postMessage({
                command: 'sendMessage',
                text: text
            });

            input.value = '';
            input.focus();
        }

        function sendQuickMessage(text) {
            document.getElementById('messageInput').value = text;
            sendMessage();
        }

        function handleKeyPress(event) {
            if (event.key === 'Enter') {
                sendMessage();
            }
        }

        function insertCode(code) {
            vscode.postMessage({
                command: 'insertCode',
                code: code
            });
        }

        function clearChat() {
            if (confirm('Clear chat history?')) {
                vscode.postMessage({
                    command: 'clearChat'
                });
            }
        }

        window.addEventListener('message', event => {
            const message = event.data;
            switch (message.command) {
                case 'updateChat':
                    updateChatDisplay(message.messages);
                    break;
            }
        });

        function updateChatDisplay(msgs) {
            messages = msgs;
            const container = document.getElementById('chatContainer');
            
            if (messages.length === 0) {
                container.innerHTML = '<div class="empty-state"><h2>How can I help you today?</h2></div>';
                return;
            }

            let html = '';
            messages.forEach(msg => {
                html += \`
                    <div class="message \${msg.role}">
                        <div class="message-content">
                            <div class="message-header">\${msg.role === 'user' ? 'You' : 'Assistant'}</div>
                            <div class="message-text">\${escapeHtml(msg.content)}</div>
                            \${msg.code ? \`
                                <div class="code-block">
                                    <pre>\${escapeHtml(msg.code)}</pre>
                                    <div class="code-actions">
                                        <button class="code-btn" onclick='insertCode(\\\`\${msg.code.replace(/\`/g, '\\\\`')}\\\`)'>Insert into Editor</button>
                                    </div>
                                </div>
                            \` : ''}
                        </div>
                    </div>
                \`;
            });

            container.innerHTML = html;
            container.scrollTop = container.scrollHeight;
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }
    </script>
</body>
</html>`;
    }
}
