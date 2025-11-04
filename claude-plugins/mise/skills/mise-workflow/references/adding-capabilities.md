# Adding Capabilities to Mise Projects - Advanced Topics

Advanced tool management, complex configurations, and edge cases.

> **Note**: For basic tool and task addition, use the main mise-workflow skill. This reference covers advanced scenarios only.

## Advanced Version Specifications

Beyond basic semantic versioning, mise supports several advanced version specifications:

### Git References

Use specific git branches, tags, or commits:

```toml
[tools]
# Git branch
node = "ref:main"
node = "ref:feature/new-api"

# Git tag
node = "ref:v20.11.0"

# Git commit
node = "ref:abc123def456"
```

Useful for:
- Testing pre-release versions
- Using custom forks
- Pinning to specific commits for reproducibility

### Local Paths

Use locally built or custom tool versions:

```toml
[tools]
# Absolute path
node = "path:/usr/local/custom-node"

# Relative path (from config root)
node = "path:./vendor/node"

# Home directory
node = "path:~/.local/share/custom-builds/node"
```

Use cases:
- Custom builds with patches
- Vendored tools
- Offline development
- Testing local modifications

### System Versions

Use system-installed tools instead of mise-managed:

```toml
[tools]
node = "system"     # Use system node
python = "system"   # Use system python
```

When to use:
- Tool installed by system package manager
- Organization requires specific system tools
- Avoid duplication in containers

### Version Prefixes

Match versions by prefix:

```toml
[tools]
# Matches latest version starting with "20"
node = "prefix:20"

# Matches latest 20.11.x
node = "prefix:20.11"

# Matches latest v1.2.x (with 'v' prefix)
go = "prefix:v1.2"
```

Difference from semantic versioning:
- `prefix:20` matches 20.11.0, 20.9.5, 20.1.0
- `20` (semantic) resolves to latest 20.x.x (e.g., 20.11.0)

### Sub-versions

Pin to specific sub-versions for reproducibility:

```toml
[tools]
# Exact version with sub-version
node = "20.11.0-alpine"
python = "3.11.5-dev"
```

## Advanced Tool Configuration

### Tool Options

Backends support custom options for specialized configurations:

#### Go Backend: Build Tags

```toml
[tools]
# Single tag
"go:github.com/golang-migrate/migrate/v4/cmd/migrate" = {
    version = "latest",
    tags = "postgres"
}

# Multiple tags
"go:github.com/golang-migrate/migrate/v4/cmd/migrate" = {
    version = "v4.16.0",
    tags = "postgres,mysql,mongodb"
}

# With specific version
"go:github.com/someorg/tool/cmd/tool" = {
    version = "v1.2.3",
    tags = "enterprise,feature_x"
}
```

#### Conditional Tool Installation

Install tools based on conditions:

```toml
[tools]
# Only on specific OS
"cargo:cargo-watch" = { version = "latest", os = ["linux", "macos"] }
"cargo:windows-specific-tool" = { version = "latest", os = ["windows"] }

# Only in specific environment
"npm:@sentry/cli" = { version = "latest", if = "{{ env.CI == '1' }}" }
"ubi:cli/cli" = { version = "latest", if = "{{ env.DEPLOY == 'true' }}" }

# Architecture-specific
python = { version = "3.11", if = "{{ arch == 'x86_64' }}" }
python = { version = "3.11-arm64", if = "{{ arch == 'arm64' }}" }
```

#### Python Backend: Virtualenv Options

```toml
[tools]
# Custom virtualenv location
python = { version = "3.11", virtualenv = ".venv" }

# Multiple Python versions with separate virtualenvs
"python:3.11" = { version = "3.11.5", virtualenv = ".venv-3.11" }
"python:3.12" = { version = "3.12.0", virtualenv = ".venv-3.12" }
```

### Multi-Tool Dependency Chains

Ensure tools are installed in correct order:

```toml
[tools]
# First install rust (required by cargo tools)
rust = "latest"

# Then cargo tools (depend on rust)
"cargo:ripgrep" = "latest"
"cargo:fd-find" = "latest"

# Go must be installed first
go = "1.21"

# Then go tools
"go:github.com/golangci/golangci-lint/cmd/golangci-lint" = "latest"

# Node for npm tools
node = "20"

# npm tools require node
"npm:prettier" = "latest"
"npm:eslint" = "8.50.0"
```

Tool order in `[tools]` section matters - mise installs top to bottom.

### Backend-Specific Repositories

Use custom registries or repositories:

```toml
[tools]
# npm: custom registry
"npm:@company/private-tool" = { version = "latest", registry = "https://npm.company.com" }

# go: private repository
"go:github.company.com/team/tool/cmd/tool" = "latest"

# cargo: git dependency
"cargo:custom-tool" = { version = "latest", git = "https://github.com/user/tool" }
```

### Version Constraints for Multiple Environments

```toml
[tools]
# Default (development)
node = "20"

[env.staging.tools]
# Override for staging
node = "20.11.0"

[env.production.tools]
# Pin exact version for production
node = "20.11.0"
```

## Advanced File Task Configuration

### Complete File Task Metadata

All available options in `#MISE` comments:

```bash
#!/usr/bin/env bash
#MISE description="Complex build task"
#MISE alias="b"
#MISE depends=["install", "codegen"]
#MISE depends_post=["notify", "cleanup"]
#MISE wait_for=["db-migrate"]
#MISE sources=["src/**/*.ts", "!src/**/*.test.ts"]
#MISE outputs=["dist/**/*.js", "dist/manifest.json"]
#MISE dir="{{config_root}}/packages/app"
#MISE env={NODE_ENV="production", DEBUG="false", VERBOSE="1"}
#MISE hide=false
#MISE raw=false
#MISE quiet=false

set -euo pipefail

# Complex build logic here
echo "Building from $MISE_CONFIG_ROOT..."
```

### Multi-Language File Tasks

**Ruby task:**

```ruby
#!/usr/bin/env ruby
#MISE description="Process data with Ruby"
#MISE sources=["data/**/*.csv"]
#MISE outputs=["processed/**/*.json"]

require 'csv'
require 'json'

# Process CSV files...
Dir.glob("data/*.csv").each do |file|
  # Processing logic
end
```

**Node.js task:**

```javascript
#!/usr/bin/env node
//MISE description="Build with Node.js"
//MISE depends=["install"]
//MISE sources=["src/**/*.ts"]
//MISE outputs=["dist/**/*.js"]

const fs = require('fs');
const path = require('path');

// Build logic here
```

**Python task with complex dependencies:**

```python
#!/usr/bin/env python3
#MISE description="Data pipeline task"
#MISE depends=["fetch-data", "validate-schema"]
#MISE sources=["data/raw/**/*.csv"]
#MISE outputs=["data/processed/**/*.parquet"]
#MISE env={PYTHONPATH="./src", LOG_LEVEL="INFO"}

import sys
import pandas as pd
from pathlib import Path

# Complex data processing
```

### Task Script Templating

Use mise templates in file task scripts:

```bash
#!/usr/bin/env bash
#MISE description="Deploy application"
#MISE env={VERSION="{{vars.version}}", ENV="{{env.DEPLOY_ENV}}"}

set -euo pipefail

# Template variables are expanded before execution
echo "Deploying version $VERSION to $ENV"

# Access mise variables
cd "$MISE_PROJECT_ROOT"
./scripts/deploy.sh \
    --version "$VERSION" \
    --environment "$ENV" \
    --region "{{vars.aws_region}}"
```

## Complex Template Usage

### Advanced Template Functions

```toml
[vars]
# Execute commands
git_sha = "{{exec(command='git rev-parse HEAD')}}"
git_branch = "{{exec(command='git branch --show-current')}}"
build_date = "{{exec(command='date +%Y%m%d')}}"
package_version = "{{exec(command='jq -r .version package.json')}}"

# Hash for cache busting
content_hash = "{{hash(value=config_root)}}"
lockfile_hash = "{{hash_file(file='package-lock.json')}}"

# Conditionals
deploy_env = "{{if(cond=env.PROD, t='production', f='development')}}"
debug_flag = "{{if(cond=vars.debug_mode, t='--debug', f='')}}"

# OS/Arch detection
binary_name = "app-{{os}}-{{arch}}"
build_target = "{{os}}-{{arch}}"

# Path manipulation
project_bin = "{{project_root}}/bin"
cache_dir = "{{config_root}}/.cache"
```

### Nested Template References

```toml
[vars]
version = "1.0.0"
env_suffix = "{{if(cond=env.PROD, t='prod', f='dev')}}"
full_version = "{{vars.version}}-{{vars.env_suffix}}"

[tasks.build]
description = "Build version {{vars.full_version}}"
run = "npm run build -- --version {{vars.full_version}}"
```

### Template Functions in Task Commands

```toml
[tasks.deploy]
description = "Deploy to environment"
run = """
#!/usr/bin/env bash
set -euo pipefail

# Variables are expanded before execution
VERSION="{{exec(command='git describe --tags')}}"
COMMIT="{{exec(command='git rev-parse --short HEAD')}}"
DATE="{{exec(command='date -u +%Y-%m-%dT%H:%M:%SZ')}}"

echo "Deploying $VERSION ($COMMIT) built on $DATE"
./deploy.sh --version "$VERSION" --commit "$COMMIT"
"""
```

## Monorepo Tool Management

### Root Configuration

```toml
# root mise.toml
[tools]
# Shared tools for all packages
node = "20"
"npm:prettier" = "latest"
"npm:eslint" = "latest"

[tasks.test-all]
description = "Test all packages"
run = [
    "mise --cd packages/api run test",
    "mise --cd packages/web run test",
    "mise --cd packages/worker run test",
]

[tasks.build-all]
description = "Build all packages in order"
run = [
    "mise --cd packages/shared run build",
    "mise --cd packages/api run build",
    "mise --cd packages/web run build",
]
```

### Package-Specific Overrides

```toml
# packages/api/mise.toml
[tools]
# Additional tools for API package
python = "3.11"  # API needs Python
"pipx:black" = "latest"

[tasks.build]
description = "Build API"
depends = ["//shared:build"]  # Depends on shared package
run = "npm run build"

[tasks.test]
description = "Test API"
depends = ["build"]
run = "pytest tests/"
```

### Shared Configuration Includes

Create reusable tool sets:

**mise-shared-tools.toml:**

```toml
[tools]
"npm:prettier" = "3.0.0"
"npm:eslint" = "8.50.0"
"npm:typescript" = "5.2.0"
```

**Root mise.toml:**

```toml
[config]
includes = ["mise-shared-tools.toml"]

[tools]
node = "20"
```

**Package mise.toml:**

```toml
[config]
includes = ["../../mise-shared-tools.toml"]

[tools]
# Package-specific tools
"npm:jest" = "29.0.0"
```

## Tool Backend Edge Cases

### asdf Plugin Compatibility

Use asdf plugins when core backend unavailable:

```toml
[tools]
# asdf plugin (legacy compatibility)
"asdf:hashicorp/terraform" = "1.5.0"

# Configure plugin
[tools.options]
asdf_terraform_version = "1.5.0"
asdf_terraform_plugin_url = "https://github.com/asdf-community/asdf-hashicorp"
```

### vfox Plugin Integration

Use vfox plugins for additional tools:

```toml
[tools]
# vfox plugin
"vfox:version-fox/vfox-nodejs" = "20.11.0"

[settings]
experimental = true  # Required for vfox
```

### Custom Backend Scripts

Create custom tool backends:

**mise-tasks/install-custom-tool:**

```bash
#!/usr/bin/env bash
#MISE description="Install custom tool"

set -euo pipefail

VERSION="${1:-latest}"
INSTALL_DIR="$MISE_INSTALLS_DIR/custom-tool/$VERSION"

mkdir -p "$INSTALL_DIR/bin"

# Custom installation logic
curl -fsSL "https://example.com/custom-tool-$VERSION.tar.gz" | \
    tar -xz -C "$INSTALL_DIR"

# Make executable
chmod +x "$INSTALL_DIR/bin/custom-tool"

echo "Custom tool $VERSION installed to $INSTALL_DIR"
```

### Backend Priority and Fallbacks

```toml
[tools]
# Try core backend first, fall back to asdf plugin
node = "20"  # Uses core:node backend

# If tool not in core, mise tries:
# 1. asdf plugin (if ~/.asdf/plugins/[tool] exists)
# 2. vfox plugin (if configured)
# 3. aqua registry
# 4. ubi (GitHub releases)

[settings]
# Control backend priority
backend_order = ["core", "asdf", "vfox", "aqua", "ubi"]
```

## Troubleshooting Tool Installation

### Debug Tool Installation

```bash
# Enable debug output
MISE_DEBUG=1 mise install node@20

# Show which backend is used
mise ls --json | jq '.[] | {tool, backend: .install_path | split("/") | .[7]}'

# Check tool path resolution
mise which node
mise which --path node

# Verify tool is in PATH
mise exec -- which node
```

### Installation Failures

**Tool not found:**

```bash
# Check if backend is available
mise backends

# Try explicit backend
mise use core:node@20
# or
mise use asdf:nodejs@20

# Search aqua registry
curl -s https://aquaproj.github.io/registry.json | jq '.packages[] | select(.name | contains("node"))'
```

**Version not available:**

```bash
# List available versions
mise ls-remote node

# Check backend-specific versions
mise ls-remote core:node
mise ls-remote asdf:nodejs

# Use version prefix to find available
mise ls-remote node | grep "^20\."
```

**Build failures:**

```bash
# Check build dependencies
mise doctor

# Install with verbose output
MISE_DEBUG=1 MISE_TRACE=1 mise install go:github.com/user/tool/cmd/tool

# Try with specific go version
mise use go@1.21
mise install go:github.com/user/tool/cmd/tool
```

### Tool Conflicts

Multiple tools providing same binary:

```bash
# Show which tool provides binary
mise which rg

# List all installed tools
mise ls --installed

# Remove conflicting tool
mise uninstall cargo:ripgrep
# or
mise use ubi:BurntSushi/ripgrep  # Switch backend
```

### Cache Issues

```bash
# Clear tool cache
rm -rf ~/.cache/mise/

# Clear specific tool cache
rm -rf ~/.cache/mise/[tool-name]/

# Reinstall without cache
mise install --force node@20
```

## Performance Optimization

### Parallel Tool Installation

```bash
# Install multiple tools in parallel
mise install --jobs 8

# Install specific tools in parallel
mise install node@20 python@3.11 go@latest --jobs 3
```

### Minimal Tool Sets

```toml
[tools]
# Core tools only
node = "20"
python = "3.11"

[env.development.tools]
# Additional development tools
"npm:nodemon" = "latest"
"pipx:ipython" = "latest"

[env.ci.tools]
# CI-specific tools
"npm:@sentry/cli" = "latest"
```

Use:

```bash
# Development
mise install

# CI (minimal set)
MISE_ENV=ci mise install
```

### Tool Installation Scripts

Batch install tools:

```bash
#!/usr/bin/env bash
# install-tools.sh

set -euo pipefail

echo "Installing core tools..."
mise install node@20 python@3.11 go@latest --jobs 4

echo "Installing CLI tools..."
mise install \
    npm:prettier \
    npm:eslint \
    cargo:ripgrep \
    cargo:fd-find \
    --jobs 4

echo "Installing project tools..."
mise install  # Install remaining from mise.toml

echo "✅ All tools installed!"
```

## Advanced Patterns

### Environment-Specific Tools

```toml
[tools]
# Default tools
node = "20"

[env.testing.tools]
# Testing environment needs older node
node = "18"
"npm:jest" = "27.0.0"

[env.production.tools]
# Production uses latest stable
node = "20.11.0"  # Pinned version
```

### Feature Flag Tools

```toml
[tools]
# Always installed
node = "20"

# Conditional installation
"npm:storybook" = { version = "latest", if = "{{ env.ENABLE_STORYBOOK == '1' }}" }
"npm:cypress" = { version = "latest", if = "{{ env.E2E_TESTS == '1' }}" }
```

Usage:

```bash
# Without feature flags
mise install

# With feature flags
ENABLE_STORYBOOK=1 E2E_TESTS=1 mise install
```

### Tool Aliasing

```toml
[tools]
# Multiple versions of same tool
"node:20" = "20.11.0"
"node:18" = "18.18.0"

[tasks.test-compat]
description = "Test compatibility across Node versions"
run = [
    "mise exec node:20 -- npm test",
    "mise exec node:18 -- npm test",
]
```

### Dynamic Tool Versions

```toml
[vars]
node_version = "{{exec(command='cat .nvmrc')}}"
python_version = "{{exec(command='cat .python-version')}}"

[tools]
node = "{{vars.node_version}}"
python = "{{vars.python_version}}"
```

Reads versions from external files.

### Tool Version Matrices

Test against multiple versions:

```toml
[tasks.test-matrix]
description = "Test against multiple Node versions"
run = """
for version in 18 20 21; do
  echo "Testing with Node $version..."
  mise use node@$version
  mise install
  npm test
done
"""
```

### Offline Tool Installation

Pre-download tools for offline use:

```bash
# Download tools to cache
mise install node@20
mise install python@3.11

# Cache location
ls -la ~/.cache/mise/

# In offline environment, tools install from cache
mise install  # Uses cached downloads
```

### Container-Optimized Tool Management

```dockerfile
# Dockerfile
FROM debian:bullseye-slim

# Install mise
RUN curl https://mise.run | sh

# Copy mise config
COPY mise.toml /app/

# Install tools (cached layer)
RUN cd /app && mise install

# Only system tools in container
COPY mise.toml /app/
RUN cd /app && \
    mise use --pin node@20 python@3.11 && \
    mise install && \
    rm -rf ~/.cache/mise/
```

### Tool Verification

Add checksums for security:

```toml
[tools]
node = { version = "20.11.0", checksum = "sha256:..." }

[settings]
verify_checksums = true  # Fail if checksum doesn't match
```

## Migration Patterns

### From asdf to mise

```bash
# Read from .tool-versions
cat .tool-versions
# nodejs 20.11.0
# python 3.11.5

# Generate mise.toml
mise use node@20.11.0 python@3.11.5

# Migrate plugins
for plugin in $(asdf plugin list); do
    version=$(asdf current $plugin | awk '{print $2}')
    mise use $plugin@$version
done
```

### From nvm/rbenv/pyenv to mise

```bash
# Read from version files
NODE_VERSION=$(cat .nvmrc)
PYTHON_VERSION=$(cat .python-version)
RUBY_VERSION=$(cat .ruby-version)

# Generate mise config
mise use node@$NODE_VERSION
mise use python@$PYTHON_VERSION
mise use ruby@$RUBY_VERSION
```

### Gradual Migration

```toml
[tools]
# Keep existing version managers for some tools
node = "system"  # Still using nvm
python = "3.11"  # Migrated to mise
ruby = "system"  # Still using rbenv

# Gradually migrate each tool
```

## Migrating from Tera Templates to Usage

### Background

**Tera templates** (deprecated, removal in mise 2026.11.0) were the old way to define task arguments using template functions like `{{arg()}}`, `{{option()}}`, and `{{flag()}}` directly in the `run` script.

**Problems with Tera:**
- Two-pass parsing issues (templates return empty strings during spec collection)
- Inconsistent shell escaping across different shells
- No automatic `--help` generation
- Poor separation of argument definition from script logic

**Usage field** (recommended) uses clean KDL syntax in a dedicated field, with arguments available as `$usage_*` environment variables.

### Migration Examples

#### Example 1: Simple Argument with Default

**Old (Tera):**
```toml
[tasks.test]
description = "Run tests"
run = 'npm test -- {{arg(name="filter", default=".*")}}'
```

**New (Usage):**
```toml
[tasks.test]
description = "Run tests"
usage = '''
arg "<filter>" help="Test filter pattern" default=".*"
'''
run = 'npm test -- "${usage_filter?}"'
```

#### Example 2: Multiple Arguments and Flags

**Old (Tera):**
```toml
[tasks.deploy]
description = "Deploy application"
run = '''
./deploy.sh \
  {{arg(name="environment")}} \
  {{option(name="region", default="us-east-1")}} \
  {{flag(name="dry-run")}}
'''
```

**New (Usage):**
```toml
[tasks.deploy]
description = "Deploy application"
usage = '''
arg "<environment>" help="Target environment"
flag "--region <region>" help="AWS region" default="us-east-1"
flag "--dry-run" help="Preview changes only"
'''
run = '''
./deploy.sh "${usage_environment?}" \
  --region "${usage_region?}" \
  ${usage_dry_run:+--dry-run}
'''
```

#### Example 3: Choices/Validation

**Old (Tera):**
```toml
[tasks.build]
description = "Build application"
run = 'cargo build --profile {{arg(name="profile", default="dev")}}'
```

**New (Usage):**
```toml
[tasks.build]
description = "Build application"
usage = '''
arg "<profile>" help="Build profile" default="dev" {
  choices "dev" "release" "debug"
}
'''
run = 'cargo build --profile "${usage_profile?}"'
```

#### Example 4: Variadic Arguments

**Old (Tera):**
```toml
[tasks.lint]
description = "Lint files"
run = 'eslint {{arg(name="files", var=true)}}'
```

**New (Usage):**
```toml
[tasks.lint]
description = "Lint files"
usage = '''
arg "<files>" help="Files to lint" var=#true
'''
run = 'eslint ${usage_files?}'
```

#### Example 5: File Task Migration

**Old (Tera in file task):**
```bash
#!/usr/bin/env bash
#MISE description="Process files"
set -euo pipefail
INPUT="{{arg(name='input')}}"
OUTPUT="{{option(name='output', default='out.txt')}}"
VERBOSE="{{flag(name='verbose')}}"
```

**New (Usage in file task):**
```bash
#!/usr/bin/env bash
#MISE description="Process files"
#USAGE arg "<input>" help="Input file"
#USAGE flag "--output <output>" help="Output file" default="out.txt"
#USAGE flag "-v --verbose" help="Verbose output"
set -euo pipefail
INPUT="${usage_input?}"
OUTPUT="${usage_output?}"
VERBOSE="${usage_verbose:-false}"
```

### Migration Checklist

1. **Move argument definitions** from `run` to `usage` field
2. **Convert syntax:**
   - `{{arg(name="x")}}` → `arg "<x>"` in usage, `${usage_x?}` in run
   - `{{option(name="x", default="y")}}` → `flag "--x <x>" default="y"` + `${usage_x?}`
   - `{{flag(name="x")}}` → `flag "--x"` + `${usage_x:+--x}` or `${usage_x:-false}`
3. **Add help text** - Now required and generates `--help` automatically
4. **Use bash expansions:**
   - `${var?}` for required arguments (error if unset)
   - `${var:-default}` for optional with defaults
   - `${var:+--flag}` for boolean flags (only include if true)
5. **Add choices** if argument has limited valid values
6. **Test execution** - Verify arguments work as expected
7. **Check help output** - Run `mise run taskname --help`

### Why Migrate Now

- **Removal scheduled** - Tera template support ends in mise 2026.11.0
- **Better UX** - Automatic help generation and validation
- **Clearer code** - Separation of argument definition from logic
- **More reliable** - No two-pass parsing issues or escaping problems

### Quick Find & Replace Patterns

Search for tasks using Tera templates:
```bash
# Find tasks with Tera templates
grep -r "{{arg\|{{option\|{{flag" mise.toml mise-tasks/
```

Convert common patterns:
- `{{arg(name="x")}}` → Add `arg "<x>"` to usage, use `${usage_x?}` in run
- `{{flag(name="x")}}` → Add `flag "--x"` to usage, use `${usage_x:+--x}` in run
- `default=` in Tera → `default=` in usage field
- No equivalent for help text in Tera → Add `help=` in usage field
