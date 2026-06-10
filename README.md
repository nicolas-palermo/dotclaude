# dotclaude

A [Claude Code](https://claude.com/claude-code) plugin marketplace for a **harness-agents style development process** — a set of plugins that drive development through coordinated agents, skills and lifecycle hooks.

> Names (`dotclaude` marketplace and the first plugin) are provisional and will be finalized later.

## Structure

```
dotclaude/
├── .claude-plugin/
│   └── marketplace.json     # Marketplace manifest (lists all plugins)
├── plugins/                 # One subdirectory per plugin
│   └── <plugin-name>/
│       ├── .claude-plugin/
│       │   └── plugin.json   # Plugin manifest
│       ├── agents/           # Sub-agent definitions
│       ├── skills/           # Packaged skills
│       └── hooks/
│           └── hooks.json    # Lifecycle hooks
└── README.md
```

## Install

Add the marketplace, then install a plugin from it:

```bash
/plugin marketplace add NicolasPalermo/dotclaude
/plugin install <plugin-name>@dotclaude
```

While developing locally, point the marketplace at this checkout:

```bash
/plugin marketplace add /Users/galina/work/dotclaude
```

## Plugins

_None yet — the first plugin is in progress._
