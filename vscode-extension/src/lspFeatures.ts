import * as vscode from 'vscode';

export class TalonDocumentSymbolProvider implements vscode.DocumentSymbolProvider {
    provideDocumentSymbols(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.SymbolInformation[] | vscode.DocumentSymbol[]> {
        const symbols: vscode.DocumentSymbol[] = [];
        const text = document.getText();
        const lines = text.split('\n');

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            
            const funcMatch = line.match(/^func\s+(\w+)\s*\(/);
            if (funcMatch) {
                const range = new vscode.Range(i, 0, i, line.length);
                const symbol = new vscode.DocumentSymbol(
                    funcMatch[1],
                    '',
                    vscode.SymbolKind.Function,
                    range,
                    range
                );
                symbols.push(symbol);
            }

            const letMatch = line.match(/^let\s+(\w+)\s*=/);
            if (letMatch) {
                const range = new vscode.Range(i, 0, i, line.length);
                const symbol = new vscode.DocumentSymbol(
                    letMatch[1],
                    '',
                    vscode.SymbolKind.Variable,
                    range,
                    range
                );
                symbols.push(symbol);
            }
        }

        return symbols;
    }
}

export class TalonDefinitionProvider implements vscode.DefinitionProvider {
    provideDefinition(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.Definition | vscode.LocationLink[]> {
        const wordRange = document.getWordRangeAtPosition(position);
        if (!wordRange) {
            return null;
        }

        const word = document.getText(wordRange);
        const text = document.getText();
        const lines = text.split('\n');

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            
            const funcMatch = line.match(new RegExp(`^func\\s+${word}\\s*\\(`));
            if (funcMatch) {
                return new vscode.Location(document.uri, new vscode.Position(i, 0));
            }

            const letMatch = line.match(new RegExp(`^let\\s+${word}\\s*=`));
            if (letMatch) {
                return new vscode.Location(document.uri, new vscode.Position(i, 0));
            }
        }

        return null;
    }
}

export class TalonReferenceProvider implements vscode.ReferenceProvider {
    provideReferences(
        document: vscode.TextDocument,
        position: vscode.Position,
        context: vscode.ReferenceContext,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.Location[]> {
        const wordRange = document.getWordRangeAtPosition(position);
        if (!wordRange) {
            return [];
        }

        const word = document.getText(wordRange);
        const text = document.getText();
        const lines = text.split('\n');
        const locations: vscode.Location[] = [];

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const regex = new RegExp(`\\b${word}\\b`, 'g');
            let match;
            
            while ((match = regex.exec(line)) !== null) {
                const pos = new vscode.Position(i, match.index);
                locations.push(new vscode.Location(document.uri, pos));
            }
        }

        return locations;
    }
}

export class TalonRenameProvider implements vscode.RenameProvider {
    provideRenameEdits(
        document: vscode.TextDocument,
        position: vscode.Position,
        newName: string,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.WorkspaceEdit> {
        const wordRange = document.getWordRangeAtPosition(position);
        if (!wordRange) {
            return null;
        }

        const word = document.getText(wordRange);
        const text = document.getText();
        const lines = text.split('\n');
        const edit = new vscode.WorkspaceEdit();

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const regex = new RegExp(`\\b${word}\\b`, 'g');
            let match;
            
            while ((match = regex.exec(line)) !== null) {
                const range = new vscode.Range(
                    i,
                    match.index,
                    i,
                    match.index + word.length
                );
                edit.replace(document.uri, range, newName);
            }
        }

        return edit;
    }

    prepareRename(
        document: vscode.TextDocument,
        position: vscode.Position,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.Range | { range: vscode.Range; placeholder: string }> {
        const wordRange = document.getWordRangeAtPosition(position);
        if (!wordRange) {
            throw new Error('Nothing to rename here');
        }

        const word = document.getText(wordRange);
        return {
            range: wordRange,
            placeholder: word
        };
    }
}

export class TalonFoldingRangeProvider implements vscode.FoldingRangeProvider {
    provideFoldingRanges(
        document: vscode.TextDocument,
        context: vscode.FoldingContext,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.FoldingRange[]> {
        const ranges: vscode.FoldingRange[] = [];
        const text = document.getText();
        const lines = text.split('\n');
        const stack: number[] = [];

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i].trim();
            
            if (line.match(/^(func|if|while|for)\b/)) {
                stack.push(i);
            } else if (line === 'end' && stack.length > 0) {
                const start = stack.pop()!;
                ranges.push(new vscode.FoldingRange(start, i));
            }
        }

        return ranges;
    }
}

export class TalonCodeLensProvider implements vscode.CodeLensProvider {
    provideCodeLenses(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.CodeLens[]> {
        const lenses: vscode.CodeLens[] = [];
        const text = document.getText();
        const lines = text.split('\n');

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            
            const funcMatch = line.match(/^func\s+(\w+)\s*\(/);
            if (funcMatch) {
                const range = new vscode.Range(i, 0, i, line.length);
                
                const runLens = new vscode.CodeLens(range, {
                    title: 'Run Function',
                    command: 'talon.runFunction',
                    arguments: [funcMatch[1]]
                });
                lenses.push(runLens);

                const refsLens = new vscode.CodeLens(range, {
                    title: '0 references',
                    command: 'editor.action.showReferences',
                    arguments: [document.uri, range.start, []]
                });
                lenses.push(refsLens);
            }
        }

        return lenses;
    }
}

export class TalonInlayHintsProvider implements vscode.InlayHintsProvider {
    provideInlayHints(
        document: vscode.TextDocument,
        range: vscode.Range,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.InlayHint[]> {
        const hints: vscode.InlayHint[] = [];
        const text = document.getText(range);
        const lines = text.split('\n');

        for (let i = range.start.line; i <= range.end.line && i < document.lineCount; i++) {
            const line = document.lineAt(i).text;
            
            const p64Match = line.match(/p64\s*\(\s*0x([0-9a-fA-F]+)\s*\)/);
            if (p64Match) {
                const value = parseInt(p64Match[1], 16);
                const hint = new vscode.InlayHint(
                    new vscode.Position(i, line.length),
                    ` = ${value}`,
                    vscode.InlayHintKind.Type
                );
                hint.paddingLeft = true;
                hints.push(hint);
            }
        }

        return hints;
    }
}

export class TalonSemanticTokensProvider implements vscode.DocumentSemanticTokensProvider {
    static readonly legend = new vscode.SemanticTokensLegend(
        ['function', 'variable', 'parameter', 'keyword', 'number', 'string', 'comment'],
        ['declaration', 'definition', 'readonly']
    );

    provideDocumentSemanticTokens(
        document: vscode.TextDocument,
        token: vscode.CancellationToken
    ): vscode.ProviderResult<vscode.SemanticTokens> {
        const builder = new vscode.SemanticTokensBuilder(TalonSemanticTokensProvider.legend);
        const text = document.getText();
        const lines = text.split('\n');

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            
            const keywords = ['let', 'func', 'if', 'while', 'for', 'end', 'return'];
            keywords.forEach(keyword => {
                const regex = new RegExp(`\\b${keyword}\\b`, 'g');
                let match;
                while ((match = regex.exec(line)) !== null) {
                    builder.push(i, match.index, keyword.length, 3, 0);
                }
            });

            const funcMatch = line.match(/func\s+(\w+)/);
            if (funcMatch) {
                const index = line.indexOf(funcMatch[1]);
                builder.push(i, index, funcMatch[1].length, 0, 1);
            }

            const letMatch = line.match(/let\s+(\w+)/);
            if (letMatch) {
                const index = line.indexOf(letMatch[1]);
                builder.push(i, index, letMatch[1].length, 1, 1);
            }
        }

        return builder.build();
    }
}
