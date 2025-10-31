#!/usr/bin/env bash
# SessionStart hook: Inject mise project context at session start
# This helps Claude understand the project immediately

set -euo pipefail

# Timeout protection is handled per-command below
# (Global timeout wrapper removed as it breaks stdin piping)

# Read hook input from stdin with timeout
if command -v timeout &> /dev/null; then
    hook_input=$(timeout 1s cat 2>/dev/null || echo "{}")
else
    hook_input=$(cat 2>/dev/null || echo "{}")
fi

# Validate we got valid input
if [ -z "$hook_input" ] || [ "$hook_input" = "{}" ]; then
    exit 0
fi

# Check if jq is installed (required for JSON parsing)
if ! command -v jq &> /dev/null; then
    echo "⚠️ **Note:** jq is not installed - limited mise context available"
    echo ""
    echo "Install jq for full mise project context:"
    echo "- macOS: \`brew install jq\`"
    echo "- Ubuntu/Debian: \`sudo apt-get install jq\`"
    echo "- Other: https://jqlang.github.io/jq/"
    echo ""
    exit 0
fi

# Extract current working directory with validation
cwd=$(echo "$hook_input" | jq -r '.cwd // empty' 2>/dev/null || echo "")

# Exit if no cwd or invalid JSON
if [ -z "$cwd" ]; then
    exit 0
fi

# Verify cwd is a real directory
if [ ! -d "$cwd" ]; then
    exit 0
fi

# Check if mise is installed
if ! command -v mise &> /dev/null; then
    exit 0
fi

# Verify mise is executable
if ! mise --version &> /dev/null; then
    exit 0
fi

# Check if we're in a mise project (has mise.toml or .mise.toml)
config_file=""
if [ -f "$cwd/mise.toml" ]; then
    config_file="$cwd/mise.toml"
elif [ -f "$cwd/.mise.toml" ]; then
    config_file="$cwd/.mise.toml"
elif [ -f "$cwd/config/mise.toml" ]; then
    config_file="$cwd/config/mise.toml"
fi

# Exit if no mise config found
if [ -z "$config_file" ]; then
    exit 0
fi

# Verify config file is readable
if [ ! -r "$config_file" ]; then
    echo "⚠️ **Warning:** mise configuration file exists but is not readable: $config_file"
    echo ""
    exit 0
fi

echo "## Mise Project Detected"
echo ""
echo "This project uses mise for tool and task management."
echo ""

# Show mise.toml location and content
echo "### Configuration ($config_file)"
echo ""

# Validate TOML syntax before displaying
if command -v mise &> /dev/null && mise config ls &> /dev/null; then
    # Config is valid according to mise
    echo '```toml'
    cat "$config_file" 2>/dev/null || echo "# Unable to read configuration file"
    echo '```'
else
    # Config might be malformed
    echo "_⚠️ Configuration file may have errors. Run \`mise doctor\` to check._"
    echo ""
    echo '```toml'
    cat "$config_file" 2>/dev/null || echo "# Unable to read configuration file"
    echo '```'
fi
echo ""

# Show installed tools with versions
echo "### Tools"
echo ""

# Run mise ls with timeout and error handling
if command -v timeout &> /dev/null; then
    tools_json=$(timeout 2s mise ls --json 2>/dev/null || echo "[]")
else
    tools_json=$(mise ls --json 2>/dev/null || echo "[]")
fi

# Validate JSON output
if ! echo "$tools_json" | jq empty 2>/dev/null; then
    echo "_Unable to retrieve tool information (invalid JSON response)_"
elif [ "$tools_json" = "[]" ] || [ -z "$tools_json" ]; then
    echo "_No tools configured_"
else
    # Parse with better error handling and support for missing .active field
    tools_output=$(echo "$tools_json" | jq -r '
        to_entries[] |
        .key as $tool |
        .value[] |
        select(.active == true or .active == null or (.active | type == "null")) |
        "- **\($tool)@\(.version)** (from \(.source.path | split("/") | .[-1]))"
    ' 2>/dev/null)

    if [ -z "$tools_output" ]; then
        echo "_No active tools found_"
    else
        echo "$tools_output"
    fi
fi
echo ""

# Show available tasks with descriptions
echo "### Tasks"
echo ""

# Run mise tasks with timeout and error handling
if command -v timeout &> /dev/null; then
    tasks_json=$(timeout 2s mise tasks ls --json 2>/dev/null || echo "[]")
else
    tasks_json=$(mise tasks ls --json 2>/dev/null || echo "[]")
fi

# Validate JSON output
if ! echo "$tasks_json" | jq empty 2>/dev/null; then
    echo "_Unable to retrieve task information (invalid JSON response)_"
elif [ "$tasks_json" = "[]" ] || [ -z "$tasks_json" ]; then
    echo "_No tasks defined_"
else
    # Parse with error handling
    tasks_output=$(echo "$tasks_json" | jq -r '
        .[] |
        "- **\(.name)**: \(.description // "_no description_")"
    ' 2>/dev/null)

    if [ -z "$tasks_output" ]; then
        echo "_Unable to parse task information_"
    else
        echo "$tasks_output"
    fi
fi
echo ""

# Check for any issues with mise doctor
echo "### Health Check"
echo ""

# Run mise doctor with timeout
if command -v timeout &> /dev/null; then
    doctor_output=$(timeout 2s mise doctor 2>&1 || echo "")
else
    doctor_output=$(mise doctor 2>&1 || echo "")
fi

# Check if doctor ran successfully
if [ -z "$doctor_output" ]; then
    echo "_Unable to run health check_"
elif echo "$doctor_output" | grep -qE "WARN|ERROR"; then
    echo "⚠️ **Issues found:**"
    echo ""
    echo '```'
    # Extract only WARN and ERROR lines, with fallback
    issue_lines=$(echo "$doctor_output" | grep -E "WARN|ERROR" 2>/dev/null || echo "")
    if [ -z "$issue_lines" ]; then
        echo "Issues present but couldn't extract details"
    else
        echo "$issue_lines"
    fi
    echo '```'
    echo ""
    echo "_Run \`mise doctor\` for full details._"
else
    echo "✅ No issues detected"
fi
echo ""

echo "💡 **Tip:** Use the \`mise-workflow\` skill to work with this project's tools and tasks."
echo ""

exit 0
