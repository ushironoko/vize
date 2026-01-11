# @vizejs/musea-mcp-server

MCP (Model Context Protocol) server for Musea design system integration.

## Features

- **AI Integration** - Connect Musea to AI assistants
- **Component Discovery** - Query available components
- **Design Token Access** - Retrieve design system tokens
- **Documentation** - Access component docs via MCP

## Installation

```bash
npm install -g @vizejs/musea-mcp-server
```

## Usage

### With Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "musea": {
      "command": "musea-mcp-server",
      "args": ["--project", "/path/to/project"]
    }
  }
}
```

### Standalone

```bash
musea-mcp-server --project ./my-vue-app
```

## MCP Tools

- `list_components` - List all components
- `get_component` - Get component details
- `get_variants` - Get component variants
- `get_tokens` - Get design tokens

## License

MIT
