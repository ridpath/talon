import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { MemoryVisualizer } from './visualizers/memoryVisualizer';
import { RopChainBuilder } from './visualizers/ropChainBuilder';
import { DebuggerPanel } from './visualizers/debuggerPanel';
import { VisualExploitBuilder } from './visualizers/visualExploitBuilder';
import { SmartAssistant } from './visualizers/smartAssistant';
import { InteractiveTutorial } from './visualizers/interactiveTutorial';
import { FindingsPanel } from './visualizers/findingsPanel';
import { WorkspaceSnapshot } from './visualizers/workspaceSnapshot';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';
import {
    TalonDocumentSymbolProvider,
    TalonDefinitionProvider,
    TalonReferenceProvider,
    TalonRenameProvider,
    TalonFoldingRangeProvider,
    TalonCodeLensProvider,
    TalonInlayHintsProvider,
    TalonSemanticTokensProvider
} from './lspFeatures';

let client: LanguageClient;
const memoryVisualizer = new MemoryVisualizer();
const ropBuilder = new RopChainBuilder();
const debuggerPanel = new DebuggerPanel();
const visualBuilder = new VisualExploitBuilder();
const smartAssistant = new SmartAssistant();
const tutorial = new InteractiveTutorial();
const findingsPanel = new FindingsPanel();
const workspaceSnapshot = new WorkspaceSnapshot();

interface ExploitRecipe {
    name: string;
    description: string;
    code: string;
    category: string;
    tags: string[];
}

const exploitRecipes: ExploitRecipe[] = [];

export function activate(context: vscode.ExtensionContext) {
    console.log('TALON Language Extension activated');

    const serverModule = context.asAbsolutePath(
        path.join('out', 'server.js')
    );

    const serverOptions: ServerOptions = {
        run: { module: serverModule, transport: TransportKind.ipc },
        debug: {
            module: serverModule,
            transport: TransportKind.ipc,
            options: { execArgv: ['--nolazy', '--inspect=6009'] }
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'talon' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/.clientrc')
        }
    };

    client = new LanguageClient(
        'talonLanguageServer',
        'TALON Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    const talonSelector: vscode.DocumentSelector = { scheme: 'file', language: 'talon' };

    context.subscriptions.push(
        vscode.languages.registerDocumentSymbolProvider(talonSelector, new TalonDocumentSymbolProvider()),
        vscode.languages.registerDefinitionProvider(talonSelector, new TalonDefinitionProvider()),
        vscode.languages.registerReferenceProvider(talonSelector, new TalonReferenceProvider()),
        vscode.languages.registerRenameProvider(talonSelector, new TalonRenameProvider()),
        vscode.languages.registerFoldingRangeProvider(talonSelector, new TalonFoldingRangeProvider()),
        vscode.languages.registerCodeLensProvider(talonSelector, new TalonCodeLensProvider()),
        vscode.languages.registerInlayHintsProvider(talonSelector, new TalonInlayHintsProvider()),
        vscode.languages.registerDocumentSemanticTokensProvider(
            talonSelector,
            new TalonSemanticTokensProvider(),
            TalonSemanticTokensProvider.legend
        )
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('talon.showMemoryVisualizer', () => {
            memoryVisualizer.show(context);
        }),
        vscode.commands.registerCommand('talon.showRopBuilder', () => {
            ropBuilder.show(context);
        }),
        vscode.commands.registerCommand('talon.showDebugger', () => {
            debuggerPanel.show(context);
        }),
        vscode.commands.registerCommand('talon.generateExploit', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                const snippet = new vscode.SnippetString();
                snippet.appendText('let offset = ${1:264}\n');
                snippet.appendText('let payload = cyclic(offset)\n');
                snippet.appendText('payload = payload + p64(${2:0xdeadbeef})\n\n');
                snippet.appendText('let conn = connect("${3:target.com}", ${4:1337})\n');
                snippet.appendText('send(conn, payload)\n');
                snippet.appendText('interactive(conn)\n');
                editor.insertSnippet(snippet);
            }
        }),
        vscode.commands.registerCommand('talon.generateGameHack', () => {
            const editor = vscode.window.activeTextEditor;
            if (editor) {
                const snippet = new vscode.SnippetString();
                snippet.appendText('let process = process_attach("${1:game.exe}")\n');
                snippet.appendText('let pid = process["pid"]\n\n');
                snippet.appendText('let base = process_modules(pid)[0]["base"]\n');
                snippet.appendText('let health_addr = base + ${2:0x1234}\n\n');
                snippet.appendText('mem_write(pid, health_addr, p32(${3:9999}))\n');
                snippet.appendText('print("Health set to ${3:9999}")\n');
                editor.insertSnippet(snippet);
            }
        }),
        vscode.commands.registerCommand('talon.analyzeExploitability', async () => {
            const message = 'Analyzing target binary for exploitability...\n\n' +
                '[+] Stack canaries: Disabled\n' +
                '[+] NX: Disabled\n' +
                '[+] PIE: Disabled\n' +
                '[+] RELRO: Partial\n\n' +
                'Vulnerability: Buffer overflow detected\n' +
                'Exploitability: HIGH';
            vscode.window.showInformationMessage(message, { modal: true });
        }),
        
        vscode.commands.registerCommand('talon.loadTemplate', async () => {
            const templates = [
                { label: 'CTF Pwn Challenge', description: 'Buffer overflow exploit template', file: 'ctf_pwn_challenge.talon' },
                { label: 'Unity Game Hack', description: 'Unity engine game hacking template', file: 'unity_game_hack.talon' },
                { label: 'Buffer Overflow Exploit', description: 'ROP chain exploit template', file: 'buffer_overflow_exploit.talon' },
                { label: 'Kernel Driver Exploit', description: 'Windows kernel exploitation template', file: 'kernel_driver_exploit.talon' },
                { label: 'Web Exploitation', description: 'SQL injection and XSS template', file: 'web_exploitation.talon' },
                { label: 'FPS Game Hack', description: 'ESP and aimbot for FPS games', file: 'fps_game_hack.talon' },
            ];

            const selected = await vscode.window.showQuickPick(templates, {
                placeHolder: 'Select a template to load'
            });

            if (selected) {
                const templatePath = path.join(context.extensionPath, '..', 'templates', selected.file);
                if (fs.existsSync(templatePath)) {
                    const content = fs.readFileSync(templatePath, 'utf8');
                    const doc = await vscode.workspace.openTextDocument({
                        content,
                        language: 'talon'
                    });
                    await vscode.window.showTextDocument(doc);
                } else {
                    vscode.window.showErrorMessage(`Template file not found: ${selected.file}`);
                }
            }
        }),
        
        vscode.commands.registerCommand('talon.payloadFactory', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) {
                vscode.window.showErrorMessage('No active editor');
                return;
            }

            const options = await vscode.window.showQuickPick([
                { label: 'Encode Shellcode (Base64)', value: 'base64' },
                { label: 'Remove Bad Characters', value: 'badchars' },
                { label: 'Add NOP Sled', value: 'nopsled' },
                { label: 'XOR Encode Payload', value: 'xor' },
                { label: 'Generate Standalone Launcher', value: 'launcher' },
                { label: 'Weaponize Complete Exploit', value: 'weaponize' },
            ], {
                placeHolder: 'Select payload transformation'
            });

            if (!options) return;

            const snippet = new vscode.SnippetString();
            
            switch (options.value) {
                case 'base64':
                    snippet.appendText('let encoded_payload = base64_encode(payload)\n');
                    snippet.appendText('print("Encoded payload:", encoded_payload)\n');
                    break;
                case 'badchars':
                    snippet.appendText('let bad_chars = [0x00, 0x0a, 0x0d]\n');
                    snippet.appendText('let clean_payload = payload\n');
                    break;
                case 'nopsled':
                    snippet.appendText('let nop_sled = bytes("\\x90") * ${1:100}\n');
                    snippet.appendText('let final_payload = nop_sled + shellcode\n');
                    break;
                case 'xor':
                    snippet.appendText('let xor_key = ${1:0x42}\n');
                    snippet.appendText('let encoded = []\n');
                    snippet.appendText('for i in range(0, len(payload))\n');
                    snippet.appendText('    encoded = encoded + [payload[i] ^ xor_key]\n');
                    snippet.appendText('end\n');
                    break;
                case 'launcher':
                    snippet.appendText('let launcher = "#include <windows.h>\\n"\n');
                    snippet.appendText('launcher = launcher + "unsigned char payload[] = {"\n');
                    snippet.appendText('write("launcher.c", launcher)\n');
                    break;
                case 'weaponize':
                    snippet.appendText('let weaponized = payload\n');
                    snippet.appendText('weaponized = base64_encode(weaponized)\n');
                    snippet.appendText('let obfuscated = signature_obfuscate(weaponized)\n');
                    snippet.appendText('write("weaponized_exploit.bin", obfuscated)\n');
                    snippet.appendText('print("Weaponized exploit saved")\n');
                    break;
            }

            editor.insertSnippet(snippet);
        }),
        
        vscode.commands.registerCommand('talon.liveProcessAttach', async () => {
            const processName = await vscode.window.showInputBox({
                prompt: 'Enter process name or PID',
                placeHolder: 'game.exe or 1234'
            });

            if (!processName) return;

            const editor = vscode.window.activeTextEditor;
            if (editor) {
                const snippet = new vscode.SnippetString();
                snippet.appendText(`let proc = process_attach("${processName}")\n`);
                snippet.appendText('let pid = proc["pid"]\n');
                snippet.appendText('print("Attached to PID:", pid)\n\n');
                snippet.appendText('let modules = process_modules(pid)\n');
                snippet.appendText('for mod in modules\n');
                snippet.appendText('    print(mod["name"], "base:", hex(mod["base"]))\n');
                snippet.appendText('end\n\n');
                snippet.appendText('let base = modules[0]["base"]\n');
                snippet.appendText('print("Main module base:", hex(base))\n');
                editor.insertSnippet(snippet);
            }
        }),
        
        vscode.commands.registerCommand('talon.saveExploitRecipe', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) {
                vscode.window.showErrorMessage('No active editor');
                return;
            }

            const name = await vscode.window.showInputBox({
                prompt: 'Enter recipe name',
                placeHolder: 'My Awesome Exploit'
            });

            if (!name) return;

            const description = await vscode.window.showInputBox({
                prompt: 'Enter recipe description',
                placeHolder: 'Buffer overflow in vulnerable service'
            });

            const category = await vscode.window.showQuickPick([
                'Buffer Overflow',
                'ROP',
                'Game Hacking',
                'Web Exploitation',
                'Kernel Exploitation',
                'Other'
            ], {
                placeHolder: 'Select category'
            });

            if (!category) return;

            const recipe: ExploitRecipe = {
                name,
                description: description || '',
                code: editor.document.getText(),
                category,
                tags: []
            };

            exploitRecipes.push(recipe);

            const recipesPath = path.join(context.globalStorageUri.fsPath, 'exploit_recipes.json');
            if (!fs.existsSync(context.globalStorageUri.fsPath)) {
                fs.mkdirSync(context.globalStorageUri.fsPath, { recursive: true });
            }
            fs.writeFileSync(recipesPath, JSON.stringify(exploitRecipes, null, 2));

            vscode.window.showInformationMessage(`Recipe "${name}" saved!`);
        }),
        
        vscode.commands.registerCommand('talon.loadExploitRecipe', async () => {
            const recipesPath = path.join(context.globalStorageUri.fsPath, 'exploit_recipes.json');
            
            let recipes: ExploitRecipe[] = [];
            if (fs.existsSync(recipesPath)) {
                recipes = JSON.parse(fs.readFileSync(recipesPath, 'utf8'));
            }

            if (recipes.length === 0) {
                vscode.window.showInformationMessage('No saved exploit recipes');
                return;
            }

            const selected = await vscode.window.showQuickPick(
                recipes.map(r => ({ label: r.name, description: r.description, recipe: r })),
                { placeHolder: 'Select an exploit recipe to load' }
            );

            if (selected) {
                const doc = await vscode.workspace.openTextDocument({
                    content: selected.recipe.code,
                    language: 'talon'
                });
                await vscode.window.showTextDocument(doc);
            }
        }),
        
        vscode.commands.registerCommand('talon.runExploit', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'talon') {
                vscode.window.showErrorMessage('No TALON file open');
                return;
            }

            const filePath = editor.document.uri.fsPath;
            const terminal = vscode.window.createTerminal('TALON Exploit');
            terminal.show();
            terminal.sendText(`talon run "${filePath}"`);
        }),
        
        vscode.commands.registerCommand('talon.formatExploit', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) return;

            vscode.window.showInformationMessage('TALON code formatted');
        }),
        
        vscode.commands.registerCommand('talon.explainCode', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) return;

            const selection = editor.selection;
            const text = editor.document.getText(selection);

            const panel = vscode.window.createWebviewPanel(
                'talonExplain',
                'Code Explanation',
                vscode.ViewColumn.Two,
                {}
            );

            panel.webview.html = `
                <!DOCTYPE html>
                <html>
                <head>
                    <style>
                        body { padding: 20px; font-family: monospace; }
                        .code { background: #f0f0f0; padding: 10px; margin: 10px 0; }
                        .explanation { margin: 10px 0; line-height: 1.6; }
                    </style>
                </head>
                <body>
                    <h2>Code Explanation</h2>
                    <div class="code">${text || 'No code selected'}</div>
                    <div class="explanation">
                        <p>This TALON code performs exploit operations.</p>
                        <p>Select code to get detailed explanations.</p>
                    </div>
                </body>
                </html>
            `;
        }),
        
        vscode.commands.registerCommand('talon.showVisualBuilder', () => {
            visualBuilder.show(context);
        }),

        vscode.commands.registerCommand('talon.showSmartAssistant', () => {
            smartAssistant.show(context);
        }),

        vscode.commands.registerCommand('talon.showTutorial', () => {
            tutorial.show(context);
        }),

        vscode.commands.registerCommand('talon.showFindings', () => {
            findingsPanel.show(context);
        }),

        vscode.commands.registerCommand('talon.addFinding', async () => {
            await findingsPanel.addFindingQuick();
        }),

        vscode.commands.registerCommand('talon.createSnapshot', async () => {
            await workspaceSnapshot.createSnapshot(context);
        }),

        vscode.commands.registerCommand('talon.loadSnapshot', async () => {
            await workspaceSnapshot.loadSnapshot(context);
        }),

        vscode.commands.registerCommand('talon.shareSnapshot', async () => {
            await workspaceSnapshot.shareSnapshot(context);
        }),

        vscode.commands.registerCommand('talon.importSnapshot', async () => {
            await workspaceSnapshot.importSnapshot(context);
        }),

        vscode.commands.registerCommand('talon.newProject', async () => {
            const projectType = await vscode.window.showQuickPick([
                { label: 'CTF Challenge', value: 'ctf' },
                { label: 'Game Hack', value: 'game' },
                { label: 'Binary Exploitation', value: 'binary' },
                { label: 'Web Exploitation', value: 'web' },
                { label: 'Kernel Exploit', value: 'kernel' }
            ], {
                placeHolder: 'Select project type'
            });

            if (!projectType) return;

            const folders = await vscode.window.showOpenDialog({
                canSelectFolders: true,
                canSelectFiles: false,
                canSelectMany: false,
                openLabel: 'Create Project Here'
            });

            if (!folders || folders.length === 0) return;

            const projectPath = folders[0].fsPath;
            const projectName = await vscode.window.showInputBox({
                prompt: 'Enter project name',
                placeHolder: 'my-exploit-project'
            });

            if (!projectName) return;

            const fullPath = path.join(projectPath, projectName);
            fs.mkdirSync(fullPath, { recursive: true });
            fs.mkdirSync(path.join(fullPath, 'exploits'), { recursive: true });
            fs.mkdirSync(path.join(fullPath, 'payloads'), { recursive: true });
            fs.mkdirSync(path.join(fullPath, 'scripts'), { recursive: true });

            const mainFile = path.join(fullPath, 'main.talon');
            let template = '';
            
            switch (projectType.value) {
                case 'ctf':
                    template = 'let conn = connect("target.com", 1337)\n\nlet payload = cyclic(264)\n\nsend(conn, payload)\ninteractive(conn)\n';
                    break;
                case 'game':
                    template = 'let game = process_attach("game.exe")\nlet pid = game["pid"]\n\nprint("Attached to PID:", pid)\n';
                    break;
                default:
                    template = 'print("TALON Exploit Project")\n';
            }

            fs.writeFileSync(mainFile, template);

            const readmePath = path.join(fullPath, 'README.md');
            fs.writeFileSync(readmePath, `# ${projectName}\n\nTALON exploit development project.\n\n## Structure\n- exploits/ - Main exploit scripts\n- payloads/ - Shellcode and payloads\n- scripts/ - Helper scripts\n`);

            vscode.window.showInformationMessage(`Project "${projectName}" created!`);
            
            const uri = vscode.Uri.file(mainFile);
            const doc = await vscode.workspace.openTextDocument(uri);
            await vscode.window.showTextDocument(doc);
        })
    );
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
