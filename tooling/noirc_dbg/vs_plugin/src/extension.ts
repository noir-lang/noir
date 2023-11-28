// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {

	// Use the console to output diagnostic information (console.log) and errors (console.error)
	// This line of code will only be executed once when your extension is activated
	console.log(`The debugger loaded code from: ${vscode.workspace.workspaceFolders?.at(0)?.uri}`);

	const options: vscode.OpenDialogOptions = {
		canSelectMany: false,
    canSelectFolders: true,
    canSelectFiles: false,
		title: "Choose noir binary directory",
		openLabel: 'Open',
		defaultUri: vscode.Uri.file(""),
		filters: {
      // 'Noir files': ['*.nr'],
      'All files': ['*']
		}
	};

	context.subscriptions.push(vscode.commands.registerCommand('extension.noirc_dbg.selectDir', async (_config: any) => {
		return vscode.window.showOpenDialog(options).then((fileUri) => {
    		if (fileUri && fileUri[0]) {
				return fileUri[0].fsPath;
			}
      return null;
		})
	}));

}

// This method is called when your extension is deactivated
export function deactivate() {}

