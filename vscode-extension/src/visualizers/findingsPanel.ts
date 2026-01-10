import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

interface Finding {
    id: string;
    title: string;
    description: string;
    category: 'vulnerability' | 'gadget' | 'offset' | 'address' | 'note';
    severity?: 'low' | 'medium' | 'high' | 'critical';
    file?: string;
    line?: number;
    address?: string;
    code?: string;
    tags: string[];
    timestamp: number;
}

export class FindingsPanel {
    private panel: vscode.WebviewPanel | undefined;
    private findings: Finding[] = [];
    private context: vscode.ExtensionContext | undefined;

    public show(context: vscode.ExtensionContext) {
        this.context = context;
        this.loadFindings();

        if (this.panel) {
            this.panel.reveal(vscode.ViewColumn.Two);
            return;
        }

        this.panel = vscode.window.createWebviewPanel(
            'talonFindings',
            'TALON Research Findings',
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
                    case 'addFinding':
                        this.addFinding(message.finding);
                        break;
                    case 'deleteFinding':
                        this.deleteFinding(message.id);
                        break;
                    case 'exportFindings':
                        this.exportFindings();
                        break;
                    case 'goToLocation':
                        this.goToLocation(message.file, message.line);
                        break;
                    case 'insertCode':
                        this.insertCode(message.code);
                        break;
                }
            },
            undefined,
            context.subscriptions
        );

        this.panel.onDidDispose(() => {
            this.panel = undefined;
        });

        this.updatePanel();
    }

    public async addFindingQuick() {
        const editor = vscode.window.activeTextEditor;
        
        const title = await vscode.window.showInputBox({
            prompt: 'Finding title',
            placeHolder: 'Buffer overflow at offset 264'
        });

        if (!title) return;

        const category = await vscode.window.showQuickPick([
            { label: 'Vulnerability', value: 'vulnerability' },
            { label: 'ROP Gadget', value: 'gadget' },
            { label: 'Offset', value: 'offset' },
            { label: 'Address', value: 'address' },
            { label: 'Note', value: 'note' }
        ], { placeHolder: 'Select category' });

        if (!category) return;

        const description = await vscode.window.showInputBox({
            prompt: 'Description (optional)',
            placeHolder: 'Vulnerable strcpy allows control of RIP'
        });

        let severity: 'low' | 'medium' | 'high' | 'critical' | undefined;
        if (category.value === 'vulnerability') {
            const sev = await vscode.window.showQuickPick([
                { label: 'Low', value: 'low' },
                { label: 'Medium', value: 'medium' },
                { label: 'High', value: 'high' },
                { label: 'Critical', value: 'critical' }
            ], { placeHolder: 'Select severity' });
            severity = sev?.value as any;
        }

        const finding: Finding = {
            id: Date.now().toString(),
            title,
            description: description || '',
            category: category.value as any,
            severity,
            file: editor?.document.fileName,
            line: editor ? editor.selection.active.line + 1 : undefined,
            code: editor ? editor.document.getText(editor.selection) : undefined,
            tags: [],
            timestamp: Date.now()
        };

        this.addFinding(finding);
        vscode.window.showInformationMessage('Finding added!');
    }

    private addFinding(finding: Finding) {
        this.findings.push(finding);
        this.saveFindings();
        this.updatePanel();
    }

    private deleteFinding(id: string) {
        this.findings = this.findings.filter(f => f.id !== id);
        this.saveFindings();
        this.updatePanel();
    }

    private async exportFindings() {
        const format = await vscode.window.showQuickPick([
            { label: 'Markdown Report', value: 'md' },
            { label: 'JSON', value: 'json' },
            { label: 'HTML Report', value: 'html' }
        ], { placeHolder: 'Select export format' });

        if (!format) return;

        let content = '';
        let extension = format.value;

        if (format.value === 'md') {
            content = this.generateMarkdownReport();
        } else if (format.value === 'json') {
            content = JSON.stringify(this.findings, null, 2);
        } else if (format.value === 'html') {
            content = this.generateHtmlReport();
        }

        const uri = await vscode.window.showSaveDialog({
            filters: {
                'Report': [extension]
            },
            defaultUri: vscode.Uri.file(`talon-findings-${Date.now()}.${extension}`)
        });

        if (uri) {
            fs.writeFileSync(uri.fsPath, content);
            vscode.window.showInformationMessage('Findings exported!');
        }
    }

    private generateMarkdownReport(): string {
        let md = `# TALON Exploit Research Findings\n\n`;
        md += `**Generated:** ${new Date().toLocaleString()}\n`;
        md += `**Total Findings:** ${this.findings.length}\n\n`;

        const categories = ['vulnerability', 'gadget', 'offset', 'address', 'note'];
        
        for (const cat of categories) {
            const catFindings = this.findings.filter(f => f.category === cat);
            if (catFindings.length === 0) continue;

            md += `## ${cat.charAt(0).toUpperCase() + cat.slice(1)}s\n\n`;

            for (const finding of catFindings) {
                md += `### ${finding.title}\n\n`;
                if (finding.severity) md += `**Severity:** ${finding.severity}\n\n`;
                if (finding.description) md += `${finding.description}\n\n`;
                if (finding.address) md += `**Address:** ${finding.address}\n\n`;
                if (finding.file) md += `**Location:** ${finding.file}:${finding.line}\n\n`;
                if (finding.code) md += `\`\`\`talon\n${finding.code}\n\`\`\`\n\n`;
                md += `---\n\n`;
            }
        }

        return md;
    }

    private generateHtmlReport(): string {
        let html = `<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>TALON Findings Report</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; }
        h1 { color: #333; border-bottom: 3px solid #667eea; padding-bottom: 10px; }
        .finding { background: #f5f5f5; padding: 15px; margin: 15px 0; border-left: 4px solid #569cd6; }
        .vulnerability { border-left-color: #f44336; }
        .gadget { border-left-color: #4caf50; }
        .severity { display: inline-block; padding: 4px 12px; border-radius: 4px; color: white; font-weight: bold; }
        .critical { background: #f44336; }
        .high { background: #ff9800; }
        .medium { background: #ff9800; }
        .low { background: #2196f3; }
        code { background: #e0e0e0; padding: 2px 6px; border-radius: 3px; }
        pre { background: #2d2d30; color: #d4d4d4; padding: 15px; border-radius: 5px; overflow-x: auto; }
    </style>
</head>
<body>
    <h1>TALON Exploit Research Findings</h1>
    <p><strong>Generated:</strong> ${new Date().toLocaleString()}</p>
    <p><strong>Total Findings:</strong> ${this.findings.length}</p>
`;

        for (const finding of this.findings) {
            html += `<div class="finding ${finding.category}">`;
            html += `<h3>${this.escapeHtml(finding.title)}</h3>`;
            if (finding.severity) {
                html += `<span class="severity ${finding.severity}">${finding.severity.toUpperCase()}</span>`;
            }
            if (finding.description) {
                html += `<p>${this.escapeHtml(finding.description)}</p>`;
            }
            if (finding.address) {
                html += `<p><strong>Address:</strong> <code>${finding.address}</code></p>`;
            }
            if (finding.file) {
                html += `<p><strong>Location:</strong> ${this.escapeHtml(finding.file)}:${finding.line}</p>`;
            }
            if (finding.code) {
                html += `<pre>${this.escapeHtml(finding.code)}</pre>`;
            }
            html += `</div>`;
        }

        html += `</body></html>`;
        return html;
    }

    private escapeHtml(text: string): string {
        return text
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#039;');
    }

    private goToLocation(file: string, line: number) {
        if (file && line) {
            vscode.workspace.openTextDocument(file).then(doc => {
                vscode.window.showTextDocument(doc).then(editor => {
                    const position = new vscode.Position(line - 1, 0);
                    editor.selection = new vscode.Selection(position, position);
                    editor.revealRange(new vscode.Range(position, position));
                });
            });
        }
    }

    private insertCode(code: string) {
        const editor = vscode.window.activeTextEditor;
        if (editor) {
            const position = editor.selection.active;
            editor.edit(editBuilder => {
                editBuilder.insert(position, code + '\n');
            });
        }
    }

    private loadFindings() {
        if (this.context) {
            const findingsPath = path.join(this.context.globalStorageUri.fsPath, 'findings.json');
            if (fs.existsSync(findingsPath)) {
                try {
                    const data = fs.readFileSync(findingsPath, 'utf8');
                    this.findings = JSON.parse(data);
                } catch (e) {
                    this.findings = [];
                }
            }
        }
    }

    private saveFindings() {
        if (this.context) {
            const storagePath = this.context.globalStorageUri.fsPath;
            if (!fs.existsSync(storagePath)) {
                fs.mkdirSync(storagePath, { recursive: true });
            }
            const findingsPath = path.join(storagePath, 'findings.json');
            fs.writeFileSync(findingsPath, JSON.stringify(this.findings, null, 2));
        }
    }

    private updatePanel() {
        if (this.panel) {
            this.panel.webview.postMessage({
                command: 'updateFindings',
                findings: this.findings
            });
        }
    }

    private getWebviewContent(): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Research Findings</title>
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
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .header h1 {
            font-size: 20px;
            font-weight: 600;
        }
        .stats {
            display: flex;
            gap: 15px;
            margin-bottom: 20px;
        }
        .stat-card {
            background: #252526;
            padding: 15px;
            border-radius: 6px;
            flex: 1;
            border-left: 3px solid #569cd6;
        }
        .stat-card h3 {
            font-size: 24px;
            color: #4fc3f7;
            margin-bottom: 5px;
        }
        .stat-card p {
            font-size: 12px;
            opacity: 0.8;
        }
        .finding-item {
            background: #252526;
            padding: 15px;
            margin-bottom: 15px;
            border-radius: 6px;
            border-left: 4px solid #569cd6;
            transition: all 0.2s;
        }
        .finding-item:hover {
            background: #2d2d30;
            transform: translateX(4px);
        }
        .finding-item.vulnerability { border-left-color: #f44336; }
        .finding-item.gadget { border-left-color: #4caf50; }
        .finding-item.offset { border-left-color: #ff9800; }
        .finding-item.address { border-left-color: #2196f3; }
        .finding-item.note { border-left-color: #9c27b0; }
        .finding-header {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 10px;
        }
        .finding-title {
            font-size: 16px;
            font-weight: 600;
            color: #4fc3f7;
            margin-bottom: 5px;
        }
        .finding-meta {
            display: flex;
            gap: 8px;
            margin-bottom: 10px;
        }
        .badge {
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 11px;
            font-weight: 600;
        }
        .badge.critical { background: #f44336; color: white; }
        .badge.high { background: #ff5722; color: white; }
        .badge.medium { background: #ff9800; color: white; }
        .badge.low { background: #2196f3; color: white; }
        .badge.category { background: #333; color: #d4d4d4; }
        .finding-description {
            margin-bottom: 10px;
            line-height: 1.5;
        }
        .finding-code {
            background: #1e1e1e;
            padding: 10px;
            border-radius: 4px;
            font-family: 'Consolas', monospace;
            font-size: 12px;
            margin: 10px 0;
            border: 1px solid #3e3e42;
        }
        .finding-location {
            font-size: 12px;
            color: #9cdcfe;
            cursor: pointer;
            margin-top: 8px;
        }
        .finding-location:hover {
            text-decoration: underline;
        }
        .finding-actions {
            display: flex;
            gap: 8px;
            margin-top: 10px;
        }
        .btn {
            padding: 6px 12px;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
            font-weight: 600;
            transition: all 0.2s;
        }
        .btn-delete {
            background: #f44336;
            color: white;
        }
        .btn-insert {
            background: #4caf50;
            color: white;
        }
        .btn-export {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 10px 20px;
            font-size: 14px;
        }
        .empty-state {
            text-align: center;
            padding: 60px 20px;
            opacity: 0.6;
        }
        .filter-bar {
            background: #252526;
            padding: 15px;
            border-radius: 6px;
            margin-bottom: 20px;
            display: flex;
            gap: 10px;
        }
        .filter-bar select {
            background: #333;
            border: 1px solid #555;
            color: #d4d4d4;
            padding: 8px;
            border-radius: 4px;
            flex: 1;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>Research Findings & Notes</h1>
        <button class="btn btn-export" onclick="exportFindings()">Export Report</button>
    </div>

    <div class="stats">
        <div class="stat-card">
            <h3 id="totalCount">0</h3>
            <p>Total Findings</p>
        </div>
        <div class="stat-card">
            <h3 id="vulnCount">0</h3>
            <p>Vulnerabilities</p>
        </div>
        <div class="stat-card">
            <h3 id="gadgetCount">0</h3>
            <p>ROP Gadgets</p>
        </div>
    </div>

    <div class="filter-bar">
        <select id="categoryFilter" onchange="filterFindings()">
            <option value="all">All Categories</option>
            <option value="vulnerability">Vulnerabilities</option>
            <option value="gadget">Gadgets</option>
            <option value="offset">Offsets</option>
            <option value="address">Addresses</option>
            <option value="note">Notes</option>
        </select>
        <select id="severityFilter" onchange="filterFindings()">
            <option value="all">All Severities</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
        </select>
    </div>

    <div id="findingsList"></div>

    <script>
        const vscode = acquireVsCodeApi();
        let findings = [];

        window.addEventListener('message', event => {
            const message = event.data;
            switch (message.command) {
                case 'updateFindings':
                    findings = message.findings;
                    displayFindings();
                    break;
            }
        });

        function displayFindings() {
            const container = document.getElementById('findingsList');
            
            document.getElementById('totalCount').textContent = findings.length;
            document.getElementById('vulnCount').textContent = findings.filter(f => f.category === 'vulnerability').length;
            document.getElementById('gadgetCount').textContent = findings.filter(f => f.category === 'gadget').length;

            if (findings.length === 0) {
                container.innerHTML = '<div class="empty-state"><h2>No findings yet</h2><p>Use "TALON: Add Finding" to record your discoveries</p></div>';
                return;
            }

            let html = '';
            findings.forEach(finding => {
                html += \`
                    <div class="finding-item \${finding.category}">
                        <div class="finding-header">
                            <div>
                                <div class="finding-title">\${escapeHtml(finding.title)}</div>
                                <div class="finding-meta">
                                    <span class="badge category">\${finding.category}</span>
                                    \${finding.severity ? \`<span class="badge \${finding.severity}">\${finding.severity}</span>\` : ''}
                                </div>
                            </div>
                        </div>
                        \${finding.description ? \`<div class="finding-description">\${escapeHtml(finding.description)}</div>\` : ''}
                        \${finding.address ? \`<div><strong>Address:</strong> <code>\${finding.address}</code></div>\` : ''}
                        \${finding.code ? \`<div class="finding-code">\${escapeHtml(finding.code)}</div>\` : ''}
                        \${finding.file ? \`<div class="finding-location" onclick='goToLocation("\${finding.file}", \${finding.line})">\${finding.file}:\${finding.line}</div>\` : ''}
                        <div class="finding-actions">
                            \${finding.code ? \`<button class="btn btn-insert" onclick='insertCode(\\\`\${finding.code.replace(/\`/g, '\\\\`')}\\\`)'>Insert Code</button>\` : ''}
                            <button class="btn btn-delete" onclick="deleteFinding('\${finding.id}')">Delete</button>
                        </div>
                    </div>
                \`;
            });

            container.innerHTML = html;
        }

        function deleteFinding(id) {
            if (confirm('Delete this finding?')) {
                vscode.postMessage({ command: 'deleteFinding', id: id });
            }
        }

        function exportFindings() {
            vscode.postMessage({ command: 'exportFindings' });
        }

        function goToLocation(file, line) {
            vscode.postMessage({ command: 'goToLocation', file: file, line: line });
        }

        function insertCode(code) {
            vscode.postMessage({ command: 'insertCode', code: code });
        }

        function filterFindings() {
            displayFindings();
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
