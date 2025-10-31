# Running Commands with Mise

Complete guide to executing tasks and commands with mise-managed tools.

## Quick Reference

```bash
# Run tasks
mise run <task>              # Run a task
mise r <task>                # Short alias
mise <task>                  # Direct shorthand (avoid in scripts)
mise run task1 task2         # Run multiple tasks in order
mise run build -- --release  # Pass arguments to task

# Task management
mise tasks                   # List all tasks
mise tasks ls --json         # List tasks with full details
mise tasks info <task>       # Show task configuration
mise tasks deps <task>       # Show task dependencies

# Execute with mise environment
mise exec -- command args    # Execute command with mise tools
mise x -- command args       # Short alias

# Watch and rebuild
mise watch <task>            # Re-run task on file changes

# Advanced
mise run --jobs 8 test       # Run with 8 parallel jobs
mise run --interleave test   # Show output immediately (not line-buffered)
```

## Running Tasks

### Basic Execution

Run a task defined in mise.toml or mise-tasks/:

```bash
mise run build
```

This will:
1. Load the mise environment (tools, env vars)
2. Resolve task dependencies
3. Execute the task with the mise environment

### Task Aliases

There are multiple ways to run tasks:

```bash
mise run build    # Full form (recommended for scripts)
mise r build      # Short form
mise build        # Direct form (avoid in scripts/docs)
```

**Important:** The direct form `mise build` can be shadowed if mise adds a `build` command in the future. Always use `mise run build` in scripts and documentation.

### Passing Arguments

Pass arguments to tasks after `--`:

```bash
# Single task with args
mise run build -- --release --verbose

# Args go to the last command in multi-command tasks
mise run test -- --filter=integration
```

If a task has multiple commands, arguments only pass to the last one:

```toml
[tasks.build]
run = [
    "cargo clean",
    "cargo build"  # Args go here
]
```

To pass arguments to earlier commands, define them explicitly in the task or use separate tasks.

### Running Multiple Tasks

Run tasks in sequence:

```bash
# Run lint, then test, then build
mise run lint test build
```

Tasks run in the order specified. If any task fails, execution stops.

Use the `:::` delimiter to pass different arguments to different tasks:

```bash
mise run build arg1 arg2 ::: test arg3 arg4
# Runs: build with [arg1, arg2], then test with [arg3, arg4]
```

### Task Dependencies

Tasks can declare dependencies that run first:

```toml
[tasks.test]
description = "Run tests"
depends = ["build"]  # Runs build first
run = "cargo test"

[tasks.build]
description = "Build the project"
run = "cargo build"
```

When you run `mise run test`:
1. Mise checks dependencies: `test` depends on `build`
2. Runs `build` first
3. If `build` succeeds, runs `test`

Dependencies run in parallel when possible:

```toml
[tasks.ci]
depends = ["lint", "test", "check-types"]  # All 3 run in parallel
```

### Advanced Dependency Control

Three types of dependencies:

**`depends`** - Run before this task (parallel if possible):
```toml
[tasks.deploy]
depends = ["build", "test"]  # Both run in parallel before deploy
```

**`depends_post`** - Run after this task:
```toml
[tasks.build]
depends_post = ["notify"]  # Runs notify after build completes
```

**`wait_for`** - Don't start until these finish, but don't add as dependencies:
```toml
[tasks.test]
wait_for = ["db-setup"]  # If db-setup is running, wait for it, but don't start it
```

### Wildcards in Task Names

Use glob patterns to match multiple tasks:

```bash
# Run all test tasks
mise run test:*

# Run all tasks in test namespace
mise run test:**

# Run specific pattern
mise run generate:{completions,docs:*}
```

Wildcard patterns:
- `?` - Match any single character
- `*` - Match 0+ characters
- `**` - Match 0+ groups (separated by `:`)
- `{a,b,c}` - Match any of the alternatives
- `[abc]` - Match any character in set
- `[!abc]` - Match any character not in set

Example with dependencies:

```toml
[tasks."lint:eslint"]
run = "eslint ."

[tasks."lint:prettier"]
run = "prettier --check ."

[tasks.lint]
depends = ["lint:*"]  # Runs all lint:* tasks
```

## Task Output and Parallelism

### Parallel Execution

By default, mise runs up to 4 tasks in parallel:

```bash
# Use default (4 parallel jobs)
mise run test

# Custom parallelism
mise run --jobs 8 test

# Sequential execution (one at a time)
mise run --jobs 1 test
```

Set globally:
```bash
export MISE_JOBS=8
# or
mise settings set jobs 8
```

### Output Modes

**Line-buffered (default):**
```bash
mise run test
# Output:
# [test:unit] Running unit tests...
# [test:integration] Running integration tests...
```

Each line is prefixed with the task name to avoid interleaving output.

**Interleaved (raw output):**
```bash
mise run --interleave test
# Output appears immediately without prefixes
```

Use interleaved when:
- Running a single task (`--jobs 1`)
- Need interactive output
- Debugging task execution

Set globally:
```bash
export MISE_TASK_OUTPUT=interleave
# or
mise settings set task_output interleave
```

### Standard Input

By default, tasks don't read from stdin. To enable:

```toml
[tasks.interactive]
description = "Interactive task that needs stdin"
raw = true  # Enable stdin
run = "read -p 'Enter name: ' name && echo Hello $name"
```

When `raw = true`:
- Task receives stdin
- Runs exclusively (no parallel execution with other tasks)
- Output redactions are disabled

## Watching Files

### Using mise watch

Automatically re-run tasks when files change:

```bash
# Watch and re-run on any change
mise watch build

# Watch specific task with args
mise watch test -- --verbose
```

Requirements:
- Install `watchexec`: `mise use -g watchexec@latest`

When files change, mise will:
1. Detect the change
2. Wait for file system to settle
3. Re-run the task

### File-based Task Triggering

Define sources and outputs to skip unnecessary runs:

```toml
[tasks.build]
description = "Build the CLI"
run = "cargo build"
sources = ['Cargo.toml', 'src/**/*.rs']
outputs = ['target/debug/mycli']
```

Mise compares timestamps:
- If outputs are newer than sources, skips task
- If sources changed, runs task
- On first run or missing outputs, always runs

Example workflow:

```bash
mise run build  # Runs (no output exists)
mise run build  # Skips (output newer than sources)
# Edit src/main.rs
mise run build  # Runs (source changed)
```

## Executing Commands

### mise exec

Run arbitrary commands with mise environment:

```bash
# Execute with mise tools
mise exec -- node --version

# Execute with specific tool versions
mise exec -- python script.py

# Execute in different directory
mise --cd /path/to/project exec -- go build
```

Short alias:
```bash
mise x -- command args
```

### Use Cases

**Test a command before adding a task:**
```bash
mise exec -- npm run build
# Works? Add it as a task:
# mise tasks add build npm run build
```

**Run with specific environment:**
```bash
MISE_ENV=production mise exec -- ./deploy.sh
```

**Debug tool availability:**
```bash
mise exec -- which node
mise exec -- which python
```

## Task Configuration Deep Dive

### Task Definition Options

Full TOML task configuration:

```toml
[tasks.example]
description = "Task description (REQUIRED)"
alias = "ex"                    # Alternative name
depends = ["other-task"]        # Dependencies
depends_post = ["cleanup"]      # Post-dependencies
wait_for = ["db"]               # Wait without dependency
run = "echo 'Hello'"            # Command to run
run = ["cmd1", "cmd2"]          # Multiple commands
dir = "{{config_root}}/sub"     # Working directory
env = { VAR = "value" }         # Environment variables
sources = ["src/**/*.ts"]       # Input files
outputs = ["dist/**/*.js"]      # Output files
hide = false                    # Hide from task list
raw = false                     # Enable stdin
quiet = false                   # Suppress output
shell = "bash -c"               # Shell to use (default)
```

### Task Execution Context

Environment variables available in tasks:

```bash
# Mise variables
MISE_PROJECT_ROOT      # Root of the project
MISE_CONFIG_ROOT       # Directory containing mise.toml
MISE_ORIGINAL_CWD      # Directory where mise run was called
MISE_TASK_NAME         # Name of the current task
MISE_TASK_DIR          # Directory containing task script (file tasks)
MISE_TASK_FILE         # Full path to task script (file tasks)

# User-defined
# From [env] section in mise.toml
# From task's env field
```

Example using context:

```toml
[tasks.clean]
description = "Clean build artifacts"
run = "rm -rf {{config_root}}/dist"

[tasks.test]
description = "Run tests"
env = { TEST_ENV = "test" }
run = "npm test"
```

## Task Grouping and Organization

### Semantic Grouping

Use `:` to namespace tasks:

```toml
[tasks."test:unit"]
description = "Run unit tests"
run = "npm run test:unit"

[tasks."test:integration"]
description = "Run integration tests"
run = "npm run test:integration"

[tasks."test:e2e"]
description = "Run e2e tests"
run = "npm run test:e2e"

[tasks.test]
description = "Run all tests"
depends = ["test:*"]
```

Run specific groups:

```bash
mise run test:*           # All test tasks
mise run test:unit        # Just unit tests
mise run test:**:local    # All nested tasks ending in :local
```

### Hidden Tasks

Hide implementation details:

```toml
[tasks."_internal"]
description = "Internal helper task"
hide = true
run = "echo 'Internal'"

[tasks.public]
description = "Public task"
depends = ["_internal"]
run = "echo 'Public'"
```

Hidden tasks:
- Don't appear in `mise tasks` by default
- Show with `mise tasks --hidden`
- Still usable as dependencies

## Debugging Task Execution

### Enable Debug Output

```bash
# Basic debugging
MISE_DEBUG=1 mise run build

# Verbose tracing
MISE_TRACE=1 mise run build

# Both show:
# - Task resolution
# - Dependency order
# - Command execution
# - Environment setup
```

### Common Issues

**Task not found:**
```bash
mise tasks              # List all tasks
mise tasks --hidden     # Include hidden tasks
mise tasks info <name>  # Show task details
```

**Task fails:**
```bash
# Check task definition
mise tasks info build

# Test command directly
mise exec -- <command>

# Check dependencies
mise tasks deps build

# Enable debug output
MISE_DEBUG=1 mise run build
```

**Wrong environment:**
```bash
# Check environment variables
mise env

# Check tool versions
mise ls

# Verify tools are in PATH
mise exec -- which node
```

**Dependency issues:**
```bash
# Show dependency tree
mise tasks deps <task>

# Check for circular dependencies
# mise will error if detected

# Run dependencies manually
mise run <dependency>
```

### Testing Tasks

Before committing a task:

```bash
# 1. Format configuration
mise fmt

# 2. Check for issues
mise doctor

# 3. Verify task has description
mise tasks ls --json | jq '.[] | select(.name == "mytask")'

# 4. Test execution
mise run mytask

# 5. Test with args
mise run mytask -- --verbose

# 6. Test dependencies
mise tasks deps mytask
mise run <dependency>
```

## Advanced Patterns

### Conditional Task Execution

Use shell conditionals in tasks:

```toml
[tasks.build]
description = "Build (skip if no changes)"
run = '''
if [ -f dist/app ] && [ dist/app -nt src/main.ts ]; then
  echo "Skipping build (no changes)"
else
  npm run build
fi
'''
```

Or use sources/outputs (preferred):

```toml
[tasks.build]
description = "Build (skip if no changes)"
sources = ["src/**/*.ts"]
outputs = ["dist/app"]
run = "npm run build"
```

### Task Composition

Compose complex workflows:

```toml
[tasks.ci]
description = "Run CI pipeline"
depends = ["lint", "test", "build"]  # Parallel

[tasks.deploy]
description = "Deploy application"
depends = ["ci"]  # Wait for CI
run = ["mise run docker:build", "mise run docker:push"]
```

### Multi-Environment Tasks

```toml
[tasks.deploy:staging]
description = "Deploy to staging"
env = { ENV = "staging" }
run = "./deploy.sh"

[tasks.deploy:production]
description = "Deploy to production"
env = { ENV = "production" }
run = "./deploy.sh"
```

Or use `MISE_ENV`:

```bash
MISE_ENV=production mise run deploy
```

### Integration with External Tools

```toml
[tasks.docker:build]
description = "Build Docker image"
run = "docker build -t myapp:{{env.VERSION}} ."

[tasks.docker:push]
description = "Push Docker image"
depends = ["docker:build"]
run = "docker push myapp:{{env.VERSION}}"

[tasks.k8s:deploy]
description = "Deploy to Kubernetes"
depends = ["docker:push"]
run = "kubectl apply -f k8s/"
```

## Performance Tips

1. **Use parallelism:** Set `MISE_JOBS` appropriately for your system
2. **Leverage sources/outputs:** Avoid unnecessary rebuilds
3. **Group related tasks:** Use dependencies to model relationships
4. **Use file tasks for complex logic:** Better performance than inline shell scripts
5. **Cache expensive operations:** Use mise's built-in caching with sources/outputs
6. **Profile task execution:** Use `MISE_DEBUG=1` to see timings

## Best Practices

1. **Always add descriptions:** Required and helps with documentation
2. **Use task names consistently:** Namespace with `:` for organization
3. **Model dependencies explicitly:** Makes workflows clear
4. **Test tasks after creation:** Run them to ensure they work
5. **Use `mise run` in scripts:** Avoid direct task invocation
6. **Document complex tasks:** Add comments in TOML or task files
7. **Keep tasks focused:** One task, one purpose; compose with dependencies
