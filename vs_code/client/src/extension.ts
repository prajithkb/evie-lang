/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

// import * as cp from 'child_process';
// import * as util from 'util';
// import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
	// // The server is implemented in node
	// const serverModule = context.asAbsolutePath(
	// 	path.join('cargo', 'run', '--manifest-path', 'asd', '--package', 'evie_language_server', '--bin', 'evie_language_server')
	// );
	// The debug options for the server
	// --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
	// const debugOptions = { execArgv: ['--nolazy', '--inspect=6009'] };

	// If the extension is launched in debug mode then the debug server options are used
	// Otherwise the run options are used
	const serverOptions: ServerOptions = {
		run: {
			command: 'cargo',
			args: ['run', '--manifest-path', '/Users/kprajith/workspace/rust/evie-lang/Cargo.toml', '--package', 'evie_language_server', '--bin', 'evie_language_server']

		},
		debug: {
			command: 'cargo',
			args: ['run', '--manifest-path', '/Users/kprajith/workspace/rust/evie-lang/Cargo.toml', '--package', 'evie_language_server', '--bin', 'evie_language_server']
		},
		transport: TransportKind.stdio
	};

	// const serverOptions: ServerOptions = () => Promise.resolve(cp.exec("cargo run --manifest-path /Users/kprajith/workspace/rust/evie-lang/Cargo.toml --package evie_language_server --bin evie_language_server"));

	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [{ scheme: 'file', language: 'evie' }],
		synchronize: {
			// Notify the server about file changes to '.clientrc files contained in the workspace
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		},
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'evieLanguageServer',
		'Evie Language Server',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
