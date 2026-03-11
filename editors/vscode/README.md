# cohdl VS Code Extension

Language support for the cohdl hardware description language, including syntax
highlighting and integration with the `cohdl-lsp` language server.

## Development

1. Install dependencies:

   ```sh
   cd editors/vscode
   npm install
   ```

2. Open the `editors/vscode/` folder in VS Code.

3. Press **F5** to launch the Extension Development Host with the extension
   loaded.

## Building and Installing

1. Install the VS Code Extension packaging tool:

   ```sh
   npm install -g @vscode/vsce
   ```

2. Build the `.vsix` package:

   ```sh
   cd editors/vscode
   vsce package
   ```

   This produces `cohdl-lang-0.1.0.vsix`.

3. Install the extension:

   ```sh
   code --install-extension cohdl-lang-0.1.0.vsix
   ```

## Configuration

### `cohdl.serverPath`

Absolute path to the `cohdl-lsp` binary. If left empty (the default), the
extension looks for `cohdl-lsp` on your `PATH`.

Open **Settings** and search for `cohdl.serverPath`, then set it to the full
path of your compiled `cohdl-lsp` binary, for example:

```
/home/you/cohdl/target/release/cohdl-lsp
```

### Restart Language Server

Run the **cohdl: Restart Language Server** command from the Command Palette
(`Ctrl+Shift+P`) to stop and restart the language server without reloading the
window.
