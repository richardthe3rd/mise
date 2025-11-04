# Running Commands with Mise - Advanced Topics

Advanced task execution patterns, parallel execution, and complex orchestration scenarios.

> **Note**: For basic task execution, use the main mise-workflow skill. This reference covers advanced scenarios only.

## Advanced Dependency Control

Three types of dependencies provide fine-grained orchestration control:

### depends - Parallel Pre-Dependencies

```toml
[tasks.deploy]
depends = ["build", "test"]  # Both run in parallel before deploy
run = "./deploy.sh"
```

### depends_post - Post-Task Actions

Run tasks after completion, useful for cleanup or notifications:

```toml
[tasks.build]
description = "Build application"
run = "npm run build"
depends_post = ["notify", "tag-release"]

[tasks.notify]
description = "Send build notification"
run = "curl -X POST $WEBHOOK_URL -d 'Build complete'"

[tasks.tag-release]
description = "Tag git release"
run = "git tag v{{vars.version}}"
```

### wait_for - Coordination Without Dependencies

Wait for a task if it's already running, but don't start it:

```toml
[tasks.test]
description = "Run tests"
wait_for = ["db-migrate"]  # If db-migrate is running, wait; don't start it
run = "npm test"

[tasks.db-migrate]
description = "Run migrations"
run = "./migrate.sh"
```

Useful when multiple developers might trigger tasks simultaneously.

### Complex Dependency Graphs

```toml
[tasks.deploy]
depends = ["build", "test"]        # Run before (parallel)
depends_post = ["notify", "tag"]   # Run after (parallel)
wait_for = ["db-migrate"]          # Wait if running

[tasks.build]
depends = ["install", "codegen"]

[tasks.test]
depends = ["build"]

[tasks.db-migrate]
run = "db-migrate up"
```

Execution order for `mise run deploy`:
1. `install` and `codegen` run in parallel
2. `build` runs after both complete
3. `test` runs after `build`
4. `deploy` waits for `db-migrate` if running
5. `deploy` runs after `build` and `test` complete
6. `notify` and `tag` run in parallel after `deploy`

## Wildcards and Pattern Matching

Use glob patterns to match multiple tasks:

```bash
mise run test:*           # All test tasks
mise run test:**          # All nested test tasks
mise run lint:{js,css}    # Run lint:js and lint:css
```

### Pattern Syntax

| Pattern | Description | Example Match |
|---------|-------------|---------------|
| `?` | Single character | `test?` → test1, test2 |
| `*` | 0+ characters | `test:*` → test:unit, test:e2e |
| `**` | 0+ groups | `test:**` → test:unit:db, test:e2e:api |
| `{a,b}` | Alternatives | `{lint,test}` → lint, test |
| `[abc]` | Character set | `test[123]` → test1, test2 |
| `[!abc]` | Negation | `test[!3]` → test1, test2 |

### Wildcard Dependencies

```toml
[tasks."lint:eslint"]
run = "eslint src/"

[tasks."lint:prettier"]
run = "prettier --check src/"

[tasks."lint:tsc"]
run = "tsc --noEmit"

[tasks.lint]
description = "Run all linters"
depends = ["lint:*"]  # Runs all lint:* tasks in parallel
```

## Parallel Execution and Output

### Tuning Parallelism

```bash
# Set parallel job count
mise run --jobs 8 test
MISE_JOBS=8 mise run test
mise settings set jobs 8

# Sequential execution
mise run --jobs 1 test
```

Default: 4 parallel jobs

### Output Modes

**Line-buffered (default)** - Prefixes each line with task name:

```bash
mise run test
# Output:
# [test:unit] Running unit tests...
# [test:integration] Running integration tests...
```

**Interleaved (raw)** - Shows output immediately:

```bash
mise run --interleave test
# or
export MISE_TASK_OUTPUT=interleave
```

Use interleaved when:
- Running a single task
- Need interactive output
- Debugging with real-time logs

### Standard Input Handling

By default, tasks don't receive stdin. Enable with `raw = true`:

```toml
[tasks.interactive]
description = "Interactive task"
raw = true  # Enable stdin
run = "read -p 'Enter name: ' name && echo Hello $name"
```

When `raw = true`:
- Task receives stdin
- Runs exclusively (no parallel execution)
- Output redactions disabled
- Useful for prompts, interactive CLIs, debuggers

## File-Based Task Triggering

Use sources and outputs to skip unnecessary rebuilds:

```toml
[tasks.build]
description = "Build the CLI"
run = "cargo build"
sources = ['Cargo.toml', 'Cargo.lock', 'src/**/*.rs']
outputs = ['target/debug/mycli']
```

### Timestamp Comparison

Mise compares modification times:
- If outputs are newer than sources → skip task
- If sources changed → run task
- On first run or missing outputs → always run

### Complex Source Patterns

```toml
[tasks.build-frontend]
description = "Build frontend assets"
sources = [
    "src/**/*.ts",
    "src/**/*.tsx",
    "package.json",
    "tsconfig.json",
    "!src/**/*.test.ts",    # Exclude tests
    "!src/**/*.stories.tsx" # Exclude stories
]
outputs = [
    "dist/**/*.js",
    "dist/**/*.css",
    "dist/manifest.json"
]
run = "npm run build"
```

### File Task Scripts with Sources

For file-based tasks (`mise-tasks/build`):

```bash
#!/usr/bin/env bash
#MISE description="Build with caching"
#MISE sources=["src/**/*.ts", "package.json"]
#MISE outputs=["dist/**/*.js"]
#MISE depends=["install"]

set -euo pipefail
npm run build
```

## Task Configuration Reference

All available task options:

```toml
[tasks.example]
description = "Task description (REQUIRED)"
alias = "ex"                    # Alternative name
depends = ["other-task"]        # Pre-dependencies (parallel)
depends_post = ["cleanup"]      # Post-dependencies (parallel)
wait_for = ["db"]               # Wait without starting
run = "echo 'Hello'"            # Command (string)
run = ["cmd1", "cmd2"]          # Commands (array)
dir = "{{config_root}}/sub"     # Working directory
env = { VAR = "value" }         # Environment variables
sources = ["src/**/*.ts"]       # Input files (for caching)
outputs = ["dist/**/*.js"]      # Output files (for caching)
hide = false                    # Hide from task list
raw = false                     # Enable stdin
quiet = false                   # Suppress output
shell = "bash -c"               # Shell to use
```

## Task Execution Context

Environment variables available in tasks:

```bash
# Mise-provided variables
MISE_PROJECT_ROOT      # Git root or directory with .mise.toml
MISE_CONFIG_ROOT       # Directory containing mise.toml
MISE_ORIGINAL_CWD      # Directory where mise run was called
MISE_TASK_NAME         # Name of the current task
MISE_TASK_DIR          # Directory containing task script (file tasks)
MISE_TASK_FILE         # Full path to task script (file tasks)

# Tool paths
PATH                   # Includes mise-managed tools
# Plus all env vars from [env] section
```

### Using Context Variables

```toml
[tasks.clean]
description = "Clean build artifacts"
run = "rm -rf {{config_root}}/dist {{config_root}}/build"

[tasks.archive]
description = "Create release archive"
run = """
cd {{config_root}}
tar czf release-{{env.VERSION}}.tar.gz dist/
"""

[tasks.deploy]
description = "Deploy from project root"
dir = "{{project_root}}"
run = "./scripts/deploy.sh"
```

## Task Organization Patterns

### Semantic Namespacing

Use `:` to create task hierarchies:

```toml
[tasks."test:unit"]
description = "Run unit tests"
run = "npm run test:unit"

[tasks."test:integration"]
description = "Run integration tests"
run = "npm run test:integration"

[tasks."test:e2e:local"]
description = "Run e2e tests locally"
run = "npm run test:e2e"

[tasks."test:e2e:ci"]
description = "Run e2e tests in CI"
env = { CI = "1" }
run = "npm run test:e2e"

[tasks.test]
description = "Run all tests"
depends = ["test:*"]  # Matches test:unit, test:integration, test:e2e:*
```

### Hidden Tasks

Hide implementation details from `mise tasks`:

```toml
[tasks."_internal:setup"]
description = "Internal setup task"
hide = true
run = "echo 'Setting up...'"

[tasks."_internal:cleanup"]
description = "Internal cleanup task"
hide = true
run = "echo 'Cleaning up...'"

[tasks.ci]
description = "Run CI pipeline"
depends = ["_internal:setup", "lint", "test"]
depends_post = ["_internal:cleanup"]
```

Hidden tasks:
- Don't appear in `mise tasks` by default
- Show with `mise tasks --hidden`
- Still usable as dependencies
- Prefix with `_` by convention

## Debugging Complex Task Execution

### Debug Output Levels

```bash
# Basic debugging
MISE_DEBUG=1 mise run build

# Verbose tracing
MISE_TRACE=1 mise run build
```

Shows:
- Task resolution and dependency order
- Command execution and exit codes
- Environment setup
- File timestamp comparisons (sources/outputs)
- Parallel execution scheduling

### Dependency Graph Visualization

```bash
# Show dependency tree
mise tasks deps build

# Generate DOT graph
mise tasks deps build --dot

# Visualize with graphviz
mise tasks deps build --dot | dot -Tpng > deps.png
```

### Testing Complex Dependencies

```bash
# Check task definition
mise tasks info build

# Test dependencies individually
mise run <dependency>

# Run with sequential execution to debug
mise run --jobs 1 build

# Check for circular dependencies
# mise will error if detected
```

### Debugging File-Based Triggering

```bash
# Force run (ignore sources/outputs)
rm dist/app  # Remove outputs
mise run build

# Check source/output timestamps
ls -l src/main.ts dist/app

# Enable debug to see timestamp comparison
MISE_DEBUG=1 mise run build
```

## Advanced Execution Patterns

### Conditional Task Execution

Using shell conditionals:

```toml
[tasks.build]
description = "Build with platform detection"
run = '''
if [[ "$OSTYPE" == "darwin"* ]]; then
  npm run build:mac
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
  npm run build:linux
else
  npm run build:generic
fi
'''
```

Using MISE_ENV:

```toml
[tasks.deploy]
description = "Deploy to environment"
run = '''
case "$MISE_ENV" in
  production)
    ./deploy-prod.sh
    ;;
  staging)
    ./deploy-staging.sh
    ;;
  *)
    echo "Unknown environment: $MISE_ENV"
    exit 1
    ;;
esac
'''
```

### Task Composition Patterns

**Sequential pipeline:**

```toml
[tasks.ci]
description = "CI pipeline"
run = [
    "mise run lint",
    "mise run test",
    "mise run build",
    "mise run security-scan"
]
```

**Parallel with barrier:**

```toml
[tasks.deploy]
description = "Deploy with validation"
depends = ["build", "test", "security-scan"]  # Parallel
run = "./deploy.sh"                            # Runs after all complete
depends_post = ["smoke-test", "notify"]        # Parallel after deploy
```

**Complex multi-stage:**

```toml
[tasks.release]
description = "Full release process"
depends = ["ci", "version:bump"]
run = [
    "mise run changelog:generate",
    "mise run build:release",
    "mise run package",
    "mise run publish"
]
depends_post = ["git:tag", "github:release", "notify:slack"]
```

### Multi-Environment Tasks

Using `MISE_ENV`:

```toml
[env.development]
API_URL = "http://localhost:3000"
DB_HOST = "localhost"

[env.staging]
API_URL = "https://api.staging.example.com"
DB_HOST = "db.staging.internal"

[env.production]
API_URL = "https://api.example.com"
DB_HOST = "db.prod.internal"

[tasks.deploy]
description = "Deploy application"
run = "./deploy.sh"  # Uses env vars from current MISE_ENV
```

Run with:

```bash
MISE_ENV=production mise run deploy
```

Per-environment task variants:

```toml
[tasks."deploy:staging"]
description = "Deploy to staging"
env = { ENV = "staging", API_URL = "https://api.staging.example.com" }
run = "./deploy.sh"

[tasks."deploy:production"]
description = "Deploy to production"
env = { ENV = "production", API_URL = "https://api.example.com" }
run = "./deploy.sh"
```

### Integration with External Tools

**Docker integration:**

```toml
[tasks."docker:build"]
description = "Build Docker image"
run = "docker build -t myapp:{{env.VERSION}} ."
sources = ["Dockerfile", "src/**/*", "package.json"]
outputs = [".docker-built"]  # Sentinel file

[tasks."docker:push"]
description = "Push Docker image"
depends = ["docker:build"]
run = [
    "docker push myapp:{{env.VERSION}}",
    "docker tag myapp:{{env.VERSION}} myapp:latest",
    "docker push myapp:latest"
]

[tasks."docker:run"]
description = "Run container locally"
depends = ["docker:build"]
run = "docker run -p 3000:3000 myapp:{{env.VERSION}}"
```

**Kubernetes integration:**

```toml
[tasks."k8s:deploy"]
description = "Deploy to Kubernetes"
depends = ["docker:push"]
run = [
    "kubectl apply -f k8s/",
    "kubectl rollout status deployment/myapp"
]

[tasks."k8s:rollback"]
description = "Rollback deployment"
run = "kubectl rollout undo deployment/myapp"

[tasks."k8s:logs"]
description = "Stream logs"
raw = true  # For interactive kubectl logs
run = "kubectl logs -f deployment/myapp"
```

**CI/CD integration:**

```toml
[tasks.ci]
description = "CI pipeline"
depends = ["lint", "test", "build"]
run = [
    "mise run coverage:report",
    "mise run security:scan"
]

[tasks."ci:github"]
description = "GitHub Actions CI"
env = { CI = "1", GITHUB_ACTIONS = "true" }
run = [
    "mise run ci",
    "mise run coverage:upload"
]

[tasks."ci:gitlab"]
description = "GitLab CI"
env = { CI = "1", GITLAB_CI = "true" }
run = [
    "mise run ci",
    "mise run artifacts:upload"
]
```

## Performance Optimization

### Parallel Execution Tuning

```bash
# Match CPU cores
MISE_JOBS=$(nproc) mise run test

# Conservative (less resource usage)
MISE_JOBS=2 mise run test

# Aggressive (faster, more resources)
MISE_JOBS=16 mise run test
```

Set permanently:

```bash
mise settings set jobs $(nproc)
```

### Caching with Sources/Outputs

Best practices:
1. Always specify `sources` for tasks that read files
2. Always specify `outputs` for tasks that write files
3. Exclude irrelevant files (tests, docs) with `!` patterns
4. Use sentinel files for non-file operations:

```toml
[tasks.install]
description = "Install dependencies"
sources = ["package.json", "package-lock.json"]
outputs = ["node_modules/.installed"]  # Sentinel file
run = [
    "npm install",
    "touch node_modules/.installed"
]
```

### Task Granularity

**Too fine-grained (slow):**

```toml
[tasks."lint:file1"]
run = "eslint src/file1.ts"

[tasks."lint:file2"]
run = "eslint src/file2.ts"
# ... hundreds of tasks
```

**Optimal (balanced):**

```toml
[tasks."lint:eslint"]
run = "eslint src/"

[tasks."lint:prettier"]
run = "prettier --check src/"

[tasks.lint]
depends = ["lint:*"]
```

### Profiling Task Execution

```bash
# See timing information
time mise run build

# Enable debug for detailed timing
MISE_DEBUG=1 mise run build 2>&1 | grep -E "took|elapsed"
```

## Common Pitfalls

### Argument Passing Gotchas

Arguments only go to the last command in multi-command tasks:

```toml
[tasks.test]
run = [
    "cargo build",
    "cargo test"  # Args go here only
]
```

Solution: Use separate tasks or explicit arguments:

```toml
[tasks.test]
depends = ["build"]
run = "cargo test"  # Now args work correctly
```

### Circular Dependencies

mise detects circular dependencies:

```toml
[tasks.a]
depends = ["b"]

[tasks.b]
depends = ["a"]  # ERROR: circular dependency
```

Fix by restructuring:

```toml
[tasks.prepare]
run = "echo 'prep'"

[tasks.a]
depends = ["prepare"]

[tasks.b]
depends = ["prepare"]
```

### Missing Dependencies

When tasks run in parallel, ensure dependencies are explicit:

```toml
# WRONG - race condition
[tasks.test]
run = "cargo test"

[tasks.build]
run = "cargo build"

# RIGHT - explicit dependency
[tasks.test]
depends = ["build"]
run = "cargo test"
```

### Environment Variable Scoping

Task-specific env vars don't propagate to dependencies:

```toml
[tasks.a]
env = { FOO = "bar" }
depends = ["b"]  # b doesn't see FOO

[tasks.b]
run = "echo $FOO"  # Empty!
```

Solution: Set in [env] section or pass explicitly:

```toml
[env]
FOO = "bar"

[tasks.a]
depends = ["b"]

[tasks.b]
run = "echo $FOO"  # Works!
```
