import * as vscode from 'vscode';

interface RopGadget {
    address: string;
    instruction: string;
    hasNullBytes: boolean;
    category: string;
}

export class RopChainBuilder {
    private panel: vscode.WebviewPanel | undefined;
    private gadgets: RopGadget[] = [];
    private chain: RopGadget[] = [];

    public show(context: vscode.ExtensionContext) {
        if (this.panel) {
            this.panel.reveal();
        } else {
            this.panel = vscode.window.createWebviewPanel(
                'talonRopBuilder',
                'TALON ROP Chain Builder',
                vscode.ViewColumn.Two,
                {
                    enableScripts: true,
                    retainContextWhenHidden: true
                }
            );

            this.panel.webview.html = this.getHtmlContent();

            this.panel.onDidDispose(() => {
                this.panel = undefined;
            });
            
            this.panel.webview.onDidReceiveMessage(
                message => {
                    switch (message.command) {
                        case 'scanGadgets':
                            this.scanBinaryForGadgets(message.binary);
                            break;
                        case 'generateCode':
                            this.generateTalonCode();
                            break;
                        case 'addToChain':
                            this.chain.push(message.gadget);
                            break;
                    }
                },
                undefined,
                context.subscriptions
            );
        }
    }
    
    private async scanBinaryForGadgets(binary: string) {
        const { exec } = require('child_process');
        exec(`talon rop scan "${binary}"`, (error: any, stdout: string) => {
            if (!error && this.panel) {
                const gadgets = this.parseGadgets(stdout);
                this.gadgets = gadgets;
                this.panel.webview.postMessage({
                    command: 'gadgetsLoaded',
                    gadgets: gadgets
                });
            }
        });
    }
    
    private parseGadgets(output: string): RopGadget[] {
        const lines = output.split('\n');
        const gadgets: RopGadget[] = [];
        
        for (const line of lines) {
            const match = line.match(/(0x[0-9a-fA-F]+):\s+(.+)/);
            if (match) {
                gadgets.push({
                    address: match[1],
                    instruction: match[2],
                    hasNullBytes: match[1].includes('00'),
                    category: this.categorizeGadget(match[2])
                });
            }
        }
        
        return gadgets;
    }
    
    private categorizeGadget(instruction: string): string {
        if (instruction.includes('pop')) return 'control';
        if (instruction.includes('mov')) return 'data';
        if (instruction.includes('ret')) return 'return';
        if (instruction.includes('syscall') || instruction.includes('int')) return 'syscall';
        return 'other';
    }
    
    private async generateTalonCode() {
        const editor = vscode.window.activeTextEditor;
        if (!editor) return;
        
        let code = 'let rop_chain = [\n';
        for (const gadget of this.chain) {
            code += `    ${gadget.address},  // ${gadget.instruction}\n`;
        }
        code += ']\n';
        
        const snippet = new vscode.SnippetString(code);
        editor.insertSnippet(snippet);
        vscode.window.showInformationMessage('ROP chain code generated!');
    }

    private getHtmlContent(): string {
        return `<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: Arial, sans-serif;
            background: #1e1e1e;
            color: #d4d4d4;
            padding: 20px;
        }
        .gadget-list {
            background: #252525;
            padding: 15px;
            margin: 10px 0;
            border-left: 3px solid #007acc;
        }
        .gadget {
            padding: 8px;
            margin: 5px 0;
            background: #2d2d30;
            cursor: grab;
            border: 1px solid #3e3e42;
        }
        .gadget:hover {
            background: #3e3e42;
        }
        .chain-container {
            background: #252525;
            padding: 15px;
            min-height: 200px;
            margin: 10px 0;
            border: 2px dashed #007acc;
        }
        .chain-gadget {
            background: #0e639c;
            padding: 10px;
            margin: 5px 0;
            border-radius: 4px;
        }
        button {
            background: #0e639c;
            color: white;
            border: none;
            padding: 10px 20px;
            cursor: pointer;
            margin: 5px;
        }
        button:hover {
            background: #1177bb;
        }
        .address {
            color: #4ec9b0;
            font-family: monospace;
        }
        .instruction {
            color: #ce9178;
            font-family: monospace;
        }
    </style>
</head>
<body>
    <h2>ROP Chain Builder</h2>
    
    <div>
        <h3>Available Gadgets</h3>
        <div class="gadget-list">
            <div class="gadget" draggable="true">
                <span class="address">0x401234:</span>
                <span class="instruction">pop rdi ; ret</span>
            </div>
            <div class="gadget" draggable="true">
                <span class="address">0x401567:</span>
                <span class="instruction">pop rsi ; ret</span>
            </div>
            <div class="gadget" draggable="true">
                <span class="address">0x401890:</span>
                <span class="instruction">pop rdx ; ret</span>
            </div>
            <div class="gadget" draggable="true">
                <span class="address">0x401abc:</span>
                <span class="instruction">syscall ; ret</span>
            </div>
            <div class="gadget" draggable="true">
                <span class="address">0x401def:</span>
                <span class="instruction">xor rax, rax ; ret</span>
            </div>
        </div>
    </div>

    <div style="margin-top: 20px;">
        <h3>ROP Chain</h3>
        <div class="chain-container" id="rop-chain">
            <p style="color: #888;">Drag gadgets here to build your ROP chain</p>
        </div>
    </div>

    <div style="margin-top: 20px;">
        <button onclick="generateCode()">Generate TALON Code</button>
        <button onclick="clearChain()">Clear Chain</button>
        <button onclick="validateChain()">Validate Chain</button>
    </div>

    <div id="output" style="margin-top: 20px; background: #252525; padding: 15px; font-family: monospace;"></div>

    <script>
        const vscode = acquireVsCodeApi();
        let chain = [];

        document.querySelectorAll('.gadget').forEach(gadget => {
            gadget.addEventListener('dragstart', (e) => {
                e.dataTransfer.setData('text/plain', gadget.textContent);
            });
        });

        document.getElementById('rop-chain').addEventListener('dragover', (e) => {
            e.preventDefault();
        });

        document.getElementById('rop-chain').addEventListener('drop', (e) => {
            e.preventDefault();
            const data = e.dataTransfer.getData('text/plain');
            const chainDiv = document.getElementById('rop-chain');
            const newGadget = document.createElement('div');
            newGadget.className = 'chain-gadget';
            newGadget.textContent = data;
            chainDiv.appendChild(newGadget);
            chain.push(data);
        });

        function generateCode() {
            let code = 'let rop_chain = [\\n';
            chain.forEach(gadget => {
                const match = gadget.match(/0x([0-9a-f]+):/);
                if (match) {
                    code += \`    \${match[0].replace(':', '')},\\n\`;
                }
            });
            code += ']\\n';
            document.getElementById('output').textContent = code;
            vscode.postMessage({ command: 'insertCode', code: code });
        }

        function clearChain() {
            chain = [];
            document.getElementById('rop-chain').innerHTML = '<p style="color: #888;">Drag gadgets here to build your ROP chain</p>';
            document.getElementById('output').textContent = '';
        }

        function validateChain() {
            if (chain.length === 0) {
                alert('Chain is empty!');
            } else {
                alert(\`Chain has \${chain.length} gadgets. Validation passed!\`);
            }
        }
    </script>
</body>
</html>`;
    }
}
