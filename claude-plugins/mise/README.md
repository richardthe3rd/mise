# mise Claude Code Plugin

MCP server integration for the mise development tool manager. This plugin enables Claude Code to interact with mise for managing development tools, tasks, environment variables, and configurations.

## Installation

### Via Marketplace (Recommended)

Add the mise plugin marketplace to Claude Code:

```bash
/plugin marketplace add jdx/mise
```

Then install the mise-en-place plugin:

```bash
/plugin install mise-en-place@mise-en-place
```

### Direct Install (Local Development)

Alternatively, install directly from a local clone:

```bash
/plugin install /path/to/mise/claude-plugins/mise
```

## Requirements

You must have [mise](https://mise.jdx.dev) installed on your system with experimental features enabled.

**Install mise:**
- macOS: `brew install mise`
- Linux: `curl https://mise.run | sh`
- Windows: See [mise installation docs](https://mise.jdx.dev/getting-started.html)

**Enable experimental features:**
```bash
export MISE_EXPERIMENTAL=1
```

## Features

This plugin provides MCP server integration with access to:

### Resources

- **`mise://tools`** - List active tools in your mise configuration
- **`mise://tools?include_inactive=true`** - List all installed tools
- **`mise://tasks`** - List all available tasks with configurations
- **`mise://env`** - Show environment variables from mise
- **`mise://config`** - Display mise configuration files and project root

### Use Cases

Claude Code can now help you with:

- **Tool Management**: Query available tools, check versions, understand tool configurations
- **Task Execution**: View task definitions, understand dependencies, and task workflows
- **Environment Setup**: Access environment variables, understand project configuration
- **Project Discovery**: Browse mise.toml files, understand project structure

## Usage Examples

Once installed, Claude Code can automatically access mise data through the MCP server:

**Ask about tools:**
- "What tools are configured in this mise project?"
- "Which versions of node are installed?"
- "Show me all the Python tools available"

**Ask about tasks:**
- "What mise tasks are available?"
- "Show me the build task configuration"
- "What tasks depend on the test task?"

**Ask about environment:**
- "What environment variables does mise set?"
- "Show me the mise configuration for this project"

## How It Works

The plugin configures Claude Code to communicate with mise's built-in MCP server:

```json
{
  "mcp_servers": {
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

When Claude Code needs mise information, it:
1. Starts the `mise mcp` server via stdin/stdout
2. Queries resources like `mise://tools` or `mise://tasks`
3. Receives JSON responses with tool/task/env data
4. Uses this data to answer questions and help with development tasks

## Troubleshooting

**Server fails to start:**
- Ensure mise is installed: `mise --version`
- Verify experimental features: `echo $MISE_EXPERIMENTAL`
- Check that `mise mcp` command works: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | mise mcp`

**Resources not showing:**
- Ensure you're in a directory with a `mise.toml` file
- Verify your mise configuration: `mise ls`
- Check task configuration: `mise tasks`

**Permission errors:**
- Verify mise binary is executable
- Check that mise is in your PATH

## About mise

mise is a polyglot development tool version manager that replaces tools like asdf, nvm, pyenv, rbenv, etc. It manages:

- Runtime versions (Node.js, Python, Ruby, etc.)
- Development tools (via aqua, npm, cargo, etc.)
- Environment variables and configuration
- Task automation (like make, but better)

**Learn more:** https://mise.jdx.dev

## Contributing

This plugin is part of the mise project. Contributions are welcome!

**Report Issues:** https://github.com/jdx/mise/issues

## License

MIT License - See the mise project for full license details.
