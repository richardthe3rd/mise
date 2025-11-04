# mise Claude Code Plugin

Comprehensive Claude Code integration for mise, the polyglot development tool manager. This plugin provides intelligent workflow assistance for developers using mise in their projects.

## Features

### 1. MCP Server Integration

Direct access to mise data through the Model Context Protocol:

- **`mise://tools`** - List active tools in your mise configuration
- **`mise://tools?include_inactive=true`** - List all installed tools
- **`mise://tasks`** - List all available tasks with configurations
- **`mise://env`** - Show environment variables from mise
- **`mise://config`** - Display mise configuration files and project root

### 2. mise-workflow Skill

Intelligent skill that assists with three core development workflows:

#### Understanding Projects
- Inspect available tools and their versions
- Discover tasks and their dependencies
- Check project health with `mise doctor`
- Understand configuration hierarchy

#### Running Commands
- Execute tasks with dependencies
- Run commands with mise-managed tools
- Watch files and re-run on changes
- Debug task execution

#### Adding Capabilities
- Add tools with appropriate backends (npm:, go:, cargo:, pipx:, etc.)
- Create tasks (TOML or file-based)
- Configure task dependencies
- Validate configurations

### 3. Automatic Project Detection

**SessionStart hook** automatically detects mise projects and injects context when you start a session.

**`/mise-en-place:detect` command** provides on-demand project context:

- Shows project root and configuration file hierarchy
- Lists active tools with versions (first 8, with total count)
- Displays key development tasks (first 8-10, with total count)
- Shows full mise.toml configuration
- Fast and efficient (no slow health checks)

## Installation

### Via Marketplace (Recommended)

Add the mise plugin marketplace:

```bash
/plugin marketplace add jdx/mise
```

Then install the mise-en-place plugin:

```bash
/plugin install mise-en-place@mise-en-place
```

### Direct Install (Local Development)

Install directly from a local clone:

```bash
/plugin install /path/to/mise/claude-plugins/mise
```

Or from the repository:

```bash
/plugin install https://github.com/jdx/mise.git#:claude-plugins/mise
```

## Requirements

You must have [mise](https://mise.jdx.dev) installed on your system.

**Install mise:**
- macOS: `brew install mise`
- Linux: `curl https://mise.run | sh`
- Windows: See [mise installation docs](https://mise.jdx.dev/getting-started.html)

**Enable experimental features:**

The plugin automatically sets `MISE_EXPERIMENTAL=1` for the MCP server. No additional configuration needed.

## Usage

### Understanding a Project

The plugin works automatically when you open a mise project. You can also manually trigger detection:

```
/mise-en-place:detect
```

Claude will receive:
- Project root and configuration file hierarchy
- Available tools and versions (with counts)
- Available tasks (with descriptions)
- Full mise.toml configuration

Ask Claude:
- "What tools does this project use?"
- "What tasks are available?"
- "What does the build task do?"
- "Show me the project structure"

### Running Commands

Claude can help you execute tasks and commands:

- "Run the tests"
- "Start the development server"
- "Run the linter and fix issues"
- "Show me the dependency order for the build task"
- "Watch the build task and rebuild on changes"

### Adding Tools and Tasks

Claude can help you add new capabilities:

- "Add prettier to this project"
- "Add golangci-lint as a development tool"
- "Create a task to run all linters"
- "Add Python 3.11 to this project"
- "Create a pre-commit task that runs lint and test"

## How It Works

### MCP Server

The plugin configures Claude Code to communicate with mise's built-in MCP server:

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

When Claude needs mise information, it queries resources like `mise://tools` or `mise://tasks` and receives JSON responses with tool/task/environment data.

### mise-workflow Skill

The `mise-workflow` skill provides procedural knowledge for working with mise projects:

- **SKILL.md** (~450 lines): Quick reference for common workflows
- **references/understanding-projects.md** (~16KB): Deep dive into project inspection
- **references/running-commands.md** (~14KB): Complete guide to task execution
- **references/adding-capabilities.md** (~21KB): Comprehensive tool and task addition guide

The skill uses **progressive disclosure** - Claude starts with the concise SKILL.md and loads detailed references only when needed.

### SessionStart Hook & Detect Command

**SessionStart Hook:** When you enter a directory with a mise configuration, it automatically injects project context.

**Detect Command (`/mise-en-place:detect`):** On-demand project detection that:

1. Detects `mise.toml`, `.mise.toml`, or `config/mise.toml`
2. Queries `mise://config` MCP resource for project root and config files
3. Queries `mise://tools` MCP resource for active tools (shows first 8 + count)
4. Queries `mise://tasks` MCP resource for available tasks (shows first 8-10 + count)
5. Displays full mise.toml configuration
6. Fast execution using MCP resources (no slow shell commands)

This gives Claude immediate context about the project efficiently.

## Example Workflows

### Discovering a New Project

```
You: What's in this project?

Claude: [Receives SessionStart context automatically]
This project uses:
- Node.js 20.11.0
- Python 3.11.7
- npm:prettier for code formatting

Available tasks:
- build: Build the application
- test: Run all tests
- lint: Run linters
- dev: Start development server

No issues detected in mise doctor.
```

### Adding a Linter

```
You: Add eslint and prettier to this project

Claude: I'll add eslint and prettier as npm tools and create tasks for them.

[Runs:]
mise use npm:eslint npm:prettier

[Edits mise.toml to add tasks:]
[tasks.lint]
description = "Run all linters"
depends = ["lint:eslint", "lint:prettier"]

[tasks."lint:eslint"]
description = "Lint JavaScript/TypeScript"
run = "eslint src/"

[tasks."lint:prettier"]
description = "Check code formatting"
run = "prettier --check src/"

[tasks.lint-fix]
description = "Fix linting issues"
run = ["eslint --fix src/", "prettier --write src/"]

[Runs validation:]
mise fmt && mise doctor

Done! You can now run:
- mise run lint (to check)
- mise run lint-fix (to fix)
```

### Running Tasks

```
You: Run the tests

Claude: [Checks task dependencies first]
The test task depends on build. I'll run both.

[Executes:]
mise run test

[If it fails, Claude can debug:]
mise tasks info test
MISE_DEBUG=1 mise run test
```

## Skill Reference

The mise-workflow skill covers:

### Understanding Projects
- Quick inspection commands
- Systematic project analysis
- Reading mise.toml structure
- Understanding tool resolution
- Checking project health

### Running Commands
- Basic task execution
- Task dependencies and ordering
- Parallel execution
- Watching files
- Debugging task execution

### Adding Capabilities
- Tool backend selection (npm:, go:, cargo:, pipx:, ubi:, aqua:)
- Creating TOML tasks
- Creating file tasks
- Task patterns and templates
- Validation workflow

## Troubleshooting

**Plugin not loading:**
- Ensure mise is installed: `mise --version`
- Check plugin is installed: `/plugin list`
- Restart Claude Code

**MCP server fails to start:**
- Verify mise is in PATH: `which mise`
- Check experimental features: `echo $MISE_EXPERIMENTAL`
- Test MCP manually: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}' | mise mcp`

**SessionStart hook not triggering:**
- Ensure you're in a directory with `mise.toml`
- Check hook permissions: `ls -l hooks/scripts/session-start.sh`
- Verify jq is installed: `which jq`

**Skill not activating:**
- Try explicitly: "Use the mise-workflow skill to help me..."
- Check skill is loaded: Should appear in available skills
- Ensure SKILL.md has proper YAML frontmatter

**Tools or tasks not showing:**
- Ensure you're in a mise project directory
- Verify configuration: `mise config ls`
- Check tools are installed: `mise ls`
- Check tasks are defined: `mise tasks`

## Progressive Disclosure

The plugin uses an optimized progressive disclosure architecture for context efficiency:

1. **Plugin metadata** (~100 words): Always in context, describes when to use the plugin
2. **SKILL.md** (~200 lines): Self-sufficient quick reference covering all basic workflows
3. **Reference files** (advanced-only): Deep dives into edge cases and complex scenarios only
   - understanding-projects.md - Advanced config hierarchy, version specs, tool resolution
   - running-commands.md - Complex orchestration, parallelism, debugging patterns
   - adding-capabilities.md - Advanced tool options, conditional installation, monorepo patterns

**Key improvement**: Reference files now contain only advanced content. All basic operations are in SKILL.md, making references truly optional - loaded only when users encounter complex scenarios beyond standard workflows.

This architecture minimizes context usage while ensuring comprehensive coverage when needed.

## About mise

mise is a polyglot development tool version manager that replaces tools like asdf, nvm, pyenv, rbenv, etc. It manages:

- Runtime versions (Node.js, Python, Ruby, Go, Java, etc.)
- Development tools (via aqua, npm, cargo, go, pipx, etc.)
- Environment variables and configuration
- Task automation (like make, but better)

**Learn more:** https://mise.jdx.dev

## Contributing

This plugin is part of the mise project. Contributions are welcome!

**Report Issues:** https://github.com/jdx/mise/issues

**Plugin Structure:**
```
claude-plugins/mise/
├── .claude-plugin/
│   └── plugin.json          # Plugin manifest
├── commands/
│   └── detect.md            # Detect command definition
├── hooks/
│   ├── hooks.json           # Hook configuration
│   └── scripts/
│       └── session-start.sh # SessionStart hook
├── skills/
│   └── mise-workflow/
│       ├── SKILL.md         # Main skill file
│       └── references/      # Reference documentation
│           ├── understanding-projects.md
│           ├── running-commands.md
│           └── adding-capabilities.md
├── CHANGELOG.md             # Version history
└── README.md                # This file
```

## License

MIT License - See the mise project for full license details.

## Version History

**0.3.0** (2025-11-04)
- Added task arguments (usage field) support to SKILL.md
- Added Tera→usage migration guide to advanced reference
- Optimized progressive disclosure: SKILL.md reduced 28%, references now advanced-only
- ~40% reduction in essential context size while maintaining comprehensive coverage

**0.2.0** (2025-11-03)
- Added `/mise-en-place:detect` command for on-demand project detection
- Enhanced with `mise://config` MCP resource for config file hierarchy
- Improved efficiency: removed slow `mise doctor` check, limited output to key items
- Fixed MCP server name to use "mise-en-place"

**0.1.0** (2025-10-31)
- Added mise-workflow skill for active development
- Added SessionStart hook for automatic project detection
- Added comprehensive reference documentation
- Improved plugin description

**0.0.1** (2025-10-29)
- Initial release with MCP server integration
