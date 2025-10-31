# Understanding Mise Projects

Complete guide to inspecting and understanding projects that use mise.

## Quick Inspection Commands

```bash
# See active tools and versions
mise ls

# See all tasks with descriptions
mise tasks

# Check project health
mise doctor

# Show environment variables
mise env

# Show configuration files
mise config ls

# Get JSON output for programmatic access
mise ls --json
mise tasks ls --json
mise config ls --json
```

## Systematic Project Analysis

When encountering a new mise project, follow this process:

### Step 1: Locate Configuration

Find the mise configuration file:

```bash
# List all mise config files in hierarchy
mise config ls

# Common locations:
# - ./mise.toml or ./.mise.toml (project-local)
# - ./config/mise.toml (project config directory)
# - ~/.config/mise/config.toml (global)
# - /etc/mise/config.toml (system-wide)
```

Read the configuration:

```bash
# Read main config
cat mise.toml

# Or if it's .mise.toml
cat .mise.toml
```

### Step 2: Understand Tools

Check what development tools the project needs:

```bash
# Show all tools (active and inactive)
mise ls

# Show only tools defined in current directory
mise ls --current

# Show only installed tools
mise ls --installed

# Show missing tools
mise ls --missing

# JSON output for full details
mise ls --json
```

Output format:

```
Tool     Version    Source                           Requested
node     20.11.0    ~/project/mise.toml              20
python   3.11.7     ~/.config/mise/config.toml       3.11
go       1.21.5     ~/project/mise.toml              latest
```

Columns explained:
- **Tool**: Tool name (e.g., node, python, go)
- **Version**: Installed version
- **Source**: Config file that specified it
- **Requested**: Version specification from config

### Step 3: Discover Tasks

See what tasks are available:

```bash
# List all tasks
mise tasks

# List with full details
mise tasks ls --json

# Show hidden tasks too
mise tasks --hidden

# Get info about specific task
mise tasks info <task-name>

# Show task dependencies
mise tasks deps <task-name>
```

### Step 4: Check Health

Verify the mise setup:

```bash
# Comprehensive health check
mise doctor

# Check specific aspects
mise doctor path  # Check PATH configuration
```

Common warnings:
- Missing tools
- Outdated versions
- Configuration issues
- Shell integration problems

### Step 5: Inspect Environment

See what environment variables mise sets:

```bash
# Show all environment variables
mise env

# Show in shell-compatible format
mise env -s bash
mise env -s fish
mise env -s zsh

# Show only PATH
mise doctor path
```

## Reading mise.toml

### Configuration Structure

A mise.toml file has multiple sections:

```toml
# Minimum mise version required
min_version = '2024.1.0'

# Development tools
[tools]
node = "20"
python = "3.11"
"go:github.com/golangci/golangci-lint/cmd/golangci-lint" = "latest"

# Internal constants (not exported to shell)
[vars]
app_name = "myapp"
version = "1.0.0"
build_dir = "dist"

# Environment variables (exported to shell)
[env]
NODE_ENV = "development"
API_URL = "http://localhost:3000"
APP_NAME = "{{vars.app_name}}"
VERSION = "{{exec(command='git describe --tags')}}"

# Path additions
[[env.PATH]]
value = "{{config_root}}/bin"
prefix = true

# Environment-specific overrides
[env.production]
NODE_ENV = "production"
API_URL = "https://api.example.com"

# Tasks
[tasks.build]
description = "Build the application"
depends = ["install"]
run = "npm run build"
sources = ["src/**/*.ts"]
outputs = ["dist/**/*.js"]

[tasks.test]
description = "Run tests"
run = "npm test"
```

### Understanding [tools]

The `[tools]` section declares what development tools are needed:

```toml
[tools]
# Runtime tools (interpreters, compilers)
node = "20"              # Node.js version 20.x.x
python = "3.11.5"        # Exact Python version
go = "latest"            # Latest stable Go
ruby = "~3.2"            # Ruby 3.2.x (any patch version)

# CLI tools via language package managers
"npm:prettier" = "latest"              # Install via npm
"go:github.com/user/tool/cmd/tool" = "latest"  # Install via go install
"cargo:ripgrep" = "14.0.0"             # Install via cargo
"pipx:black" = "latest"                # Install via pipx

# Tools from registries
"ubi:BurntSushi/ripgrep" = "latest"   # GitHub releases via ubi
"aqua:cli/cli" = "latest"             # Aqua registry
```

**Tool Name Patterns:**

- Plain name (e.g., `node`, `python`) → Core tools
- Prefixed (e.g., `npm:`, `go:`, `cargo:`) → Language package managers
- `ubi:user/repo` → GitHub releases via ubi backend
- `aqua:package` → Aqua registry packages

### Understanding [env]

The `[env]` section sets environment variables:

```toml
[env]
# Static values
NODE_ENV = "development"
API_KEY = "dev-key-123"

# Template values (dynamic)
VERSION = "{{exec(command='git describe --tags')}}"
CONFIG_DIR = "{{config_root}}/config"
HOME_DIR = "{{env.HOME}}"

# Load from .env file
_.file = ".env"
_.file = [".env", ".env.local"]

# Path modifications
[[env.PATH]]
value = "{{config_root}}/bin"
prefix = true  # Add to beginning of PATH

[[env.PATH]]
value = "{{config_root}}/scripts"
prefix = false  # Add to end of PATH
```

**Environment-Specific Overrides:**

```toml
[env]
API_URL = "http://localhost:3000"

[env.production]
API_URL = "https://api.production.com"

[env.staging]
API_URL = "https://api.staging.com"
```

Use with: `MISE_ENV=production mise run deploy`

### Understanding [vars]

The `[vars]` section defines internal constants (NOT exported as environment variables):

```toml
[vars]
app_name = "myapp"
version = "1.0.0"
build_dir = "dist"
ldflags = '-s -w -X main.version={{vars.version}}'
```

Use in tasks via templates:

```toml
[tasks.build]
run = "go build -ldflags '{{vars.ldflags}}' -o {{vars.build_dir}}/{{vars.app_name}}"
```

**Key Difference:**
- `[env]` → Exported to shell (accessible via `$VAR` in shell)
- `[vars]` → Internal to mise (only via `{{vars.name}}` templates)

### Understanding [tasks.*]

Tasks define commands to run:

```toml
[tasks.build]
description = "Build the application"  # REQUIRED
alias = "b"                            # Short name
depends = ["install", "lint"]          # Run these first
depends_post = ["notify"]              # Run these after
run = "npm run build"                  # Command to execute
run = ["cmd1", "cmd2", "cmd3"]        # Multiple commands
dir = "{{config_root}}/frontend"      # Working directory
env = { DEBUG = "1", VERBOSE = "true" }  # Task-specific env vars
sources = ["src/**/*.ts"]              # Input files
outputs = ["dist/**/*.js"]             # Output files
hide = false                           # Hide from `mise tasks` list
raw = false                            # Enable stdin
quiet = false                          # Suppress output
```

**Understanding Dependencies:**

```toml
[tasks.deploy]
depends = ["build", "test"]        # Run before (parallel if possible)
depends_post = ["cleanup"]         # Run after
wait_for = ["db-migrate"]          # Wait if running, but don't start it

[tasks.build]
description = "Build"
run = "npm run build"

[tasks.test]
description = "Test"
run = "npm test"

[tasks.cleanup]
description = "Clean up"
run = "rm -rf tmp/"
```

When running `mise run deploy`:
1. Runs `build` and `test` in parallel
2. Waits for both to complete
3. Runs `deploy`
4. After deploy completes, runs `cleanup`

## Understanding Tool Resolution

### Version Resolution

How mise resolves version specifications:

| Specification | Resolves To | Example |
|--------------|-------------|---------|
| `20` | Latest 20.x.x | `node = "20"` → 20.11.0 |
| `20.11` | Latest 20.11.x | `node = "20.11"` → 20.11.0 |
| `20.11.0` | Exact version | `node = "20.11.0"` → 20.11.0 |
| `latest` | Latest stable | `node = "latest"` → 20.11.0 |
| `~20.11` | Latest 20.11.x | `node = "~20.11"` → 20.11.5 |
| `^20.11` | Latest 20.x (≥20.11) | `node = "^20.11"` → 20.15.0 |

Check what versions are available:

```bash
# List remote versions
mise ls-remote node

# List with prefix
mise ls-remote node 20

# Get latest version
mise latest node
mise latest node@20
```

### Configuration Hierarchy

Mise loads configuration from multiple files in order:

1. `/etc/mise/config.toml` (system-wide)
2. `~/.config/mise/config.toml` (global user config)
3. `~/.config/mise/config.local.toml` (global local overrides)
4. `~/project/mise.toml` (project root)
5. `~/project/.mise.toml` (hidden project config)
6. `~/project/subdir/mise.toml` (nested configs)

Later files override earlier ones. Check the hierarchy:

```bash
mise config ls
```

Output shows which config files are loaded and from where.

### Tool Source Priority

When multiple configs specify the same tool:

1. Most specific (deepest directory) wins
2. Local (current directory) overrides global
3. Later in file overrides earlier

Example:

```toml
# ~/.config/mise/config.toml
[tools]
node = "18"  # Global default

# ~/project/mise.toml
[tools]
node = "20"  # Project override (this wins)
```

Check where a tool comes from:

```bash
mise ls node
# node  20.11.0  ~/project/mise.toml  20
```

## Understanding Task Discovery

### Task Sources

Mise discovers tasks from multiple sources:

1. **TOML tasks** in `mise.toml`:
   ```toml
   [tasks.build]
   description = "Build"
   run = "npm run build"
   ```

2. **File tasks** in `mise-tasks/` directory:
   ```bash
   mise-tasks/
   ├── build           # Executable script
   ├── test            # Executable script
   └── deploy/         # Nested task
       └── staging     # Executable script (task: deploy:staging)
   ```

3. **Includes** from other files:
   ```toml
   [tasks]
   _file = "tasks.toml"          # Load tasks from another file
   _dir = "mise-tasks"           # Load from directory
   ```

4. **Inherited** from parent directories

### Task Naming

Tasks can be namespaced with `:`:

```toml
[tasks.test]
description = "Run all tests"

[tasks."test:unit"]
description = "Run unit tests"

[tasks."test:integration"]
description = "Run integration tests"

[tasks."test:e2e"]
description = "Run e2e tests"
```

Quotes required when task name contains `:`.

File-based equivalent:

```bash
mise-tasks/
├── test              # mise run test
├── test:unit         # mise run test:unit
└── test/             # Alternative: nested directory
    ├── unit          # mise run test:unit
    ├── integration   # mise run test:integration
    └── e2e           # mise run test:e2e
```

### Task Visibility

Tasks can be hidden:

```toml
[tasks."_internal"]
description = "Internal helper"
hide = true
```

Hidden tasks:
- Don't appear in `mise tasks`
- Show with `mise tasks --hidden`
- Can still be used as dependencies
- Convention: prefix with `_`

## Checking Project Health

### Using mise doctor

Comprehensive health check:

```bash
mise doctor
```

Checks:
- Shell integration
- Mise configuration
- Tool installations
- PATH configuration
- Plugin status
- Settings validation

Example output:

```
version: 2024.1.35 macos-arm64 (abc1234 2024-01-15)
activated: yes
shims_on_path: no

build_info: ...
shell: /bin/zsh

mise.toml: ~/project/mise.toml

[WARN] node@20.0.0 is not installed
[WARN] python@3.11.0 is not installed
```

### Checking Tool Installation

Verify tools are installed:

```bash
# List installed tools
mise ls --installed

# Check specific tool
mise ls node

# Show missing tools
mise ls --missing

# Check if tool is in PATH
mise which node
mise which python

# Verify tool works
mise exec -- node --version
mise exec -- python --version
```

### Validating Configuration

Check for configuration issues:

```bash
# Validate syntax
mise doctor

# Format configuration
mise fmt

# Show parsed configuration
mise config ls --json

# Test tool resolution
mise ls --current

# Test environment
mise env
```

## Common Inspection Patterns

### Understanding a New Project

```bash
# 1. Find config files
mise config ls

# 2. See what tools are needed
cat mise.toml | grep -A 20 '\[tools\]'

# 3. Check if tools are installed
mise ls --missing

# 4. Install missing tools
mise install

# 5. See available tasks
mise tasks

# 6. Check health
mise doctor
```

### Debugging Tool Issues

```bash
# 1. Check tool is in config
cat mise.toml | grep toolname

# 2. Check if installed
mise ls toolname

# 3. Check version resolution
mise latest toolname
mise ls-remote toolname

# 4. Check PATH
mise which toolname
mise exec -- which toolname

# 5. Test execution
mise exec -- toolname --version

# 6. Check environment
mise env | grep TOOLNAME
```

### Understanding Task Dependencies

```bash
# 1. List all tasks
mise tasks

# 2. Get task details
mise tasks info build

# 3. Show dependency tree
mise tasks deps build

# 4. View task configuration
mise tasks info build --json

# 5. Test task execution
MISE_DEBUG=1 mise run build
```

## Working with JSON Output

### Tools

```bash
mise ls --json | jq
```

Output structure:

```json
{
  "node": [
    {
      "version": "20.11.0",
      "install_path": "/home/user/.mise/installs/node/20.11.0",
      "source": {
        "type": "mise.toml",
        "path": "/home/user/project/mise.toml"
      },
      "requested_version": "20",
      "symlinked_from": null,
      "active": true
    }
  ]
}
```

Query examples:

```bash
# Get active tools
mise ls --json | jq -r 'to_entries[] | select(.value[].active) | .key'

# Get tool versions
mise ls --json | jq -r '.node[].version'

# Get tools from specific source
mise ls --json | jq -r 'to_entries[] | select(.value[].source.path | contains("mise.toml")) | .key'
```

### Tasks

```bash
mise tasks ls --json | jq
```

Output structure:

```json
[
  {
    "name": "build",
    "description": "Build the application",
    "depends": ["install"],
    "sources": ["src/**/*.ts"],
    "outputs": ["dist/**/*.js"],
    "dir": "/home/user/project",
    "hide": false,
    "file": "/home/user/project/mise.toml"
  }
]
```

Query examples:

```bash
# Get tasks without descriptions
mise tasks ls --json | jq '.[] | select(.description == null or .description == "")'

# Get task dependencies
mise tasks ls --json | jq -r '.[] | select(.name == "build") | .depends[]'

# Get tasks with sources defined
mise tasks ls --json | jq '.[] | select(.sources != null)'
```

### Configuration

```bash
mise config ls --json | jq
```

Output structure:

```json
[
  {
    "path": "/home/user/project/mise.toml",
    "tools": ["node@20", "python@3.11"]
  }
]
```

## Environment Variables

### Mise-specific Variables

Variables set by mise:

```bash
MISE_PROJECT_ROOT     # Root of the project
MISE_CONFIG_ROOT      # Directory containing mise.toml
MISE_DATA_DIR         # ~/.local/share/mise
MISE_CACHE_DIR        # ~/.cache/mise
MISE_CONFIG_DIR       # ~/.config/mise
MISE_ORIGINAL_CWD     # Where mise was invoked
```

Check with:

```bash
mise env | grep MISE_
```

### User-defined Variables

From `[env]` section:

```bash
# Show all env vars
mise env

# Show specific var
mise env | grep NODE_ENV

# Show in shell format
eval "$(mise env -s bash)"
echo $NODE_ENV
```

## Best Practices for Project Inspection

1. **Start with `mise config ls`** - Find all configuration files
2. **Check `mise doctor`** - Identify issues early
3. **Use JSON output for scripting** - More reliable than parsing text
4. **Verify tool installation** - `mise ls --missing`
5. **Understand task dependencies** - `mise tasks deps <task>`
6. **Check environment variables** - `mise env`
7. **Test tools work** - `mise exec -- tool --version`
