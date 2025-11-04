# Changelog

## [0.3.0] - 2025-11-04

### Added
- **Task arguments (usage field) support** in SKILL.md
  - Complete examples for TOML and file tasks with arguments
  - KDL syntax documentation for `arg` and `flag` definitions
  - Bash expansion patterns: `${usage_var?}`, `${usage_var:-default}`, `${usage_var:+value}`
  - Enables creating professional tasks with automatic `--help` generation
- **Tera→usage migration guide** in adding-capabilities.md
  - 5 comprehensive migration examples covering all patterns
  - Migration checklist and quick find/replace patterns
  - Background on why Tera is deprecated (removal 2026.11.0)

### Improved
- **Context efficiency**: Streamlined SKILL.md from 206 → 148 lines (28% reduction)
- **Progressive disclosure optimization**: Transformed all reference files to advanced-only content
  - understanding-projects.md: Now focuses on config hierarchy, advanced version specs, edge cases
  - running-commands.md: Advanced dependency control, wildcards, complex orchestration
  - adding-capabilities.md: Advanced tool options, conditional installation, monorepo patterns
- **Documentation clarity**: References now truly optional, loaded only for complex scenarios
- **README**: Updated progressive disclosure section to reflect new architecture

### Technical Details
- SKILL.md is now self-sufficient for all basic workflows
- Reference files contain only advanced topics and edge cases
- Total reduction: ~40% in essential context size while maintaining comprehensive coverage

## [0.2.0] - 2025-11-03

### Added
- **`/mise-en-place:detect` command** for on-demand project detection
  - Shows project root from `mise://config` MCP resource
  - Displays configuration file hierarchy (project → user → system)
  - Lists first 8 active tools with total count
  - Shows first 8-10 key development tasks with total count
  - Includes full mise.toml configuration content
  - Fast execution using MCP resources only

### Improved
- **Performance optimization**: Removed slow `mise doctor` check from detect command
- **Efficiency**: Limited tool/task lists to most relevant items with counts
- **Usability**: More scannable, concise output format
- **Reliability**: Fixed MCP server name to use "mise-en-place" correctly

### Fixed
- MCP server name in detect command (was "mise", now "mise-en-place")

## [0.1.0] - 2025-10-31

### Added
- **mise-workflow skill** for active development workflows
  - Understanding projects workflow
  - Running commands workflow
  - Adding capabilities workflow
- **SessionStart hook** with automatic project detection
  - Injects mise.toml configuration
  - Shows active tools and versions
  - Displays available tasks
  - Reports health check warnings
- **Comprehensive reference documentation**
  - understanding-projects.md (785 lines)
  - running-commands.md (645 lines)
  - adding-capabilities.md (953 lines)
- **Enhanced error handling** in SessionStart hook
  - Timeout protection (5s global, 2s per command)
  - jq dependency check with helpful install message
  - JSON validation before parsing
  - Directory existence verification
  - Config file readability check
  - TOML syntax validation
  - Graceful degradation on errors

### Improved
- Plugin description to mention new features
- README with usage examples and troubleshooting
- Error messages now user-friendly with recovery suggestions

### Technical Enhancements
- Progressive disclosure architecture (SKILL: 477 lines, References: 2,383 lines)
- All mise JSON commands wrapped with timeout
- Robust fallback handling for missing fields
- Support for missing .active field in tool JSON
- Better error messages for malformed JSON responses

## [0.0.1] - 2025-10-29

### Added
- Initial MCP server integration
- Basic plugin manifest
- README documentation
