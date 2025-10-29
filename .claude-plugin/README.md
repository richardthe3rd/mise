# Claude Code Plugin Marketplace

This directory contains the marketplace configuration for Claude Code plugins in the mise repository.

## Purpose

The `marketplace.json` file in this directory enables users to install mise Claude Code plugins using the shorthand syntax:

```bash
/plugin marketplace add jdx/mise
```

When users add this marketplace, Claude Code reads `marketplace.json` to discover available plugins.

## How It Works

1. **Marketplace Registration**: Users run `/plugin marketplace add jdx/mise`
2. **Discovery**: Claude Code reads `.claude-plugin/marketplace.json` from the repo root
3. **Plugin Resolution**: The marketplace points to plugins via the `source` field
4. **Installation**: Users can then run `/plugin install mise-en-place@mise-en-place` to install the plugin

## Files

- **`marketplace.json`** - Marketplace registry defining available plugins

## Plugin Location

The actual plugin is located at:
```
claude-plugins/mise/.claude-plugin/plugin.json
```

The marketplace references it with:
```json
{
  "source": "./claude-plugins/mise"
}
```

## Validation

To validate the marketplace configuration:

```bash
claude plugin validate .
```

This should be run from the repository root.

## More Information

- See `claude-plugins/README.md` for full documentation
- Plugin documentation: `claude-plugins/mise/README.md`
- [Claude Code Plugin Marketplaces](https://docs.claude.com/en/docs/claude-code/plugin-marketplaces)
