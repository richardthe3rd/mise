# Changelog

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
