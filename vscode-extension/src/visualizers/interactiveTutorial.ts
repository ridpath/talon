import * as vscode from 'vscode';

interface TutorialStep {
    title: string;
    description: string;
    code: string;
    explanation: string;
    task?: string;
    solution?: string;
}

interface Tutorial {
    id: string;
    title: string;
    difficulty: 'beginner' | 'intermediate' | 'advanced';
    category: string;
    steps: TutorialStep[];
}

export class InteractiveTutorial {
    private panel: vscode.WebviewPanel | undefined;
    private currentTutorial: Tutorial | undefined;
    private currentStep: number = 0;
    private tutorials: Tutorial[] = [];

    constructor() {
        this.initializeTutorials();
    }

    private initializeTutorials() {
        this.tutorials = [
            {
                id: 'buffer-overflow-basics',
                title: 'Buffer Overflow Fundamentals',
                difficulty: 'beginner',
                category: 'Binary Exploitation',
                steps: [
                    {
                        title: 'Understanding Buffer Overflows',
                        description: 'A buffer overflow occurs when you write more data to a buffer than it can hold, overwriting adjacent memory.',
                        code: 'let pattern = cyclic(100)\nprint("Generated pattern:", pattern)',
                        explanation: 'The cyclic() function generates a unique pattern that helps identify the exact offset at which we overwrite the return address.'
                    },
                    {
                        title: 'Finding the Offset',
                        description: 'When the program crashes, we can use the crash address to find exactly where we control execution.',
                        code: 'let offset = cyclic_find("aaab")\nprint("Offset found at:", offset)',
                        explanation: 'cyclic_find() tells us the exact position in our pattern where we gained control.',
                        task: 'Find the offset for the pattern "caaa"',
                        solution: 'let offset = cyclic_find("caaa")\nprint("Offset:", offset)'
                    },
                    {
                        title: 'Building the Payload',
                        description: 'Now we can craft a precise payload that overwrites the return address with our target.',
                        code: 'let offset = 264\nlet payload = cyclic(offset)\npayload = payload + p64(0xdeadbeef)\nprint("Payload ready:", len(payload), "bytes")',
                        explanation: 'We fill up to the offset, then add our target address. The return address is now 0xdeadbeef!',
                        task: 'Create a payload that jumps to 0x401234',
                        solution: 'let offset = 264\nlet payload = cyclic(offset) + p64(0x401234)'
                    },
                    {
                        title: 'Delivering the Exploit',
                        description: 'Finally, we send our payload to the vulnerable service.',
                        code: 'let conn = connect("target.com", 1337)\nsend(conn, payload)\ninteractive(conn)',
                        explanation: 'We connect to the target, send the payload, and get an interactive shell!',
                        task: 'Complete the exploit for localhost:4444',
                        solution: 'let conn = connect("localhost", 4444)\nsend(conn, payload)\ninteractive(conn)'
                    }
                ]
            },
            {
                id: 'rop-chain-construction',
                title: 'ROP Chain Construction',
                difficulty: 'intermediate',
                category: 'Binary Exploitation',
                steps: [
                    {
                        title: 'Why ROP Chains?',
                        description: 'When NX/DEP is enabled, we cannot execute shellcode on the stack. Instead, we chain together existing code snippets (gadgets).',
                        code: 'let gadgets = rop_find("./vuln", "pop rdi")\nprint("Found gadgets:", gadgets)',
                        explanation: 'We search the binary for useful instruction sequences ending in "ret".'
                    },
                    {
                        title: 'Setting Up Arguments',
                        description: 'To call system("/bin/sh"), we need to put the string pointer in RDI (first argument register).',
                        code: 'let pop_rdi = 0x401234  // pop rdi; ret\nlet bin_sh = 0x404000   // address of "/bin/sh"\nlet system = 0x7ffff7e12345  // system() address\n\nlet rop = p64(pop_rdi) + p64(bin_sh) + p64(system)',
                        explanation: 'This ROP chain pops /bin/sh into RDI, then calls system().',
                        task: 'Build a ROP chain to call system() with your own addresses',
                        solution: 'let pop_rdi = GADGET_ADDR\nlet bin_sh = STRING_ADDR\nlet system = SYSTEM_ADDR\nlet rop = p64(pop_rdi) + p64(bin_sh) + p64(system)'
                    },
                    {
                        title: 'Complete ROP Exploit',
                        description: 'Combine everything: overflow + ROP chain.',
                        code: 'let offset = 264\nlet payload = cyclic(offset) + rop\nsend(conn, payload)',
                        explanation: 'We overflow to the return address, then our ROP chain executes!',
                        task: 'Complete a full ROP exploit',
                        solution: 'let offset = 264\nlet payload = cyclic(offset)\npayload = payload + p64(pop_rdi) + p64(bin_sh) + p64(system)\nsend(conn, payload)\ninteractive(conn)'
                    }
                ]
            },
            {
                id: 'unity-game-hacking',
                title: 'Unity Game Hacking',
                difficulty: 'beginner',
                category: 'Game Hacking',
                steps: [
                    {
                        title: 'Attaching to Unity Games',
                        description: 'Unity games use the Mono runtime. TALON has native support for finding Unity objects.',
                        code: 'let proc = process_attach("Game.exe")\nlet pid = proc["pid"]\nprint("Attached to:", proc["name"])',
                        explanation: 'First, we attach to the game process to access its memory.'
                    },
                    {
                        title: 'Finding GameObjects',
                        description: 'Unity organizes game entities as GameObjects. We can search by class name.',
                        code: 'let players = unity_find_objects("PlayerController")\nprint("Found", len(players), "players")\nprint("First player at:", hex(players[0]["address"]))',
                        explanation: 'This searches the Mono heap for all instances of PlayerController.',
                        task: 'Find all "Enemy" objects in the game',
                        solution: 'let enemies = unity_find_objects("Enemy")\nfor enemy in enemies\n    print(hex(enemy["address"]))\nend'
                    },
                    {
                        title: 'Reading Components',
                        description: 'GameObjects have components that store data like health, position, etc.',
                        code: 'let player_addr = players[0]["address"]\nlet health_comp = unity_get_component(player_addr, "HealthComponent")\nprint("Health component:", health_comp)',
                        explanation: 'Components contain the actual gameplay data we want to modify.'
                    },
                    {
                        title: 'Modifying Values',
                        description: 'Now we can write to memory to change values like health, ammo, etc.',
                        code: 'let health_addr = health_comp["address"] + 0x10\nmem_write(pid, health_addr, p32(9999))\nprint("Health set to 9999!")',
                        explanation: 'We write directly to the health field offset in the component.',
                        task: 'Set player ammo to 999 (ammo is at offset 0x20)',
                        solution: 'let ammo_addr = health_comp["address"] + 0x20\nmem_write(pid, ammo_addr, p32(999))'
                    }
                ]
            },
            {
                id: 'anti-cheat-bypass',
                title: 'Anti-Cheat Evasion',
                difficulty: 'advanced',
                category: 'Game Hacking',
                steps: [
                    {
                        title: 'Detecting Anti-Cheat',
                        description: 'Before doing anything suspicious, we need to know what anti-cheat is running.',
                        code: 'let acs = anticheat_detect()\nfor ac in acs\n    print("Detected:", ac["name"])\nend',
                        explanation: 'This checks for EasyAntiCheat, BattlEye, VAC, and other common systems.'
                    },
                    {
                        title: 'Applying Evasions',
                        description: 'TALON can apply various anti-debugging and anti-detection techniques.',
                        code: 'if len(acs) > 0\n    let evasions = debugger_evasion()\n    for e in evasions\n        print("Applied:", e)\n    end\nend',
                        explanation: 'This applies techniques like hiding debugger presence and unhooking functions.'
                    },
                    {
                        title: 'Stealth Memory Operations',
                        description: 'Use stealth functions to avoid hooks placed by anti-cheat.',
                        code: 'let data = stealth_read(pid, addr, 64)\nstealth_write(pid, addr, payload)',
                        explanation: 'These bypass usermode hooks by reading/writing through kernel drivers.',
                        task: 'Read 256 bytes from 0x10000000 using stealth',
                        solution: 'let data = stealth_read(pid, 0x10000000, 256)\nprint("Read", len(data), "bytes")'
                    }
                ]
            },
            {
                id: 'esp-development',
                title: 'ESP (Wallhack) Development',
                difficulty: 'intermediate',
                category: 'Game Hacking',
                steps: [
                    {
                        title: 'Understanding ESP',
                        description: 'ESP draws boxes around enemies through walls by iterating entities and projecting their 3D positions to 2D screen space.',
                        code: 'esp_create(pid, entity_list_addr)\nprint("ESP overlay created")',
                        explanation: 'This creates a transparent overlay window where we can draw.'
                    },
                    {
                        title: 'Iterating Entities',
                        description: 'We need to find all player entities in the game.',
                        code: 'let entities = entity_iterate(pid, entity_list_addr)\nfor entity in entities\n    print("Entity at:", hex(entity["address"]))\nend',
                        explanation: 'This walks the entity linked list or array in game memory.',
                        task: 'Count total entities and print the number',
                        solution: 'let entities = entity_iterate(pid, entity_list_addr)\nprint("Total entities:", len(entities))'
                    },
                    {
                        title: 'World to Screen',
                        description: 'Convert 3D world positions to 2D screen coordinates.',
                        code: 'let world_pos = [entity["x"], entity["y"], entity["z"]]\nlet screen = world_to_screen(world_pos, view_matrix)\nprint("Screen pos:", screen["x"], screen["y"])',
                        explanation: 'This uses the view matrix to project coordinates, just like the game does.'
                    }
                ]
            }
        ];
    }

    public show(context: vscode.ExtensionContext) {
        if (this.panel) {
            this.panel.reveal(vscode.ViewColumn.Two);
            return;
        }

        this.panel = vscode.window.createWebviewPanel(
            'talonTutorial',
            'TALON Interactive Tutorial',
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
                    case 'selectTutorial':
                        this.selectTutorial(message.id);
                        break;
                    case 'nextStep':
                        this.nextStep();
                        break;
                    case 'prevStep':
                        this.previousStep();
                        break;
                    case 'insertCode':
                        this.insertCode(message.code);
                        break;
                    case 'checkSolution':
                        this.checkSolution(message.code);
                        break;
                }
            },
            undefined,
            context.subscriptions
        );

        this.panel.onDidDispose(() => {
            this.panel = undefined;
        });

        this.sendTutorialList();
    }

    private selectTutorial(id: string) {
        this.currentTutorial = this.tutorials.find(t => t.id === id);
        this.currentStep = 0;
        this.sendCurrentStep();
    }

    private nextStep() {
        if (this.currentTutorial && this.currentStep < this.currentTutorial.steps.length - 1) {
            this.currentStep++;
            this.sendCurrentStep();
        }
    }

    private previousStep() {
        if (this.currentStep > 0) {
            this.currentStep--;
            this.sendCurrentStep();
        }
    }

    private sendTutorialList() {
        if (this.panel) {
            this.panel.webview.postMessage({
                command: 'tutorialList',
                tutorials: this.tutorials.map(t => ({
                    id: t.id,
                    title: t.title,
                    difficulty: t.difficulty,
                    category: t.category,
                    stepCount: t.steps.length
                }))
            });
        }
    }

    private sendCurrentStep() {
        if (this.panel && this.currentTutorial) {
            const step = this.currentTutorial.steps[this.currentStep];
            this.panel.webview.postMessage({
                command: 'updateStep',
                tutorial: {
                    title: this.currentTutorial.title,
                    difficulty: this.currentTutorial.difficulty
                },
                step: {
                    index: this.currentStep,
                    total: this.currentTutorial.steps.length,
                    ...step
                }
            });
        }
    }

    private insertCode(code: string) {
        const editor = vscode.window.activeTextEditor;
        if (editor && editor.document.languageId === 'talon') {
            const position = editor.selection.active;
            editor.edit(editBuilder => {
                editBuilder.insert(position, code + '\n');
            });
        }
    }

    private checkSolution(userCode: string) {
        if (this.currentTutorial && this.panel) {
            const step = this.currentTutorial.steps[this.currentStep];
            if (step.solution) {
                const correct = userCode.trim() === step.solution.trim();
                this.panel.webview.postMessage({
                    command: 'solutionResult',
                    correct: correct,
                    message: correct ? 'Correct! Great job!' : 'Not quite. Try again or view the solution.'
                });
            }
        }
    }

    private getWebviewContent(): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Interactive Tutorial</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: #1e1e1e;
            color: #d4d4d4;
            padding: 20px;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
        }
        .header h1 {
            font-size: 24px;
            font-weight: 600;
            margin-bottom: 5px;
        }
        .tutorial-list {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 15px;
            margin-bottom: 20px;
        }
        .tutorial-card {
            background: #252526;
            padding: 20px;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.2s;
            border-left: 4px solid #569cd6;
        }
        .tutorial-card:hover {
            background: #2d2d30;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(86,156,214,0.3);
        }
        .tutorial-card h3 {
            font-size: 16px;
            margin-bottom: 8px;
            color: #4fc3f7;
        }
        .tutorial-meta {
            display: flex;
            gap: 10px;
            margin-top: 10px;
            font-size: 12px;
        }
        .badge {
            padding: 4px 8px;
            border-radius: 4px;
            font-weight: 600;
        }
        .badge.beginner { background: #4caf50; color: white; }
        .badge.intermediate { background: #ff9800; color: white; }
        .badge.advanced { background: #f44336; color: white; }
        .step-container {
            display: none;
            background: #252526;
            border-radius: 8px;
            padding: 25px;
        }
        .step-container.active {
            display: block;
        }
        .step-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 20px;
            padding-bottom: 15px;
            border-bottom: 2px solid #3e3e42;
        }
        .step-title h2 {
            font-size: 20px;
            color: #4fc3f7;
            margin-bottom: 5px;
        }
        .step-progress {
            font-size: 14px;
            color: #9cdcfe;
        }
        .step-content {
            margin-bottom: 25px;
        }
        .step-content h3 {
            font-size: 16px;
            color: #569cd6;
            margin-bottom: 10px;
        }
        .step-content p {
            line-height: 1.6;
            margin-bottom: 15px;
        }
        .code-demo {
            background: #1e1e1e;
            border: 1px solid #3e3e42;
            border-radius: 6px;
            padding: 15px;
            margin: 15px 0;
            font-family: 'Consolas', monospace;
            font-size: 13px;
        }
        .code-demo pre {
            margin: 0;
            color: #d4d4d4;
        }
        .explanation {
            background: rgba(86,156,214,0.1);
            border-left: 3px solid #569cd6;
            padding: 12px;
            margin: 15px 0;
            border-radius: 4px;
        }
        .task-box {
            background: rgba(255,152,0,0.1);
            border-left: 3px solid #ff9800;
            padding: 15px;
            margin: 20px 0;
            border-radius: 4px;
        }
        .task-box h4 {
            color: #ff9800;
            margin-bottom: 10px;
        }
        .solution-box {
            background: rgba(76,175,80,0.1);
            border-left: 3px solid #4caf50;
            padding: 15px;
            margin: 15px 0;
            border-radius: 4px;
            display: none;
        }
        .solution-box.show {
            display: block;
        }
        .action-buttons {
            display: flex;
            gap: 10px;
            margin-top: 20px;
        }
        .btn {
            padding: 10px 20px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 600;
            font-size: 14px;
            transition: all 0.2s;
        }
        .btn-primary {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .btn-primary:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102,126,234,0.4);
        }
        .btn-secondary {
            background: #333;
            color: #d4d4d4;
            border: 1px solid #555;
        }
        .btn-secondary:hover {
            background: #404040;
        }
        .btn:disabled {
            opacity: 0.5;
            cursor: not-allowed;
            transform: none;
        }
        .feedback {
            padding: 12px;
            border-radius: 6px;
            margin: 15px 0;
            font-weight: 600;
            display: none;
        }
        .feedback.success {
            background: #4caf50;
            color: white;
            display: block;
        }
        .feedback.error {
            background: #f44336;
            color: white;
            display: block;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>Interactive TALON Tutorials</h1>
        <p>Step-by-step guided learning with hands-on practice</p>
    </div>

    <div id="tutorialList" class="tutorial-list"></div>
    <div id="stepContainer" class="step-container"></div>

    <script>
        const vscode = acquireVsCodeApi();
        let currentStepData = null;

        window.addEventListener('message', event => {
            const message = event.data;
            switch (message.command) {
                case 'tutorialList':
                    displayTutorialList(message.tutorials);
                    break;
                case 'updateStep':
                    displayStep(message.tutorial, message.step);
                    break;
                case 'solutionResult':
                    showFeedback(message.correct, message.message);
                    break;
            }
        });

        function displayTutorialList(tutorials) {
            const container = document.getElementById('tutorialList');
            let html = '';
            tutorials.forEach(tut => {
                html += \`
                    <div class="tutorial-card" onclick="selectTutorial('\${tut.id}')">
                        <h3>\${tut.title}</h3>
                        <p>\${tut.stepCount} steps</p>
                        <div class="tutorial-meta">
                            <span class="badge \${tut.difficulty}">\${tut.difficulty}</span>
                            <span>\${tut.category}</span>
                        </div>
                    </div>
                \`;
            });
            container.innerHTML = html;
        }

        function selectTutorial(id) {
            document.getElementById('tutorialList').style.display = 'none';
            document.getElementById('stepContainer').classList.add('active');
            vscode.postMessage({ command: 'selectTutorial', id: id });
        }

        function displayStep(tutorial, step) {
            currentStepData = step;
            const container = document.getElementById('stepContainer');
            
            let html = \`
                <div class="step-header">
                    <div class="step-title">
                        <h2>\${tutorial.title}</h2>
                        <div class="step-progress">Step \${step.index + 1} of \${step.total}</div>
                    </div>
                </div>

                <div class="step-content">
                    <h3>\${step.title}</h3>
                    <p>\${step.description}</p>

                    <div class="code-demo">
                        <pre>\${escapeHtml(step.code)}</pre>
                    </div>

                    <div class="explanation">
                        <strong>Explanation:</strong> \${step.explanation}
                    </div>

                    \${step.task ? \`
                        <div class="task-box">
                            <h4>Your Task:</h4>
                            <p>\${step.task}</p>
                        </div>
                        <div class="solution-box" id="solutionBox">
                            <h4>Solution:</h4>
                            <div class="code-demo">
                                <pre>\${escapeHtml(step.solution || '')}</pre>
                            </div>
                        </div>
                        <div class="feedback" id="feedback"></div>
                    \` : ''}

                    <div class="action-buttons">
                        <button class="btn btn-primary" onclick="insertCode()">Insert Code</button>
                        \${step.task ? '<button class="btn btn-secondary" onclick="showSolution()">Show Solution</button>' : ''}
                        <button class="btn btn-secondary" onclick="prevStep()" \${step.index === 0 ? 'disabled' : ''}>Previous</button>
                        <button class="btn btn-primary" onclick="nextStep()" \${step.index === step.total - 1 ? 'disabled' : ''}>Next Step</button>
                    </div>
                </div>
            \`;

            container.innerHTML = html;
        }

        function insertCode() {
            if (currentStepData) {
                vscode.postMessage({
                    command: 'insertCode',
                    code: currentStepData.code
                });
            }
        }

        function showSolution() {
            document.getElementById('solutionBox').classList.add('show');
        }

        function nextStep() {
            vscode.postMessage({ command: 'nextStep' });
        }

        function prevStep() {
            vscode.postMessage({ command: 'prevStep' });
        }

        function showFeedback(correct, message) {
            const feedback = document.getElementById('feedback');
            feedback.className = 'feedback ' + (correct ? 'success' : 'error');
            feedback.textContent = message;
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
