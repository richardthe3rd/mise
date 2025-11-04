---
name: mise-workflow
description: Assist developers working in mise-enabled projects with understanding available tools/tasks, running commands, and adding new capabilities. Use when the user needs to inspect a project, execute tasks, or add tools/tasks during active development.
---

# Mise Workflow Assistant

## Overview

Three core workflows for mise projects:

1. **UNDERSTAND** - Inspect available tools and tasks
2. **RUN** - Execute tasks and commands
3. **ADD** - Add new tools and tasks

Use when the user asks about tasks/tools, wants to run commands, or needs to add capabilities.

## Workflow 1: Understanding a Project

Steps to understand a mise project:

1. **Read `mise.toml`** - Understand structure
2. `mise tasks ls --json` - List tasks
3. `mise ls --json` - List tools and versions
4. `mise doctor` - Check for issues (if needed)

### Key mise.toml Sections

```toml
[tools]           # Required tools (node = "20", python = "3.11")
[env]             # Environment variables (exported to shell)
[vars]            # Internal constants (for templates only)
[tasks.name]      # Task definitions (description + run + depends)
```

## Workflow 2: Running Tasks

### Basic Commands

```bash
mise run <task>                     # Run task
mise run test -- --verbose          # With arguments
mise run lint test build            # Multiple tasks
mise watch <task>                   # Re-run on changes
MISE_ENV=production mise run build  # Specific environment
mise exec -- <command>              # Execute with mise tools
```

### Task Dependencies

Tasks with `depends = ["other"]` run dependencies first (in parallel when possible).

### Debugging

If task fails:
1. `mise tasks info <task>` - Check definition
2. `mise exec -- <command>` - Test command directly
3. `MISE_DEBUG=1 mise run <task>` - Enable debug output
4. `mise env` - Check environment variables

## Workflow 3: Adding Tools and Tasks

### Adding Tools

Decision tree:
1. **Runtime?** (node, python, go) → `mise use node@20`
2. **npm/cargo/go package?** → `mise use npm:prettier`, `mise use cargo:ripgrep`
3. **GitHub releases?** → `mise use ubi:user/repo`

Common backends:
- `node@20` - Core runtimes (node, python, ruby, go, java)
- `npm:tool` - npm packages
- `go:github.com/user/repo/cmd/tool` - Go tools
- `cargo:tool` - Rust crates
- `pipx:tool` - Python CLI tools
- `ubi:user/repo` - GitHub releases

### Creating Tasks

**TOML (simple tasks):**
```toml
[tasks.build]
description = "Build app"  # REQUIRED
run = "npm run build"
depends = ["install"]      # Optional
```

**Tasks with arguments (usage field):**
```toml
[tasks.test]
description = "Run tests"
usage = '''
arg "<filter>" help="Test filter pattern" default=".*"
flag "-v --verbose" help="Verbose output"
flag "--watch" help="Watch mode"
'''
run = '''
npm test -- "${usage_filter?}" \
  ${usage_verbose:+--verbose} \
  ${usage_watch:+--watch}
'''
```
Run: `mise run test unit -v --watch`

Arguments become `$usage_*` variables. Use bash expansions:
- `${usage_var?}` - Required (error if unset)
- `${usage_var:-default}` - Optional with default
- `${usage_var:+value}` - Only if set (for flags)

**File task (complex scripts):**
```bash
#!/usr/bin/env bash
#MISE description="Deploy app"
#USAGE arg "<env>" help="Environment" {
#USAGE   choices "dev" "staging" "prod"
#USAGE }
#USAGE flag "--dry-run" help="Preview changes"
set -euo pipefail
./deploy.sh "${usage_env?}" "${usage_dry_run:-false}"
```

### Validation

After adding tools/tasks:
1. `mise fmt` - Format config
2. `mise doctor` - Check for issues
3. `mise run <task>` - Test execution

## Best Practices

1. Always add task descriptions (required)
2. Run `mise fmt` after editing mise.toml
3. Use appropriate backends (npm:, go:, cargo:, etc.)
4. Model dependencies with `depends`

## Troubleshooting

- Task not found → `mise tasks` to list
- Tool not found → `mise install` to install all
- Environment issues → `mise env` to check
- Changes not applying → `mise fmt` and restart shell

## Advanced Topics

For complex scenarios, see reference files:
- **understanding-projects.md** - Config hierarchy, version specs, tool resolution
- **running-commands.md** - Parallel execution, wildcards, orchestration
- **adding-capabilities.md** - Advanced tool options, monorepo patterns, Tera→usage migration
