import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

function startClient() {
    const serverOptions: ServerOptions = {
        command: 'verse-lsp',
        args: [],
        transport: TransportKind.stdio
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'verse' }],
        outputChannelName: 'Verse Language Server'
    };

    client = new LanguageClient('verse-lsp', 'Verse LSP', serverOptions, clientOptions);
    client.start();
}

export function activate(context: vscode.ExtensionContext) {
    startClient();
    
    context.subscriptions.push(
        vscode.commands.registerCommand('verse.restart', () => {
            if (client) {
                client.stop();
            }
            startClient();
        })
    );
}

export function deactivate() {
    if (client) {
        client.stop();
    }
}