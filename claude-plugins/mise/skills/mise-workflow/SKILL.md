---
name: mise-workflow
description: Assist developers working in mise-enabled projects with understanding available tools/tasks, running commands, and adding new capabilities. Use when the user needs to inspect a project, execute tasks, or add tools/tasks during active development.
---

# Mise Workflow Assistant

## Overview

This skill helps developers work efficiently with mise in their projects through three core workflows:

1. **UNDERSTAND** - Inspect what tools and tasks are available in a project
2. **RUN** - Execute tasks and commands using mise-managed tools
3. **ADD** - Add new tools and tasks to a project

This skill focuses on active development workflows, not initial mise setup or conceptual learning.

## When to Use This Skill

Use this skill when the user:
- Asks about available tasks or tools in a project
- Wants to execute a task or command
- Wants to add new capabilities (tools or tasks) to a project
- Is actively developing in a mise-enabled project
- Asks "what tasks are available?" or "what tools does this use?"
- Says "run the tests" or "execute the build"
- Says "add a task to..." or "add python to this project"

## Workflow 1: Understanding a Project

### Quick Inspection

When a user asks about a mise project, start with these commands:

```bash
# See all available tasks with descriptions
mise tasks ls --json

# See all tools and their versions
mise ls --json

# Check for any issues
mise doctor

# Show environment variables
mise env

# Show configuration files
mise config ls
```

### Systematic Project Analysis

For a comprehensive understanding of a mise project:

1. **Read the configuration**: Start by reading `mise.toml` (or `.mise.toml`) to understand project structure
2. **Check tools**: Run `mise ls --json` to see what tools are installed and active
3. **List tasks**: Run `mise tasks ls --json` to see available tasks and their descriptions
4. **Check health**: Run `mise doctor` to identify any configuration issues
5. **Understand environment**: Run `mise env` to see what environment variables are set

### Reading mise.toml

The mise.toml file has several sections:

```toml
[tools]
# Development tools needed (node, python, go, etc.)
node = "20"
python = "3.11"
"go:github.com/golangci/golangci-lint/cmd/golangci-lint" = "latest"

[env]
# Environment variables (exported to shell)
NODE_ENV = "development"
API_URL = "http://localhost:3000"

[vars]
# Internal constants (not exported, used in templates)
build_dir = "dist"
app_name = "myapp"

[tasks.build]
# Task definition
description = "Build the application"
run = "npm run build"
depends = ["install"]
```

### Common Questions

**"What tasks are available?"**
- Run `mise tasks ls --json` for programmatic access
- Or run `mise tasks` for human-readable output
- Look for the `description` field in each task

**"What tools does this project use?"**
- Check the `[tools]` section in mise.toml
- Run `mise ls` to see installed versions
- Look for backends: plain (node, python), prefixed (npm:, go:, cargo:, pipx:)

**"Are there any issues?"**
- Run `mise doctor` to check for problems
- Common issues: missing tools, outdated versions, configuration errors

**"What's the build process?"**
- Look for tasks named `build`, `compile`, `bundle`
- Check `depends` field to see task dependencies
- Trace the dependency tree with task definitions

## Workflow 2: Running Tasks and Tools

### Running Tasks

**Basic execution:**
```bash
# Run a single task
mise run <task-name>

# Run with arguments
mise run test -- --verbose

# Run multiple tasks
mise run lint test build
```

**Development workflows:**
```bash
# Watch files and re-run task on changes
mise watch <task-name>

# Run with specific environment
MISE_ENV=production mise run build
```

### Task Dependencies

Tasks can depend on other tasks via the `depends` field:

```toml
[tasks.build]
description = "Build the application"
depends = ["install", "lint"]  # Runs install and lint first
run = "npm run build"
```

When running `mise run build`, mise will:
1. Check the dependency graph
2. Run `install` and `lint` first (in parallel if possible)
3. Then run `build` after dependencies complete

### Using Tools Directly

Execute commands using mise-managed tools:

```bash
# Execute a command with mise environment
mise exec -- node --version

# Shorter version (mise x)
mise x -- python script.py

# Execute in a specific directory
mise --cd /path/to/project exec -- go build
```

### Common Execution Patterns

**Run tests:**
```bash
mise run test              # Run test task if defined
mise watch test            # Re-run tests on file changes
```

**Build and test:**
```bash
mise run build test        # Run build, then test
```

**Development server:**
```bash
mise run dev               # Start dev server
mise watch build           # Rebuild on changes
```

### Debugging Task Execution

If a task fails:

1. **Check the task definition** - Is the command correct?
2. **Verify dependencies** - Are dependent tasks defined and working?
3. **Check environment** - Run `mise env` to see variables
4. **Test the command directly** - Run the command with `mise exec --`
5. **Enable debug output** - Set `MISE_DEBUG=1` or `MISE_TRACE=1`

Example debug session:
```bash
# Enable debug output
MISE_DEBUG=1 mise run build

# Check if tools are available
mise ls

# Test command directly
mise exec -- npm run build

# Verify environment variables
mise env | grep NODE
```

## Workflow 3: Adding Tools and Tasks

### Adding Tools

**Decision tree for adding tools:**

1. **Runtime tool** (node, python, ruby, go, java)?
   → Add directly: `mise use node@20`

2. **CLI tool installed via language package manager?**
   - Go: `mise use go:github.com/user/tool/cmd/tool`
   - npm: `mise use npm:prettier`
   - Cargo: `mise use cargo:ripgrep`
   - pipx: `mise use pipx:black`

3. **CLI tool from GitHub releases?**
   → Add via ubi: `mise use ubi:github-user/repo-name`

4. **Tool from aqua registry?**
   → Add via aqua: `mise use aqua:user/repo`

**Examples:**

```bash
# Add Node.js 20
mise use node@20

# Add golangci-lint (Go tool)
mise use go:github.com/golangci/golangci-lint/cmd/golangci-lint

# Add TypeScript (npm tool)
mise use npm:typescript

# Add prettier (npm tool, pinned version)
mise use npm:prettier@3.0.0

# Add ripgrep (Cargo tool)
mise use cargo:ripgrep

# Add black Python formatter (pipx)
mise use pipx:black

# Add tool from GitHub releases
mise use ubi:BurntSushi/ripgrep
```

### Creating Tasks

**TOML vs File Tasks:**

Use **TOML tasks** when:
- Task is simple (< 10 lines)
- No complex control flow needed
- Single command or simple shell script
- Configuration stays in mise.toml

Use **File tasks** when:
- Task is complex (loops, conditionals, functions)
- Multi-language scripts (Python, Ruby, etc.)
- Need syntax highlighting and IDE support
- Easier to test and maintain as separate files

**TOML Task Template:**

```toml
[tasks.name]
description = "Clear description of what this does"  # REQUIRED!
run = "command to execute"
depends = ["other-task"]          # Optional: dependencies
sources = ["src/**/*.ts"]         # Optional: watch these files
outputs = ["dist/**/*.js"]        # Optional: output files
dir = "{{config_root}}/subdir"   # Optional: working directory
env = { DEBUG = "1" }             # Optional: environment variables
```

**File Task Template:**

Create `mise-tasks/name` (executable file):

```bash
#!/usr/bin/env bash
#MISE description="Clear description of what this does"
#MISE depends=["other-task"]
#MISE sources=["src/**/*.ts"]
#MISE outputs=["dist/**/*.js"]

# Your script here
set -euo pipefail

echo "Running task..."
# ... commands ...
```

Make it executable:
```bash
chmod +x mise-tasks/name
```

### Common Task Patterns

**Build task:**
```toml
[tasks.build]
description = "Build the application"
depends = ["install"]
run = "npm run build"
sources = ["src/**/*.ts", "package.json"]
outputs = ["dist/**/*.js"]
```

**Test task:**
```toml
[tasks.test]
description = "Run all tests"
depends = ["build"]
run = "npm test"
```

**Lint task:**
```toml
[tasks.lint]
description = "Run linters"
run = [
    "eslint src/",
    "prettier --check src/",
]
```

**Pre-commit task:**
```toml
[tasks.pre-commit]
description = "Run before git commit"
run = "mise run lint test"
```

**Watch task:**
```toml
[tasks.dev]
description = "Start development server with auto-reload"
run = "npm run dev"
```

Then use: `mise watch dev` or `mise run dev` if it has built-in watching.

### Validation Workflow

After adding tools or tasks, ALWAYS follow this workflow:

```bash
# 1. Format the configuration
mise fmt

# 2. Check for issues
mise doctor

# 3. Verify tasks have descriptions
mise tasks ls --json | jq '.[] | select(.description == null or .description == "")'

# 4. Test execution
mise run <new-task-name>

# 5. Verify tools are installed
mise ls
```

### Example: Adding a Linting Setup

User asks: "Add prettier and eslint to this project"

Steps:
1. **Add tools**: `mise use npm:prettier npm:eslint`
2. **Create task** in mise.toml:
   ```toml
   [tasks.lint]
   description = "Run prettier and eslint"
   run = [
       "prettier --check .",
       "eslint src/",
   ]

   [tasks.lint-fix]
   description = "Fix linting issues automatically"
   run = [
       "prettier --write .",
       "eslint --fix src/",
   ]
   ```
3. **Validate**: `mise fmt && mise doctor`
4. **Test**: `mise run lint`
5. **Verify**: `mise tasks ls --json | jq '.[] | select(.name == "lint")'`

## Progressive Disclosure

For detailed guidance, refer to the following references:

- **references/understanding-projects.md** - Deep dive into project inspection, configuration hierarchy, tool resolution
- **references/running-commands.md** - Complete guide to task execution, `mise run`, `mise exec`, `mise watch`, parallel execution
- **references/adding-capabilities.md** - Comprehensive tool and task addition guide with decision trees and examples

## Best Practices

1. **Always add task descriptions** - Every task MUST have a description
2. **Format after editing** - Run `mise fmt` after modifying mise.toml
3. **Validate before committing** - Run `mise doctor` to check for issues
4. **Test tasks** - Run tasks after creating them to ensure they work
5. **Use appropriate backends** - Match tool source to backend (npm:, go:, cargo:, etc.)
6. **Leverage dependencies** - Use `depends` to model task relationships
7. **Keep tasks focused** - One task, one purpose; compose with dependencies

## Common Patterns

### Monorepo Setup
```toml
# Root mise.toml
[tasks.test-all]
description = "Run tests in all packages"
run = [
    "mise --cd packages/pkg1 run test",
    "mise --cd packages/pkg2 run test",
]
```

### Multi-environment Configuration
```toml
[env]
_.file = ".env"  # Load from .env file
API_URL = "http://localhost:3000"

[env.production]
API_URL = "https://api.production.com"

[env.staging]
API_URL = "https://api.staging.com"
```

Use: `MISE_ENV=production mise run deploy`

### Tool Version Management
```toml
[tools]
node = "20"          # Use exact major version
python = "3.11.5"    # Use exact version
go = "latest"        # Always use latest
terraform = "~1.5"   # Use latest 1.5.x
```

## Troubleshooting

**Task not found:**
- Run `mise tasks` to list available tasks
- Check task name spelling
- Verify task is defined in mise.toml or mise-tasks/

**Tool not found:**
- Run `mise ls` to see installed tools
- Run `mise install` to install all tools
- Check `[tools]` section in mise.toml

**Task fails with environment error:**
- Run `mise env` to check environment variables
- Verify `[env]` section in mise.toml
- Check that tools are installed: `mise ls`

**Changes not taking effect:**
- Run `mise fmt` to format configuration
- Restart shell or run `mise activate` again
- Check for syntax errors with `mise doctor`
