import * as vscode from 'vscode';

export class DebuggerPanel {
    private panel: vscode.WebviewPanel | undefined;

    public show(context: vscode.ExtensionContext) {
        if (this.panel) {
            this.panel.reveal();
        } else {
            this.panel = vscode.window.createWebviewPanel(
                'talonDebugger',
                'TALON Live Debugger',
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
        }
    }

    private getHtmlContent(): string {
        return `<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: monospace;
            background: #1e1e1e;
            color: #d4d4d4;
            padding: 20px;
        }
        .debugger-section {
            background: #252525;
            padding: 15px;
            margin: 10px 0;
            border-left: 3px solid #007acc;
        }
        .register {
            display: inline-block;
            width: 150px;
            padding: 5px;
            margin: 3px;
            background: #2d2d30;
            border: 1px solid #3e3e42;
        }
        .register-name {
            color: #569cd6;
            font-weight: bold;
        }
        .register-value {
            color: #b5cea8;
        }
        .disasm-line {
            padding: 5px;
            margin: 2px 0;
        }
        .disasm-line:hover {
            background: #2d2d30;
        }
        .disasm-address {
            color: #4ec9b0;
        }
        .disasm-instruction {
            color: #ce9178;
            margin-left: 20px;
        }
        button {
            background: #0e639c;
            color: white;
            border: none;
            padding: 8px 16px;
            cursor: pointer;
            margin: 3px;
        }
        button:hover {
            background: #1177bb;
        }
        .current-line {
            background: #264f78;
            border-left: 3px solid #f48771;
        }
    </style>
</head>
<body>
    <h2>Live Debugger Integration</h2>
    
    <div class="debugger-section">
        <h3>Debugger Controls</h3>
        <button onclick="stepInstruction()">Step</button>
        <button onclick="continueExecution()">Continue</button>
        <button onclick="stepOver()">Step Over</button>
        <button onclick="stepOut()">Step Out</button>
        <button onclick="restart()">Restart</button>
    </div>

    <div class="debugger-section">
        <h3>Registers</h3>
        <div id="registers">
            <div class="register">
                <span class="register-name">RAX:</span>
                <span class="register-value">0x0000000000000000</span>
            </div>
            <div class="register">
                <span class="register-name">RBX:</span>
                <span class="register-value">0x00007fffffffe800</span>
            </div>
            <div class="register">
                <span class="register-name">RCX:</span>
                <span class="register-value">0x0000000000401234</span>
            </div>
            <div class="register">
                <span class="register-name">RDX:</span>
                <span class="register-value">0x0000000000000010</span>
            </div>
            <div class="register">
                <span class="register-name">RSI:</span>
                <span class="register-value">0x00007fffffffe7f0</span>
            </div>
            <div class="register">
                <span class="register-name">RDI:</span>
                <span class="register-value">0x0000000000000001</span>
            </div>
            <div class="register">
                <span class="register-name">RBP:</span>
                <span class="register-value">0x00007fffffffe800</span>
            </div>
            <div class="register">
                <span class="register-name">RSP:</span>
                <span class="register-value">0x00007fffffffe7e8</span>
            </div>
            <div class="register">
                <span class="register-name">RIP:</span>
                <span class="register-value">0x0000000000401234</span>
            </div>
        </div>
    </div>

    <div class="debugger-section">
        <h3>Disassembly</h3>
        <div id="disassembly">
            <div class="disasm-line">
                <span class="disasm-address">0x401230:</span>
                <span class="disasm-instruction">push rbp</span>
            </div>
            <div class="disasm-line current-line">
                <span class="disasm-address">0x401234:</span>
                <span class="disasm-instruction">mov rax, 0x1</span>
            </div>
            <div class="disasm-line">
                <span class="disasm-address">0x40123b:</span>
                <span class="disasm-instruction">call 0x401500</span>
            </div>
            <div class="disasm-line">
                <span class="disasm-address">0x401240:</span>
                <span class="disasm-instruction">test rax, rax</span>
            </div>
            <div class="disasm-line">
                <span class="disasm-address">0x401243:</span>
                <span class="disasm-instruction">je 0x401260</span>
            </div>
        </div>
    </div>

    <script>
        const vscode = acquireVsCodeApi();

        function stepInstruction() {
            vscode.postMessage({ command: 'debugStep' });
        }

        function continueExecution() {
            vscode.postMessage({ command: 'debugContinue' });
        }

        function stepOver() {
            vscode.postMessage({ command: 'debugStepOver' });
        }

        function stepOut() {
            vscode.postMessage({ command: 'debugStepOut' });
        }

        function restart() {
            vscode.postMessage({ command: 'debugRestart' });
        }
    </script>
</body>
</html>`;
    }
}
