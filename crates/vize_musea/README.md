# vize_musea

**Musea** - Component gallery and documentation for Vize Vue components.

<p align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei/vize/main/crates/vize_musea/logo.svg" alt="vize_musea logo" width="200" />
</p>

## Name Origin

**Musea** (plural of museum) represents a gallery space where art is displayed and documented. Similarly, `vize_musea` provides a gallery for Vue components, allowing developers to view and interact with components in isolation - similar to Storybook.

## Features

- **Component Gallery** - Browse Vue components visually
- **Art Files** - Document components with `*.art.vue`
- **Variants** - Showcase component states
- **Design Tokens** - Centralized design system (Palette)

## Usage

### Art File Parser

```rust
use vize_musea::art::{parse_art, ArtDescriptor};

let art = parse_art(source)?;
println!("Title: {}", art.title);
for variant in art.variants {
    println!("  - {}", variant.name);
}
```

### Design Tokens (Palette)

```rust
use vize_musea::palette::{Palette, Token};

let palette = Palette::from_file("palette.toml")?;
let color = palette.get("colors.primary")?;
```

### Docs Generator

```rust
use vize_musea::docs::generate_docs;

let markdown = generate_docs(&art)?;
```

## License

MIT
