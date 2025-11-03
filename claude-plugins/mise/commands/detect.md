---
description: Detect and inject mise project context into the conversation
---

Detect and analyze the mise project configuration in the current directory to provide helpful context.

Follow these steps to gather mise project information:

1. **Find the mise configuration file**:
   - Use Glob to search for mise configuration files: `mise.toml`, `.mise.toml`, or `config/mise.toml`
   - If no configuration file is found, report that this is not a mise project and exit

2. **Read and display the configuration**:
   - Read the configuration file content
   - Display it in a markdown code block with the file path

3. **Retrieve project data using MCP resources** (run in parallel):
   - Read `mise://config` resource to get project configuration metadata
   - Read `mise://tools` resource to get installed tools with versions and sources
   - Read `mise://tasks` resource to get available tasks with descriptions
   - Parse the JSON to extract:
     - Project root directory
     - List of all loaded config files (in load order)
     - Count of active tools and filter to show only first 8 from project config
     - Count of available tasks and filter to show only first 8-10 most useful tasks
   - Handle errors gracefully (if resources fail, indicate data is unavailable)

4. **Format the output**:
   Present all information in this markdown structure:
   ```
   ## Mise Project Detected

   **Project Root:** `/path/to/project`

   **Config Files:** N files loaded (project → user → system)
   - `/path/to/project/mise.toml`
   - [additional config files...]

   **Tools:** N tools configured ([X] from project config)
   - tool1@version
   - tool2@version
   - ...up to 8 tools...

   **Tasks:** N tasks available
   - **task-name**: description
   - ...up to 8-10 most useful tasks...

   **Primary Configuration** (path/to/mise.toml):
   ```toml
   [configuration content in code block]
   ```

   💡 Run common tasks with `mise run <task>` or use the `mise-workflow` skill.
   ```

5. **Acknowledge completion**:
   After displaying the information, confirm that you now have the mise project context loaded and are ready to help with tasks and tools.

**Important implementation notes**:
- Use ReadMcpResourceTool with server="mise-en-place" for config, tools, and tasks data
- MCP resources provide structured JSON data (fast and reliable)
- The `mise://config` resource provides `project_root` and `config_files` array
- Filter tools to show only active ones (where "active": true) from the project config file
- Show counts with "X tools configured (Y from project config)" format
- For tasks, prioritize showing common development tasks (build, test, lint, docs, etc.)
- Keep output concise - no need for `mise doctor` or exhaustive lists
- Extract source file name from full paths for cleaner display
- Handle missing or empty data gracefully with appropriate messages
