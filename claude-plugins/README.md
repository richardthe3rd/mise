# mise Claude Code Plugin Marketplace

This directory contains a minimal Claude Code plugin for mise with MCP server integration.

## Structure

```
mise/                                  # Repository root
├── .claude-plugin/
│   └── marketplace.json              # Marketplace registry (repo root)
└── claude-plugins/
    ├── README.md                     # This file
    └── mise/
        ├── .claude-plugin/
        │   └── plugin.json           # Plugin manifest
        └── README.md                 # Plugin documentation
```

## Usage

### Install from GitHub (shorthand)

```bash
/plugin marketplace add jdx/mise
/plugin install mise-en-place@mise-en-place
```

### Install from GitHub (full URL)

```bash
/plugin marketplace add https://github.com/jdx/mise.git
/plugin install mise-en-place@mise-en-place
```

### Install locally (development)

```bash
/plugin marketplace add /path/to/mise
/plugin install mise-en-place@mise-en-place
```

## What This Provides

The mise plugin configures Claude Code to use the `mise mcp` server, providing access to:

- **`mise://tools`** - List tools and their versions
- **`mise://tasks`** - List all tasks
- **`mise://env`** - Environment variables
- **`mise://config`** - Configuration info

## Requirements

- [mise](https://mise.jdx.dev) installed
- `MISE_EXPERIMENTAL=1` (automatically set by the plugin)

## Validation

Both marketplace and plugin validate successfully:

```bash
# At repo root
claude plugin validate .
# ✔ Validation passed with warnings

# At plugin directory
cd claude-plugins/mise
claude plugin validate .
# ✔ Validation passed
```

## How It Works

The marketplace at repo root (`.claude-plugin/marketplace.json`) points to the plugin:

```json
{
  "plugins": [
    {
      "name": "mise",
      "source": "./claude-plugins/mise"
    }
  ]
}
```

The plugin (`.claude-plugin/plugin.json`) configures the MCP server:

```json
{
  "mcpServers": {
    "mise": {
      "command": "mise",
      "args": ["mcp"],
      "env": {
        "MISE_EXPERIMENTAL": "1"
      }
    }
  }
}
```

This is a minimal example - just the MCP server configuration, no skills, hooks, or commands yet.
