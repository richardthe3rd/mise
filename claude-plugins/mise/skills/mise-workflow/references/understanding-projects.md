# Understanding Mise Projects - Advanced Topics

Deep dive into configuration hierarchy, tool resolution, and complex project scenarios.

> **Note**: For basic project inspection, use the main mise-workflow skill. This reference covers advanced scenarios only.

## Configuration Hierarchy & Precedence

Mise loads configuration from multiple locations. Later files override earlier ones:

1. `/etc/mise/config.toml` (system-wide)
2. `~/.config/mise/config.toml` (global user)
3. `~/.config/mise/conf.d/*.toml` (global config directory)
4. Parent directories walking up to `/` or `~` (project configs)
5. `./mise.toml` or `./.mise.toml` (current directory)
6. `./config/mise.toml` (XDG-style config)
7. `.mise.local.toml` (local overrides, gitignored)

```bash
# Show all loaded configs with precedence
mise config ls --json

# See which config file set a particular tool
mise ls --json | jq '.[] | {tool, source: .source.path}'
```

### Environment-Specific Configs

```toml
[env]
API_URL = "http://localhost:3000"  # Default

[env.production]
API_URL = "https://api.production.com"

[env.staging]
API_URL = "https://api.staging.com"
```

Use: `MISE_ENV=production mise run deploy`

### Config Includes

```toml
# Include other config files
[config]
includes = ["mise-local.toml", "team-tools.toml"]
```

## Advanced Tool Resolution

### Version Specifications

| Specification | Resolves To | Example |
|--------------|-------------|---------|
| `20` | Latest 20.x.x | `node = "20"` → 20.11.0 |
| `20.11` | Latest 20.11.x | `node = "20.11"` → 20.11.0 |
| `20.11.0` | Exact version | `node = "20.11.0"` → 20.11.0 |
| `latest` | Latest stable | `node = "latest"` → 20.11.0 |
| `~20.11` | Latest 20.11.x | `node = "~20.11"` → 20.11.5 |
| `^20.11` | Latest 20.x (≥20.11) | `node = "^20.11"` → 20.15.0 |
| `ref:main` | Git branch | `node = "ref:main"` → main branch |
| `path:./local` | Local path | `node = "path:./local-node"` |
| `prefix:v1.2` | Prefix match | `go = "prefix:v1.2"` → v1.2.3 |

### Tool Version Files

Mise respects version files from other tools:

- `.node-version` → node version
- `.python-version` → python version
- `.ruby-version` → ruby version
- `.java-version` → java version
- `.terraform-version` → terraform version
- `.tool-versions` → asdf-compatible multi-tool file

**Precedence**: `mise.toml` > `.tool-versions` > `.[tool]-version`

### Backend Selection for Tools

Understanding which backend mise uses:

```bash
# Show which backend manages each tool
mise ls --json | jq '.[] | {tool, backend: .install_path | split("/") | .[7]}'
```

**Backend Priority**:
1. Core backends (node, python, ruby, go, java) - built-in
2. asdf plugins - if `~/.asdf/plugins/[tool]` exists
3. vfox plugins - if configured
4. Aqua registry - for known packages
5. ubi - for GitHub releases

## Advanced Environment Management

### Template Variables

```toml
[env]
# System info
HOME_DIR = "{{env.HOME}}"
USER = "{{env.USER}}"
CWD = "{{cwd}}"

# Config locations
CONFIG_ROOT = "{{config_root}}"  # Where mise.toml is
PROJECT_ROOT = "{{project_root}}" # Git root or mise.toml location

# Dynamic execution
GIT_SHA = "{{exec(command='git rev-parse HEAD')}}"
BUILD_TIME = "{{exec(command='date +%Y%m%d')}}"

# Hash for cache busting
CACHE_KEY = "{{hash(value=config_root)}}"
FILE_HASH = "{{hash_file(file='package-lock.json')}}"
```

### Path Manipulation

```toml
[env]
# Add to beginning of PATH
[[env.PATH]]
value = "{{config_root}}/bin"
prefix = true

# Add to end of PATH
[[env.PATH]]
value = "{{config_root}}/scripts"
prefix = false

# Multiple path entries
[[env.PATH]]
value = "node_modules/.bin:{{config_root}}/tools"
prefix = true
```

### Environment Files

```toml
[env]
# Load from .env file
_.file = ".env"

# Load multiple files (later overrides earlier)
_.file = [".env", ".env.local", ".env.{{env.USER}}"]

# Conditional loading
_.file = "{{env.HOME}}/.secrets/{{project_name}}.env"
```

## Advanced Task Configuration

### Complex Dependencies

```toml
[tasks.deploy]
depends = ["build", "test"]        # Run before (parallel)
depends_post = ["notify", "tag"]   # Run after (parallel)
wait_for = ["db-migrate"]          # Wait if running, don't start

[tasks.build]
depends = ["install"]
run = "npm run build"

[tasks.test]
depends = ["build"]
run = "npm test"

[tasks.db-migrate]
run = "db-migrate up"

[tasks.notify]
run = "echo 'Deployed!' | mail -s 'Deploy' team@example.com"

[tasks.tag]
run = "git tag v{{vars.version}}"
```

Execution order for `mise run deploy`:
1. `install` runs first
2. `build` runs after `install`
3. `test` and continued `build` run in parallel with `deploy` waiting
4. `deploy` runs after both complete, waits for `db-migrate` if it's running
5. `notify` and `tag` run in parallel after `deploy`

### File-Based Tasks with Metadata

`mise-tasks/complex-build`:
```bash
#!/usr/bin/env bash
#MISE description="Complex build with multiple steps"
#MISE depends=["install", "codegen"]
#MISE sources=["src/**/*.ts", "!src/**/*.test.ts"]
#MISE outputs=["dist/**/*.js", "dist/**/*.d.ts"]
#MISE dir="{{config_root}}"
#MISE env={NODE_ENV="production", DEBUG="false"}

set -euo pipefail
# Complex build logic here
```

### Task Inputs/Outputs for Caching

```toml
[tasks.build]
description = "Build with caching"
run = "npm run build"
sources = [
    "src/**/*.ts",
    "package.json",
    "tsconfig.json",
    "!src/**/*.test.ts"  # Exclude test files
]
outputs = [
    "dist/**/*.js",
    "dist/**/*.d.ts",
    "dist/manifest.json"
]
```

When sources haven't changed since outputs were created, mise skips the task.

### Task-Specific Environment

```toml
[tasks.build]
description = "Build for production"
env = { NODE_ENV = "production", DEBUG = "false" }
run = "npm run build"

[tasks.dev]
description = "Development server"
env = { NODE_ENV = "development", DEBUG = "true", PORT = "3000" }
run = "npm run dev"
```

## Troubleshooting Complex Scenarios

### Multi-Tool Conflicts

When tools conflict (e.g., system python vs mise python):

```bash
# Show PATH resolution
mise env | grep PATH

# Show which binary is used
mise which python
mise which pip

# Show all versions available
mise ls python
mise ls --installed python

# Force install specific version
mise install python@3.11.5
```

### Configuration Debugging

```bash
# Show resolved configuration
mise config ls --json

# Show why a tool was selected
MISE_DEBUG=1 mise install node

# Show task resolution
mise tasks info build

# Show dependency graph
mise tasks deps build --dot | dot -Tpng > deps.png
```

### Environment Variable Issues

```bash
# Show all environment variables mise sets
mise env

# Compare with current shell
mise env | diff - <(env | sort)

# Show env for specific environment
MISE_ENV=production mise env

# Debug template rendering
MISE_DEBUG=1 mise env | grep -A5 "template"
```

## Performance Optimization

### Parallel Tool Installation

```bash
# Install tools in parallel (default: 4 workers)
mise install --jobs 8

# Install specific tools in parallel
mise install node@20 python@3.11 go@latest --jobs 3
```

### Sparse Tool Installation

```toml
[tools]
# Only install tools needed for current OS
"cargo:cargo-watch" = { version = "latest", os = ["linux", "macos"] }
"go:github.com/user/tool/cmd/tool" = { version = "latest", os = ["linux"] }

# Only install in specific environments
"npm:@sentry/cli" = { version = "latest", if = "{{ env.CI == '1' }}" }
```

### Task Parallelization

```toml
[tasks.test-all]
description = "Run all tests in parallel"
run = [
    "mise run test:unit &",
    "mise run test:integration &",
    "mise run test:e2e &",
    "wait"
]

# Or use depends for automatic parallelization
[tasks.test-all]
depends = ["test:unit", "test:integration", "test:e2e"]
```

## Monorepo Patterns

### Workspace Configuration

Root `mise.toml`:
```toml
[tools]
node = "20"
python = "3.11"

[tasks.test-all]
description = "Test all packages"
run = [
    "mise --cd packages/pkg1 run test",
    "mise --cd packages/pkg2 run test",
    "mise --cd packages/pkg3 run test",
]

[tasks.build-all]
description = "Build all packages"
run = "for pkg in packages/*; do mise --cd $pkg run build; done"
```

Per-package `packages/pkg1/mise.toml`:
```toml
[tasks.build]
description = "Build this package"
run = "npm run build"

[tasks.test]
description = "Test this package"
depends = ["build"]
run = "npm test"
```

### Shared Tool Configuration

`mise-shared.toml`:
```toml
[tools]
# Shared tools across all packages
"npm:prettier" = "3.0.0"
"npm:eslint" = "8.50.0"
```

Root `mise.toml`:
```toml
[config]
includes = ["mise-shared.toml"]
```

Package `mise.toml`:
```toml
[config]
includes = ["../../mise-shared.toml"]

[tools]
# Package-specific tools
"npm:jest" = "29.0.0"
```
