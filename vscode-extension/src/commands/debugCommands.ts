import * as vscode from 'vscode';
import { MemoryVisualizer } from '../visualizers/memoryVisualizer';

export function registerDebugCommands(
    context: vscode.ExtensionContext,
    memoryVisualizer: MemoryVisualizer
) {
    context.subscriptions.push(
        vscode.commands.registerCommand('talon.checksec', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor) {
                vscode.window.showErrorMessage('No file open');
                return;
            }

            const filePath = editor.document.uri.fsPath;
            const terminal = vscode.window.createTerminal('TALON Checksec');
            terminal.show();
            terminal.sendText(`talon analyze "${filePath}"`);
        }),
        
        vscode.commands.registerCommand('talon.vmmap', async () => {
            memoryVisualizer.showMemoryMappings(context);
        }),
        
        vscode.commands.registerCommand('talon.searchMemory', async () => {
            const searchTerm = await vscode.window.showInputBox({
                prompt: 'Enter string or hex pattern to search',
                placeHolder: '/bin/sh or 0x4141414141'
            });
            
            if (searchTerm) {
                memoryVisualizer.searchMemory(context, searchTerm);
            }
        }),
        
        vscode.commands.registerCommand('talon.telescope', async () => {
            const address = await vscode.window.showInputBox({
                prompt: 'Enter address to telescope from',
                placeHolder: '0x7fffffffe000'
            });
            
            if (address) {
                memoryVisualizer.telescope(context, address);
            }
        })
    );
}
