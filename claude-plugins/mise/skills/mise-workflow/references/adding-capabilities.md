# Adding Capabilities to Mise Projects

Comprehensive guide to adding tools and tasks to mise projects.

## Table of Contents

1. [Adding Tools](#adding-tools)
2. [Tool Backend Selection](#tool-backend-selection)
3. [Creating Tasks](#creating-tasks)
4. [TOML vs File Tasks](#toml-vs-file-tasks)
5. [Task Patterns and Templates](#task-patterns-and-templates)
6. [Validation Workflow](#validation-workflow)

## Adding Tools

### Quick Commands

```bash
# Add runtime tools
mise use node@20
mise use python@3.11
mise use go@latest

# Add CLI tools (language backends)
mise use npm:prettier
mise use go:github.com/golangci/golangci-lint/cmd/golangci-lint
mise use cargo:ripgrep
mise use pipx:black

# Add from GitHub releases
mise use ubi:BurntSushi/ripgrep

# Add to global config
mise use -g node@20

# Add and install immediately
mise use node@20 && mise install
```

### Understanding Tool Types

**Runtime Tools:**
- Interpreters and compilers (node, python, go, ruby, java, rust)
- Installed via mise's core backends
- Manage multiple versions
- Examples: `node`, `python`, `go`, `ruby`

**CLI Tools:**
- Command-line utilities (linters, formatters, build tools)
- Installed via language package managers or registries
- Project-specific or global
- Examples: `prettier`, `eslint`, `golangci-lint`, `rg`

**The Key Question:** Is this a runtime (version manager) or a CLI tool?

## Tool Backend Selection

### Decision Tree

```
Is it a runtime/interpreter (node, python, go, ruby, java)?
├─ YES → Use core backend
│  └─ Examples: mise use node@20, mise use python@3.11
│
└─ NO → Is it a CLI tool?
   ├─ Installed via `npm install -g`?
   │  └─ YES → Use npm: backend
   │     └─ Example: mise use npm:prettier
   │
   ├─ Installed via `go install`?
   │  └─ YES → Use go: backend
   │     └─ Example: mise use go:github.com/user/tool/cmd/tool
   │
   ├─ Installed via `cargo install`?
   │  └─ YES → Use cargo: backend
   │     └─ Example: mise use cargo:ripgrep
   │
   ├─ Installed via `pipx install`?
   │  └─ YES → Use pipx: backend
   │     └─ Example: mise use pipx:black
   │
   ├─ Available on GitHub releases?
   │  └─ YES → Use ubi: backend
   │     └─ Example: mise use ubi:BurntSushi/ripgrep
   │
   └─ In aqua registry?
      └─ YES → Use aqua: backend
         └─ Example: mise use aqua:cli/cli
```

### Backend Reference

#### Core Backends (Runtime Tools)

**Supported runtimes:**
- `node` - Node.js
- `python` - Python
- `ruby` - Ruby
- `go` - Go
- `java` - Java
- `rust` - Rust
- `deno` - Deno
- `bun` - Bun
- `elixir` - Elixir
- `erlang` - Erlang
- Plus many more...

**Usage:**
```bash
mise use <tool>@<version>
```

**Version formats:**
```bash
mise use node@20           # Latest 20.x.x
mise use node@20.11        # Latest 20.11.x
mise use node@20.11.0      # Exact version
mise use node@latest       # Latest stable
mise use node@~20.11       # Latest 20.11.x (semver tilde)
mise use node@^20.11       # Latest 20.x where x >= 11 (semver caret)
```

**Configuration:**
```toml
[tools]
node = "20"
python = "3.11.5"
go = "latest"
```

#### npm: Backend (Node.js CLI Tools)

**When to use:**
- Tool is published to npmjs.org
- Normally installed with `npm install -g <package>`
- Examples: prettier, eslint, typescript, webpack-cli

**Requirements:**
- Node.js must be installed (via mise or system)

**Usage:**
```bash
# Add npm tool
mise use npm:prettier
mise use npm:eslint
mise use npm:typescript@5.0.0

# With version
mise use npm:prettier@3.0.0
```

**Configuration:**
```toml
[tools]
node = "20"  # npm requires node
"npm:prettier" = "latest"
"npm:eslint" = "8.50.0"
"npm:typescript" = "~5.0"
```

**Common npm tools:**
- `npm:prettier` - Code formatter
- `npm:eslint` - JavaScript linter
- `npm:typescript` - TypeScript compiler
- `npm:webpack-cli` - Webpack CLI
- `npm:@angular/cli` - Angular CLI
- `npm:create-react-app` - React app generator
- `npm:nodemon` - Node.js monitor
- `npm:pm2` - Process manager

#### go: Backend (Go CLI Tools)

**When to use:**
- Tool is installed via `go install`
- Tool has a GitHub repository with Go code
- Examples: golangci-lint, goreleaser, staticcheck

**Requirements:**
- Go must be installed (via mise or system)

**Usage:**
```bash
# Full import path
mise use go:github.com/golangci/golangci-lint/cmd/golangci-lint

# With version
mise use go:github.com/golangci/golangci-lint/cmd/golangci-lint@v1.54.2

# With build tags
# (configure in mise.toml)
```

**Configuration:**
```toml
[tools]
go = "latest"  # go backend requires go
"go:github.com/golangci/golangci-lint/cmd/golangci-lint" = "latest"
"go:github.com/goreleaser/goreleaser" = "v1.20.0"

# With build tags
"go:github.com/golang-migrate/migrate/v4/cmd/migrate" = { version = "latest", tags = "postgres" }
```

**Common go tools:**
- `go:github.com/golangci/golangci-lint/cmd/golangci-lint` - Go linter
- `go:github.com/goreleaser/goreleaser` - Release automation
- `go:golang.org/x/tools/cmd/goimports` - Import formatter
- `go:github.com/cosmtrek/air` - Live reload
- `go:github.com/swaggo/swag/cmd/swag` - Swagger generator
- `go:gotest.tools/gotestsum` - Test runner
- `go:github.com/sqlc-dev/sqlc/cmd/sqlc` - SQL compiler

#### cargo: Backend (Rust CLI Tools)

**When to use:**
- Tool is published to crates.io
- Normally installed with `cargo install <crate>`
- Examples: ripgrep, fd, bat, cargo-watch

**Requirements:**
- Rust (cargo) must be installed (via mise or system)

**Usage:**
```bash
# Add cargo tool
mise use cargo:ripgrep
mise use cargo:fd-find
mise use cargo:bat

# With version
mise use cargo:ripgrep@14.0.0
```

**Configuration:**
```toml
[tools]
rust = "latest"  # cargo backend requires rust
"cargo:ripgrep" = "latest"
"cargo:fd-find" = "8.7.0"
"cargo:bat" = "latest"
```

**Common cargo tools:**
- `cargo:ripgrep` - Fast grep alternative (rg)
- `cargo:fd-find` - Fast find alternative (fd)
- `cargo:bat` - Cat clone with syntax highlighting
- `cargo:exa` - Modern ls alternative
- `cargo:tokei` - Code statistics
- `cargo:hyperfine` - Benchmarking tool
- `cargo:cargo-watch` - Watch and rebuild
- `cargo:cargo-edit` - Cargo command extensions

#### pipx: Backend (Python CLI Tools)

**When to use:**
- Python CLI tool/application
- Normally installed with `pipx install <package>`
- Should run in isolated environment
- Examples: black, flake8, poetry, aws-cli

**Requirements:**
- Python must be installed (via mise or system)
- pipx must be installed (mise can install it)

**Usage:**
```bash
# Add pipx tool
mise use pipx:black
mise use pipx:flake8
mise use pipx:poetry

# With version
mise use pipx:black@23.0.0
```

**Configuration:**
```toml
[tools]
python = "3.11"  # pipx requires python
"pipx:black" = "latest"
"pipx:flake8" = "6.1.0"
"pipx:poetry" = "latest"
"pipx:aws-cli" = "latest"
```

**Common pipx tools:**
- `pipx:black` - Python code formatter
- `pipx:flake8` - Python linter
- `pipx:mypy` - Static type checker
- `pipx:poetry` - Python dependency manager
- `pipx:pipenv` - Python virtualenv manager
- `pipx:cookiecutter` - Project template tool
- `pipx:aws-cli` - AWS command line

#### ubi: Backend (GitHub Releases)

**When to use:**
- Tool releases binaries on GitHub Releases
- No package manager available
- Examples: GitHub CLI, Kubernetes tools, HashiCorp tools

**Usage:**
```bash
# GitHub repo format: owner/repo
mise use ubi:BurntSushi/ripgrep
mise use ubi:cli/cli
mise use ubi:junegunn/fzf

# With version
mise use ubi:cli/cli@v2.35.0
```

**Configuration:**
```toml
[tools]
"ubi:cli/cli" = "latest"           # GitHub CLI
"ubi:junegunn/fzf" = "latest"      # Fuzzy finder
"ubi:BurntSushi/ripgrep" = "14.0.0"
```

**Common ubi tools:**
- `ubi:cli/cli` - GitHub CLI (gh)
- `ubi:junegunn/fzf` - Fuzzy finder
- `ubi:sharkdp/fd` - Find alternative
- `ubi:sharkdp/bat` - Cat alternative
- `ubi:dandavison/delta` - Git diff viewer
- `ubi:starship/starship` - Cross-shell prompt

#### aqua: Backend (Aqua Registry)

**When to use:**
- Tool is in the aqua registry
- Want access to 20,000+ packages
- Prefer registry-based management

**Usage:**
```bash
mise use aqua:cli/cli
mise use aqua:golangci/golangci-lint
```

**Configuration:**
```toml
[tools]
"aqua:cli/cli" = "latest"
"aqua:golangci/golangci-lint" = "v1.54.2"
```

**Search aqua registry:**
Visit: https://aquaproj.github.io/

### Version Specification

**Common patterns:**

```toml
[tools]
# Exact version
node = "20.11.0"

# Major version (latest patch)
node = "20"

# Major.minor (latest patch)
node = "20.11"

# Latest stable
node = "latest"

# Semver tilde (~) - latest patch version
node = "~20.11"     # Matches 20.11.x

# Semver caret (^) - latest minor version
node = "^20.11"     # Matches 20.x where x >= 11

# Version prefix
node = "prefix:20"  # Finds latest version starting with 20

# Path to local build
node = "path:~/.local/share/node/custom"

# System version
node = "system"     # Use system-installed version
```

### Tool Options

Some backends support additional options:

```toml
[tools]
# Go backend: build tags
"go:github.com/golang-migrate/migrate/v4/cmd/migrate" = {
    version = "latest",
    tags = "postgres,mysql"
}

# Custom tool options
python = { version = "3.11", virtualenv = ".venv" }
```

## Creating Tasks

### Quick Task Creation

**Via CLI:**
```bash
# Simple task
mise tasks add build npm run build

# Task with dependencies
mise tasks add test --depends build -- npm test

# Task with description
mise tasks add lint --description "Run linting" -- npm run lint

# Task with multiple commands
mise tasks add ci --depends "lint" --depends "test" -- mise run build
```

**Via mise.toml:**
```bash
# Edit mise.toml directly
$EDITOR mise.toml
```

### Task Creation Methods

**Method 1: Inline TOML (Simple)**
```toml
[tasks.build]
description = "Build the application"
run = "npm run build"
```

**Method 2: Detailed TOML (Complex)**
```toml
[tasks.build]
description = "Build the application"
depends = ["install"]
run = ["npm run build", "npm run bundle"]
sources = ["src/**/*.ts"]
outputs = ["dist/**/*.js"]
env = { NODE_ENV = "production" }
```

**Method 3: File Task (Very Complex)**
```bash
# Create mise-tasks/build
#!/usr/bin/env bash
#MISE description="Build the application"
#MISE depends=["install"]

set -euo pipefail

echo "Building..."
npm run build
npm run bundle
echo "Build complete!"
```

## TOML vs File Tasks

### Decision Matrix

| Factor | TOML Tasks | File Tasks |
|--------|------------|------------|
| **Complexity** | Simple (< 10 lines) | Complex (loops, conditionals) |
| **Language** | Shell only | Any language (Python, Ruby, etc.) |
| **IDE Support** | Limited | Full (syntax highlighting, linting) |
| **Testing** | Difficult | Easy (can run directly) |
| **Configuration** | In mise.toml | In #MISE comments |
| **Visibility** | All in one file | Separate files |
| **Reusability** | Template functions | Full scripting |

### When to Use TOML Tasks

✅ Use TOML tasks when:
- Task is a single command or simple script
- No complex control flow needed
- Want all configuration in one file
- Task is short (< 10 lines)
- Simple environment variable usage

Examples:
```toml
# Single command
[tasks.test]
description = "Run tests"
run = "npm test"

# Multiple simple commands
[tasks.lint]
description = "Run linters"
run = [
    "eslint src/",
    "prettier --check src/",
]

# With templates
[tasks.build]
description = "Build application"
run = "go build -o {{vars.output_dir}}/{{vars.app_name}}"
```

### When to Use File Tasks

✅ Use file tasks when:
- Task has complex logic (loops, conditionals, functions)
- Need multi-line shell scripts
- Want to use non-shell languages (Python, Ruby, etc.)
- Need IDE support (syntax highlighting, linting)
- Task is easier to test as a standalone script
- Task is > 10 lines

Examples:

**Complex Shell Script:**
```bash
#!/usr/bin/env bash
#MISE description="Deploy application"
#MISE depends=["build", "test"]

set -euo pipefail

# Complex logic
if [ "$MISE_ENV" = "production" ]; then
    echo "Deploying to production..."
    ./scripts/deploy-prod.sh
elif [ "$MISE_ENV" = "staging" ]; then
    echo "Deploying to staging..."
    ./scripts/deploy-staging.sh
else
    echo "Unknown environment: $MISE_ENV"
    exit 1
fi

# Loop through services
for service in api web worker; do
    echo "Restarting $service..."
    systemctl restart "myapp-$service"
done
```

**Python Script:**
```python
#!/usr/bin/env python3
#MISE description="Process data files"
#MISE sources=["data/*.csv"]
#MISE outputs=["processed/*.json"]

import sys
import glob
import json

# Complex data processing
for file in glob.glob("data/*.csv"):
    with open(file) as f:
        # Process CSV...
        data = process_csv(f.read())

    output = file.replace("data/", "processed/").replace(".csv", ".json")
    with open(output, "w") as f:
        json.dump(data, f)
```

### File Task Metadata

All configuration for file tasks goes in `#MISE` comments:

```bash
#!/usr/bin/env bash
#MISE description="Task description (REQUIRED)"
#MISE depends=["dep1", "dep2"]
#MISE depends_post=["cleanup"]
#MISE sources=["src/**/*.ts"]
#MISE outputs=["dist/**/*.js"]
#MISE hide=false
#MISE raw=false
#MISE quiet=false
#MISE env={DEBUG="1", VERBOSE="true"}

# Your script here
```

**Make executable:**
```bash
chmod +x mise-tasks/taskname
```

## Task Patterns and Templates

### Common Task Patterns

#### Build Task

```toml
[tasks.build]
description = "Build the application"
depends = ["install"]
sources = ["src/**/*", "package.json"]
outputs = ["dist/**/*"]
run = "npm run build"
```

#### Test Task

```toml
[tasks.test]
description = "Run all tests"
depends = ["build"]
run = "npm test"

[tasks."test:unit"]
description = "Run unit tests"
run = "npm run test:unit"

[tasks."test:integration"]
description = "Run integration tests"
depends = ["docker:up"]
run = "npm run test:integration"
depends_post = ["docker:down"]

[tasks."test:e2e"]
description = "Run end-to-end tests"
depends = ["build", "docker:up"]
run = "npm run test:e2e"
depends_post = ["docker:down"]
```

#### Lint Task

```toml
[tasks.lint]
description = "Run all linters"
depends = ["lint:*"]

[tasks."lint:eslint"]
description = "Lint JavaScript/TypeScript"
run = "eslint src/"

[tasks."lint:prettier"]
description = "Check code formatting"
run = "prettier --check src/"

[tasks."lint:types"]
description = "Check TypeScript types"
run = "tsc --noEmit"

[tasks.lint-fix]
description = "Fix linting issues"
run = [
    "eslint --fix src/",
    "prettier --write src/",
]
```

#### Watch Task

```toml
[tasks.dev]
description = "Start development server"
run = "npm run dev"

[tasks.watch]
description = "Watch and rebuild on changes"
run = "npm run build"
sources = ["src/**/*"]
# Then use: mise watch watch
```

#### Docker Tasks

```toml
[tasks."docker:build"]
description = "Build Docker image"
run = "docker build -t myapp:{{env.VERSION}} ."

[tasks."docker:push"]
description = "Push Docker image"
depends = ["docker:build"]
run = "docker push myapp:{{env.VERSION}}"

[tasks."docker:up"]
description = "Start Docker services"
run = "docker-compose up -d"

[tasks."docker:down"]
description = "Stop Docker services"
run = "docker-compose down"
```

#### CI/CD Task

```toml
[tasks.ci]
description = "Run CI pipeline"
depends = ["lint", "test", "build"]

[tasks.deploy]
description = "Deploy application"
depends = ["ci"]
run = "./scripts/deploy.sh"
confirm = "Are you sure you want to deploy?"
```

#### Pre-commit Task

```toml
[tasks.pre-commit]
description = "Run before committing"
depends = ["lint-fix", "test:unit"]
run = "git add -u"
```

#### Clean Task

```toml
[tasks.clean]
description = "Clean build artifacts"
run = [
    "rm -rf dist/",
    "rm -rf node_modules/.cache/",
    "rm -rf .mise/cache/",
]

[tasks.clean-all]
description = "Clean everything including dependencies"
depends = ["clean"]
run = "rm -rf node_modules/"
```

### Template Variables

Use templates in task definitions:

```toml
[vars]
app_name = "myapp"
version = "1.0.0"
build_dir = "dist"

[env]
API_URL = "http://localhost:3000"
VERSION = "{{vars.version}}"

[tasks.build]
description = "Build {{vars.app_name}}"
run = "npm run build -- --out {{vars.build_dir}}"

[tasks.version]
description = "Show version"
run = "echo 'Version: {{vars.version}}'"

[tasks.deploy]
description = "Deploy to {{env.API_URL}}"
run = "./deploy.sh {{env.API_URL}}"
```

**Available template functions:**

```toml
# Execute command
VERSION = "{{exec(command='git describe --tags')}}"

# Config root directory
BUILD_PATH = "{{config_root}}/dist"

# Current working directory
CWD = "{{cwd}}"

# Environment variable
HOME_DIR = "{{env.HOME}}"

# Vars
OUTPUT = "{{vars.build_dir}}"

# Conditionals
ENV_TYPE = "{{if(cond=env.PROD, t='production', f='development')}}"

# OS/Arch
PLATFORM = "{{os}}"  # linux, macos, windows
ARCH = "{{arch}}"    # x86_64, arm64
```

### Multi-Environment Tasks

```toml
[env]
API_URL = "http://localhost:3000"

[env.staging]
API_URL = "https://api.staging.example.com"

[env.production]
API_URL = "https://api.example.com"

[tasks.deploy]
description = "Deploy to {{env.API_URL}}"
run = "./deploy.sh"
```

Usage:
```bash
# Deploy to staging
MISE_ENV=staging mise run deploy

# Deploy to production
MISE_ENV=production mise run deploy
```

## Validation Workflow

After adding tools or tasks, ALWAYS follow this workflow:

### Step 1: Format Configuration

```bash
mise fmt
```

This formats `mise.toml` to standard style:
- Sorts sections
- Aligns formatting
- Fixes basic syntax issues

### Step 2: Check for Issues

```bash
mise doctor
```

Checks for:
- Missing tools
- Configuration errors
- Path issues
- Plugin problems

### Step 3: Verify Task Descriptions

**All tasks MUST have descriptions.** Verify:

```bash
mise tasks ls --json | jq '.[] | select(.description == null or .description == "")'
```

If any tasks appear, add descriptions to them.

### Step 4: Test Tool Installation

```bash
# Check if tools are installed
mise ls --missing

# Install any missing tools
mise install

# Verify tools work
mise exec -- <tool> --version
```

### Step 5: Test Task Execution

```bash
# Test the new task
mise run <task-name>

# Test with arguments
mise run <task-name> -- --arg value

# Test dependencies
mise tasks deps <task-name>

# Enable debug output if issues
MISE_DEBUG=1 mise run <task-name>
```

### Step 6: Verify Task Configuration

```bash
# Show task details
mise tasks info <task-name>

# Show as JSON
mise tasks info <task-name> --json

# Verify in task list
mise tasks | grep <task-name>
```

### Complete Validation Script

```bash
#!/usr/bin/env bash
# validate-mise.sh

set -euo pipefail

echo "==> Formatting configuration..."
mise fmt

echo "==> Checking health..."
mise doctor

echo "==> Checking for tasks without descriptions..."
MISSING_DESC=$(mise tasks ls --json | jq -r '.[] | select(.description == null or .description == "") | .name')
if [ -n "$MISSING_DESC" ]; then
    echo "ERROR: Tasks without descriptions:"
    echo "$MISSING_DESC"
    exit 1
fi

echo "==> Checking for missing tools..."
MISSING_TOOLS=$(mise ls --missing 2>&1 || true)
if [ -n "$MISSING_TOOLS" ]; then
    echo "WARNING: Missing tools:"
    echo "$MISSING_TOOLS"
    echo "Run: mise install"
fi

echo "==> Testing tasks..."
for task in $(mise tasks ls --json | jq -r '.[].name'); do
    echo "  Testing $task..."
    mise tasks info "$task" > /dev/null || echo "    ERROR: Failed to get info"
done

echo "✅ Validation complete!"
```

## Best Practices Summary

### Tools
1. **Choose the right backend** - Match tool source to backend
2. **Specify versions explicitly** - Avoid `latest` in production
3. **Document tool choices** - Comment why each tool is needed
4. **Test tools after adding** - Verify with `mise exec -- tool --version`
5. **Group related tools** - Keep runtime and CLI tools organized

### Tasks
1. **Always add descriptions** - REQUIRED for all tasks
2. **Use semantic names** - Namespace with `:` for organization
3. **Model dependencies** - Use `depends` to show relationships
4. **Keep tasks focused** - One task, one purpose
5. **Use TOML for simple, files for complex** - Follow decision matrix
6. **Test tasks after creation** - Run them to ensure they work
7. **Format and validate** - Run `mise fmt && mise doctor`

### Configuration
1. **Format consistently** - Always run `mise fmt`
2. **Validate before committing** - Run `mise doctor`
3. **Use templates for DRY** - Leverage `[vars]` and templates
4. **Environment-specific config** - Use `[env.<name>]` sections
5. **Comment complex setups** - Explain non-obvious choices
