# vize_armature

**Armature** - The structural parser framework for Vize Vue templates.

<p align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei/vize/main/crates/vize_armature/logo.svg" alt="vize_armature logo" width="200" />
</p>

## Name Origin

**Armature** (/ˈɑːrmətʃər/) is the internal skeleton or framework that supports a sculpture during its creation. Just as an armature provides the structural foundation that a sculptor builds upon, `vize_armature` provides the parsing framework that analyzes and structures Vue templates.

The armature is the invisible but essential structure that gives shape to the final work - similarly, this crate provides the parsing infrastructure that extracts structure from raw template text.

## Features

- **High-Performance Tokenizer**: State machine-based HTML tokenizer optimized for Vue template syntax
- **Vue Template Parser**: Full support for Vue-specific syntax including:
  - Directives (`v-if`, `v-for`, `v-bind`, `v-on`, etc.)
  - Interpolation (`{{ expression }}`)
  - Custom delimiters
  - Dynamic arguments (`v-bind:[key]`)
  - Modifiers (`@click.stop.prevent`)
- **Arena Allocation**: Zero-copy parsing using bumpalo arena allocator
- **Error Recovery**: Graceful handling of malformed templates with detailed error messages
- **Source Location Tracking**: Precise line/column tracking for IDE integration

## Dependencies

- `vize_carton` - Core types and arena allocator
- `vize_relief` - AST definitions, errors, and options

## Usage

```rust
use vize_armature::{parse, parse_with_options, ParserOptions};
use vize_carton::Bump;

// Simple parsing
let allocator = Bump::new();
let source = "<div>{{ message }}</div>";
let (ast, errors) = parse(&allocator, source);

// Parsing with options
let options = ParserOptions::default();
let (ast, errors) = parse_with_options(&allocator, source, options);
```

## Architecture

```
vize_armature
├── tokenizer.rs  # State machine tokenizer
└── parser.rs     # AST builder using tokenizer callbacks
```

The parser uses a callback-based design where the tokenizer emits events that the parser handles to build the AST.

## License

MIT
