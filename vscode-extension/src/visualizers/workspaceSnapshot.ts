import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

interface Snapshot {
    id: string;
    name: string;
    description: string;
    timestamp: number;
    files: { [path: string]: string };
    findings: any[];
    notes: string;
    memoryState?: any;
    breakpoints?: any[];
    configuration?: any;
}

export class WorkspaceSnapshot {
    private context: vscode.ExtensionContext | undefined;

    public async createSnapshot(context: vscode.ExtensionContext) {
        this.context = context;

        const name = await vscode.window.showInputBox({
            prompt: 'Snapshot name',
            placeHolder: 'Unity exploit v2 - working ROP chain'
        });

        if (!name) return;

        const description = await vscode.window.showInputBox({
            prompt: 'Description (optional)',
            placeHolder: 'Successfully bypassed ASLR, next step is shellcode'
        });

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Creating workspace snapshot...',
            cancellable: false
        }, async (progress) => {
            progress.report({ increment: 0, message: 'Collecting files...' });

            const snapshot: Snapshot = {
                id: Date.now().toString(),
                name,
                description: description || '',
                timestamp: Date.now(),
                files: {},
                findings: this.collectFindings(),
                notes: '',
                configuration: this.collectConfiguration()
            };

            progress.report({ increment: 30, message: 'Saving workspace files...' });
            snapshot.files = await this.collectWorkspaceFiles();

            progress.report({ increment: 60, message: 'Saving editor state...' });
            snapshot.breakpoints = this.collectBreakpoints();

            progress.report({ increment: 90, message: 'Finalizing snapshot...' });
            await this.saveSnapshot(snapshot);

            vscode.window.showInformationMessage(`Snapshot "${name}" created successfully!`);
        });
    }

    public async loadSnapshot(context: vscode.ExtensionContext) {
        this.context = context;

        const snapshots = await this.listSnapshots();
        
        if (snapshots.length === 0) {
            vscode.window.showInformationMessage('No snapshots found');
            return;
        }

        const items = snapshots.map(s => ({
            label: s.name,
            description: new Date(s.timestamp).toLocaleString(),
            detail: s.description,
            snapshot: s
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a snapshot to restore'
        });

        if (!selected) return;

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Restoring workspace snapshot...',
            cancellable: false
        }, async (progress) => {
            progress.report({ increment: 0, message: 'Loading snapshot data...' });
            const snapshot = selected.snapshot;

            progress.report({ increment: 30, message: 'Restoring files...' });
            await this.restoreFiles(snapshot.files);

            progress.report({ increment: 60, message: 'Restoring configuration...' });
            if (snapshot.configuration) {
                await this.restoreConfiguration(snapshot.configuration);
            }

            progress.report({ increment: 90, message: 'Finalizing...' });
            vscode.window.showInformationMessage(`Snapshot "${snapshot.name}" restored!`);
        });
    }

    public async shareSnapshot(context: vscode.ExtensionContext) {
        this.context = context;

        const snapshots = await this.listSnapshots();
        
        if (snapshots.length === 0) {
            vscode.window.showInformationMessage('No snapshots to share');
            return;
        }

        const items = snapshots.map(s => ({
            label: s.name,
            description: new Date(s.timestamp).toLocaleString(),
            snapshot: s
        }));

        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a snapshot to export'
        });

        if (!selected) return;

        const uri = await vscode.window.showSaveDialog({
            filters: {
                'TALON Snapshot': ['talon-snapshot']
            },
            defaultUri: vscode.Uri.file(`${selected.snapshot.name.replace(/[^a-z0-9]/gi, '_')}.talon-snapshot`)
        });

        if (uri) {
            const data = JSON.stringify(selected.snapshot, null, 2);
            fs.writeFileSync(uri.fsPath, data);
            vscode.window.showInformationMessage('Snapshot exported! Share this file with your team.');
        }
    }

    public async importSnapshot(context: vscode.ExtensionContext) {
        this.context = context;

        const uri = await vscode.window.showOpenDialog({
            filters: {
                'TALON Snapshot': ['talon-snapshot']
            },
            canSelectMany: false
        });

        if (!uri || uri.length === 0) return;

        try {
            const data = fs.readFileSync(uri[0].fsPath, 'utf8');
            const snapshot: Snapshot = JSON.parse(data);

            snapshot.id = Date.now().toString();

            await this.saveSnapshot(snapshot);
            vscode.window.showInformationMessage(`Snapshot "${snapshot.name}" imported successfully!`);
        } catch (e) {
            vscode.window.showErrorMessage('Failed to import snapshot: Invalid file format');
        }
    }

    private async collectWorkspaceFiles(): Promise<{ [path: string]: string }> {
        const files: { [path: string]: string } = {};
        
        const talonFiles = await vscode.workspace.findFiles('**/*.talon');
        
        for (const file of talonFiles) {
            try {
                const content = fs.readFileSync(file.fsPath, 'utf8');
                files[vscode.workspace.asRelativePath(file)] = content;
            } catch (e) {
            }
        }

        if (vscode.window.activeTextEditor) {
            const doc = vscode.window.activeTextEditor.document;
            if (doc.languageId === 'talon') {
                const relativePath = vscode.workspace.asRelativePath(doc.uri);
                files[relativePath] = doc.getText();
            }
        }

        return files;
    }

    private collectFindings(): any[] {
        if (this.context) {
            const findingsPath = path.join(this.context.globalStorageUri.fsPath, 'findings.json');
            if (fs.existsSync(findingsPath)) {
                try {
                    const data = fs.readFileSync(findingsPath, 'utf8');
                    return JSON.parse(data);
                } catch (e) {
                    return [];
                }
            }
        }
        return [];
    }

    private collectBreakpoints(): any[] {
        const breakpoints: any[] = [];
        
        return breakpoints;
    }

    private collectConfiguration(): any {
        return {
            target: vscode.workspace.getConfiguration('talon').get('target'),
            architecture: vscode.workspace.getConfiguration('talon').get('architecture'),
            settings: vscode.workspace.getConfiguration('talon')
        };
    }

    private async restoreFiles(files: { [path: string]: string }) {
        const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
        if (!workspaceFolder) {
            vscode.window.showWarningMessage('No workspace folder open. Files not restored.');
            return;
        }

        for (const [relativePath, content] of Object.entries(files)) {
            const fullPath = path.join(workspaceFolder.uri.fsPath, relativePath);
            const dir = path.dirname(fullPath);

            if (!fs.existsSync(dir)) {
                fs.mkdirSync(dir, { recursive: true });
            }

            fs.writeFileSync(fullPath, content);
        }
    }

    private async restoreConfiguration(config: any) {
        if (config.target) {
            await vscode.workspace.getConfiguration('talon').update('target', config.target);
        }
        if (config.architecture) {
            await vscode.workspace.getConfiguration('talon').update('architecture', config.architecture);
        }
    }

    private async saveSnapshot(snapshot: Snapshot) {
        if (!this.context) return;

        const storagePath = this.context.globalStorageUri.fsPath;
        if (!fs.existsSync(storagePath)) {
            fs.mkdirSync(storagePath, { recursive: true });
        }

        const snapshotsPath = path.join(storagePath, 'snapshots');
        if (!fs.existsSync(snapshotsPath)) {
            fs.mkdirSync(snapshotsPath);
        }

        const snapshotFile = path.join(snapshotsPath, `${snapshot.id}.json`);
        fs.writeFileSync(snapshotFile, JSON.stringify(snapshot, null, 2));
    }

    private async listSnapshots(): Promise<Snapshot[]> {
        if (!this.context) return [];

        const snapshotsPath = path.join(this.context.globalStorageUri.fsPath, 'snapshots');
        if (!fs.existsSync(snapshotsPath)) {
            return [];
        }

        const files = fs.readdirSync(snapshotsPath);
        const snapshots: Snapshot[] = [];

        for (const file of files) {
            if (file.endsWith('.json')) {
                try {
                    const data = fs.readFileSync(path.join(snapshotsPath, file), 'utf8');
                    snapshots.push(JSON.parse(data));
                } catch (e) {
                }
            }
        }

        return snapshots.sort((a, b) => b.timestamp - a.timestamp);
    }
}
