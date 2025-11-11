# Alescript Web Playground

This directory contains the WebAssembly-based web playground for alescript.

## Features

- **Live Code Editor**: Write alescript code in the textarea on the left
- **Instant Execution**: Click "Run" to execute your code and see results on the right
- **Example Programs**: Select from predefined examples to learn alescript syntax
- **Modern UI**: Beautiful, responsive interface with gradient design

## Building

To build the WebAssembly module:

```bash
# From the project root directory
wasm-pack build --target web --out-dir web/pkg
```

This will generate:
- `web/pkg/alescript.js` - JavaScript bindings
- `web/pkg/alescript_bg.wasm` - WebAssembly binary
- Other supporting files

## Running

To run the playground locally, you need to serve the files over HTTP (not file://):

```bash
# Using Python
cd web
python3 -m http.server 8080

# Using Node.js (if you have http-server installed)
cd web
npx http-server

# Using Rust (if you have miniserve installed)
cd web
miniserve . --index index.html
```

Then open your browser to `http://localhost:8080`

## Examples

The playground includes three example programs:

1. **Hello World** - Basic output with the `toast` statement
2. **Fibonacci Sequence** - Demonstrates recipes (functions), loops, and variable manipulation
3. **Brewing Demo** - Shows off all the brewing-themed features of alescript

## Keyboard Shortcuts

- **Ctrl+Enter** (or **Cmd+Enter** on Mac): Run the current code

## How It Works

1. The alescript interpreter is compiled to WebAssembly using wasm-bindgen
2. JavaScript loads the WASM module and provides a bridge to call the interpreter
3. Code execution happens entirely in the browser - no server required!
4. Output is captured and displayed in real-time

## Files

- `index.html` - Main playground page with editor and output
- `pkg/` - Generated WebAssembly files (git-ignored)
- `README.md` - This file
