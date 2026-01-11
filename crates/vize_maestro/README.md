# vize_maestro

**Maestro** - Language Server Protocol implementation for Vize Vue templates.

<p align="center">
  <img src="./logo.svg" alt="vize_maestro logo" width="200" />
</p>

## Name Origin

**Maestro** is a master conductor who coordinates an orchestra, bringing together all the instruments in harmony. Similarly, `vize_maestro` orchestrates all the Vize compiler tools to provide a seamless IDE experience through the Language Server Protocol.

## Features

- **Diagnostics** - Parse errors, lint warnings, type errors
- **Completion** - Vue directives, components, Composition API
- **Hover** - Type information, documentation
- **Go to Definition** - Template to script navigation
- **Find References** - Cross-SFC reference search
- **Rename** - Safe identifier renaming
- **Semantic Tokens** - Vue-specific syntax highlighting
- **Code Lens** - Reference counts
- **Code Actions** - Quick fixes, refactoring

## Usage

### As Library

```rust
use vize_maestro::{serve, serve_tcp};

#[tokio::main]
async fn main() {
    // stdio mode (for VS Code)
    serve().await.unwrap();

    // or TCP mode (for debugging)
    // serve_tcp(9527).await.unwrap();
}
```

### With CLI

```bash
vize lsp              # stdio mode
vize lsp --port 9527  # TCP mode
```

## Architecture

```
LSP Client (VS Code)
       ↓
   tower-lsp
       ↓
  MaestroServer
       ↓
  ┌────┴────┐
  ↓         ↓
DocumentStore  VirtualCodeGenerator
       ↓              ↓
   IdeContext ← VirtualDocuments
       ↓
 IDE Services (Hover, Completion, Definition, ...)
```

## License

MIT
