import * as vscode from 'vscode';

export class MemoryVisualizer {
    private panel: vscode.WebviewPanel | undefined;
    private currentMemory: Map<string, Uint8Array> = new Map();

    public show(context: vscode.ExtensionContext) {
        if (this.panel) {
            this.panel.reveal();
        } else {
            this.panel = vscode.window.createWebviewPanel(
                'talonMemoryVisualizer',
                'TALON Memory Visualizer',
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
                        case 'readMemory':
                            this.readMemory(message.address, message.size);
                            break;
                        case 'writeMemory':
                            this.writeMemory(message.address, message.data);
                            break;
                    }
                },
                undefined,
                context.subscriptions
            );
        }
    }
    
    public showMemoryMappings(context: vscode.ExtensionContext) {
        this.show(context);
        if (this.panel) {
            this.panel.webview.postMessage({
                command: 'showTab',
                tab: 'mappings'
            });
        }
    }
    
    public searchMemory(context: vscode.ExtensionContext, searchTerm: string) {
        this.show(context);
        if (this.panel) {
            this.panel.webview.postMessage({
                command: 'search',
                term: searchTerm
            });
        }
    }
    
    public telescope(context: vscode.ExtensionContext, address: string) {
        this.show(context);
        if (this.panel) {
            this.panel.webview.postMessage({
                command: 'telescope',
                address: address
            });
        }
    }
    
    private async readMemory(address: string, size: number) {
        const { exec } = require('child_process');
        exec(`talon debug read-memory ${address} ${size}`, (error: any, stdout: string) => {
            if (!error && this.panel) {
                this.panel.webview.postMessage({
                    command: 'memoryData',
                    address: address,
                    data: stdout
                });
            }
        });
    }
    
    private async writeMemory(address: string, data: string) {
        const { exec } = require('child_process');
        exec(`talon debug write-memory ${address} ${data}`, (error: any, stdout: string) => {
            if (!error) {
                vscode.window.showInformationMessage(`Memory written to ${address}`);
            }
        });
    }

    private getHtmlContent(): string {
        return `<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: 'Courier New', monospace;
            background: #1e1e1e;
            color: #d4d4d4;
            padding: 0;
            margin: 0;
        }
        .tabs {
            display: flex;
            background: #252525;
            border-bottom: 2px solid #007acc;
        }
        .tab {
            padding: 10px 20px;
            cursor: pointer;
            border-right: 1px solid #3e3e42;
        }
        .tab:hover {
            background: #2d2d30;
        }
        .tab.active {
            background: #007acc;
            color: #fff;
        }
        .tab-content {
            display: none;
            padding: 20px;
        }
        .tab-content.active {
            display: block;
        }
        .memory-line {
            margin: 2px 0;
            padding: 4px 8px;
            font-size: 13px;
        }
        .memory-line:hover {
            background: #2d2d30;
        }
        .memory-address {
            color: #4ec9b0;
            font-weight: bold;
            display: inline-block;
            width: 120px;
        }
        .memory-bytes {
            color: #ce9178;
            margin: 0 20px;
            display: inline-block;
            width: 280px;
        }
        .memory-ascii {
            color: #608b4e;
            display: inline-block;
        }
        .byte-cyclic {
            color: #ff6b6b;
            font-weight: bold;
        }
        .byte-rop {
            color: #4ecdc4;
            font-weight: bold;
        }
        .byte-shellcode {
            color: #ffe66d;
            font-weight: bold;
        }
        h2 {
            color: #569cd6;
            margin: 0 0 15px 0;
        }
        .search-box {
            margin-bottom: 15px;
        }
        .search-input {
            background: #3c3c3c;
            color: #d4d4d4;
            border: 1px solid #007acc;
            padding: 8px;
            width: 300px;
            font-family: monospace;
        }
        .search-button {
            background: #007acc;
            color: #fff;
            border: none;
            padding: 8px 16px;
            cursor: pointer;
            margin-left: 10px;
        }
        .search-button:hover {
            background: #005a9e;
        }
        .telescope-line {
            margin: 4px 0;
            padding: 6px;
            background: #252525;
            border-left: 3px solid #007acc;
        }
        .pointer-arrow {
            color: #007acc;
            margin: 0 8px;
        }
        .annotation {
            color: #858585;
            font-style: italic;
            margin-left: 20px;
        }
    </style>
</head>
<body>
    <div class="tabs">
        <div class="tab active" onclick="showTab('stack')">Stack</div>
        <div class="tab" onclick="showTab('heap')">Heap</div>
        <div class="tab" onclick="showTab('mappings')">Mappings</div>
        <div class="tab" onclick="showTab('search')">Search</div>
        <div class="tab" onclick="showTab('telescope')">Telescope</div>
    </div>
    
    <div id="stack" class="tab-content active">
        <h2>Stack Memory @ 0x7fffffffe000</h2>
        <div id="stack-view">
            <div class="memory-line">
                <span class="memory-address">0x7fffffffe000:</span>
                <span class="memory-bytes">41 41 41 41 41 41 41 41</span>
                <span class="memory-ascii">AAAAAAAA</span>
                <span class="annotation">buffer overflow</span>
            </div>
            <div class="memory-line">
                <span class="memory-address">0x7fffffffe008:</span>
                <span class="memory-bytes">42 42 42 42 42 42 42 42</span>
                <span class="memory-ascii">BBBBBBBB</span>
            </div>
            <div class="memory-line">
                <span class="memory-address">0x7fffffffe010:</span>
                <span class="memory-bytes byte-rop">ef be ad de 00 00 00 00</span>
                <span class="memory-ascii">........</span>
                <span class="annotation">p64(0xdeadbeef)</span>
            </div>
            <div class="memory-line">
                <span class="memory-address">0x7fffffffe018:</span>
                <span class="memory-bytes byte-rop">00 10 40 00 00 00 00 00</span>
                <span class="memory-ascii">..@.....</span>
                <span class="annotation">return address</span>
            </div>
        </div>
    </div>
    
    <div id="heap" class="tab-content">
        <h2>Heap Memory</h2>
        <div id="heap-view">
            <p>Heap chunks will appear here when attached to a process</p>
        </div>
    </div>
    
    <div id="mappings" class="tab-content">
        <h2>Memory Mappings (vmmap)</h2>
        <div id="mappings-view" style="font-size: 12px;">
            <div class="memory-line">0x400000-0x401000   r-x   text segment</div>
            <div class="memory-line">0x600000-0x601000   rw-   data segment</div>
            <div class="memory-line">0x1234000-0x1235000 rw-   heap</div>
            <div class="memory-line">0x7ffff7a0d000-0x7ffff7bcd000 r-x   libc-2.27.so</div>
            <div class="memory-line">0x7ffffffde000-0x7ffffffff000 rw-   stack</div>
        </div>
    </div>
    
    <div id="search" class="tab-content">
        <h2>Memory Search</h2>
        <div class="search-box">
            <input type="text" id="searchInput" class="search-input" placeholder="Search for /bin/sh or 0x41414141">
            <button class="search-button" onclick="performSearch()">Search</button>
        </div>
        <div id="search-results"></div>
    </div>
    
    <div id="telescope" class="tab-content">
        <h2>Telescope - Follow Pointer Chain</h2>
        <div class="search-box">
            <input type="text" id="telescopeAddress" class="search-input" placeholder="0x7fffffffe000" value="0x7fffffffe000">
            <button class="search-button" onclick="startTelescope()">Telescope</button>
        </div>
        <div id="telescope-view">
            <div class="telescope-line">
                0x7fffffffe000 <span class="pointer-arrow">--></span> 0x400800 <span class="annotation">main+0</span>
            </div>
            <div class="telescope-line">
                0x7fffffffe008 <span class="pointer-arrow">--></span> 0x7ffff7a2d830 <span class="annotation">__libc_start_main+240</span>
            </div>
        </div>
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        
        function showTab(tabName) {
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));
            event.target.classList.add('active');
            document.getElementById(tabName).classList.add('active');
            
            vscode.postMessage({ command: 'tabChanged', tab: tabName });
        }
        
        function performSearch() {
            const term = document.getElementById('searchInput').value;
            vscode.postMessage({ command: 'search', term: term });
        }
        
        function startTelescope() {
            const address = document.getElementById('telescopeAddress').value;
            vscode.postMessage({ command: 'telescope', address: address });
        }
        
        window.addEventListener('message', event => {
            const message = event.data;
            switch(message.command) {
                case 'memoryData':
                    updateMemoryView(message.address, message.data);
                    break;
                case 'searchResults':
                    updateSearchResults(message.results);
                    break;
                case 'telescopeData':
                    updateTelescopeView(message.data);
                    break;
            }
        });
        
        function updateMemoryView(address, data) {
            console.log('Memory updated:', address);
        }
        
        function updateSearchResults(results) {
            const resultsDiv = document.getElementById('search-results');
            resultsDiv.innerHTML = '<h3>Found ' + results.length + ' matches</h3>';
            results.forEach(r => {
                resultsDiv.innerHTML += '<div class="memory-line">' + r + '</div>';
            });
        }
        
        function updateTelescopeView(data) {
            document.getElementById('telescope-view').innerHTML = data;
        }
    </script>
</body>
</html>`;
    }
}
