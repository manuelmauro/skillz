# ADR 0001: Skilo CLI Tool Architecture

## Status

Proposed

## Context

The [Agent Skills Specification](https://agentskills.io/specification) defines a standard format for creating reusable skill packages that AI agents can discover and execute. Skills consist of a `SKILL.md` manifest with YAML frontmatter and optional supporting directories (`scripts/`, `references/`, `assets/`).

Developers creating and maintaining skills need tooling to:

- Scaffold new skills with the correct structure
- Validate skills against the specification
- Format skills consistently
- Integrate validation into CI/CD pipelines

Currently, there is a `skills-ref` library mentioned in the specification for validation, but no comprehensive CLI tool exists to support the full skill development lifecycle.

## Decision

We will build **skilo**, a command-line tool written in Rust that provides comprehensive tooling for Agent Skills development.

### Core Commands

| Command | Description |
|---------|-------------|
| `skilo new <name>` | Scaffold a new skill with required structure |
| `skilo lint [path]` | Validate skill(s) against the specification |
| `skilo fmt [path]` | Format SKILL.md files consistently |
| `skilo check [path]` | Run all validations (lint + format check) |
| `skilo validate [path]` | Alias for `lint` with strict mode |

### Subcommand Details

#### `skilo new <name>`

Creates a new skill directory with:

```
<name>/
├── SKILL.md           # Manifest with frontmatter template
├── scripts/           # Optional: executable scripts
├── references/        # Optional: additional documentation
└── assets/            # Optional: static resources
```

Options:
- `--template <template>` - Use a predefined template (default: hello-world)
- `--lang <language>` - Preferred language for scripts: python, bash, javascript, typescript (default: python)
- `--license <license>` - Set the license field (default: none)
- `--description <desc>` - Set the description field
- `--no-optional-dirs` - Skip creating optional directories
- `--no-scripts` - Skip creating scripts directory and example script

Supported script languages:
| Language | Extension | Shebang |
|----------|-----------|---------|
| python | `.py` | `#!/usr/bin/env python3` |
| bash | `.sh` | `#!/usr/bin/env bash` |
| javascript | `.js` | `#!/usr/bin/env node` |
| typescript | `.ts` | `#!/usr/bin/env -S npx ts-node` |

#### `skilo lint [path]`

Validates skills against the Agent Skills Specification:

**Frontmatter Validation:**
- `name`: 1-64 chars, lowercase alphanumeric + hyphens, no leading/trailing/consecutive hyphens
- `name` matches parent directory name
- `description`: 1-1024 chars, present and non-empty
- `license`: Valid SPDX identifier or file reference (if present)
- `compatibility`: Max 500 chars (if present)
- `metadata`: Valid key-value pairs (if present)
- `allowed-tools`: Space-delimited list format (if present)

**Structure Validation:**
- `SKILL.md` exists and is valid Markdown
- Body content is under 500 lines (warning if exceeded)
- File references use relative paths from skill root
- Referenced files exist

**Script Validation:**
- Scripts in `scripts/` are executable
- Scripts have proper shebang lines
- No obvious syntax errors (optional, language-specific)

Options:
- `--strict` - Treat warnings as errors
- `--format <format>` - Output format: text, json, sarif (default: text)
- `--fix` - Auto-fix simple issues where possible

#### `skilo fmt [path]`

Formats SKILL.md files for consistency:

- Normalizes YAML frontmatter formatting
- Ensures consistent spacing between frontmatter and body
- Standardizes Markdown formatting (headings, lists, code blocks)
- Sorts frontmatter keys in canonical order

Options:
- `--check` - Check formatting without modifying files (exit 1 if changes needed)
- `--diff` - Show diff of changes

#### `skilo check [path]`

Runs comprehensive validation suitable for CI:

- Executes `lint --strict`
- Executes `fmt --check`
- Returns non-zero exit code on any failure

Options:
- `--format <format>` - Output format: text, json, sarif

### Configuration

Supports optional `.skilorc.toml` configuration file:

```toml
[lint]
strict = true
max_body_lines = 500

[fmt]
sort_frontmatter = true
indent_size = 2

[new]
default_license = "MIT"
default_template = "hello-world"
default_lang = "python"
```

### Built-in Templates

#### `hello-world` (default)

A minimal working skill that demonstrates the basic structure with a simple greeting script.

**Generated structure:**
```
<name>/
├── SKILL.md
└── scripts/
    └── greet.{ext}
```

**SKILL.md:**
```markdown
---
name: <name>
description: A simple hello world skill that greets the user.
---

# <Name>

This skill provides a simple greeting functionality.

## Usage

Run the greeting script to display a personalized message.

## Scripts

- `scripts/greet.{ext}` - Outputs a greeting message
```

**scripts/greet.py (Python):**
```python
#!/usr/bin/env python3
"""A simple greeting script."""

import sys

def main():
    name = sys.argv[1] if len(sys.argv) > 1 else "World"
    print(f"Hello, {name}!")

if __name__ == "__main__":
    main()
```

**scripts/greet.sh (Bash):**
```bash
#!/usr/bin/env bash
# A simple greeting script.

set -euo pipefail

name="${1:-World}"
echo "Hello, ${name}!"
```

**scripts/greet.js (JavaScript):**
```javascript
#!/usr/bin/env node
// A simple greeting script.

const name = process.argv[2] || "World";
console.log(`Hello, ${name}!`);
```

**scripts/greet.ts (TypeScript):**
```typescript
#!/usr/bin/env -S npx ts-node
// A simple greeting script.

const name: string = process.argv[2] || "World";
console.log(`Hello, ${name}!`);
```

#### `minimal`

Bare-bones skill with only the required SKILL.md file.

#### `full`

Complete skill with all optional directories and example files in each.

#### `script-based`

Skill focused on script execution with multiple example scripts.

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Validation errors found |
| 2 | Invalid arguments or configuration |
| 3 | I/O error (file not found, permission denied) |

### Design Principles

1. **Fail Fast**: Report all errors in a single run, not one at a time
2. **Clear Messages**: Error messages include file path, line number, and fix suggestion
3. **CI-Friendly**: Machine-readable output formats, meaningful exit codes
4. **Zero Config**: Works out of the box with sensible defaults
5. **Incremental Adoption**: Each command is useful independently

## Consequences

### Positive

- Standardized skill development workflow
- Early detection of specification violations
- Consistent formatting across skill repositories
- Easy CI/CD integration
- Single tool for all skill-related operations

### Negative

- Another tool to install and maintain
- Rust dependency for builds (mitigated by pre-built binaries)
- Must track specification changes

### Neutral

- Complements rather than replaces manual skill authoring
- Does not handle skill distribution or registry operations

## Implementation Notes

### Dependencies

- `clap` - CLI argument parsing
- `serde` + `serde_yaml` - YAML frontmatter parsing
- `pulldown-cmark` - Markdown parsing
- `walkdir` - Directory traversal
- `colored` - Terminal output formatting
- `thiserror` - Error handling
- `toml` - Configuration file parsing

### Crate Structure

```
src/
├── main.rs           # Entry point, CLI setup
├── commands/
│   ├── mod.rs
│   ├── new.rs        # Scaffold command
│   ├── lint.rs       # Validation command
│   ├── fmt.rs        # Format command
│   └── check.rs      # Combined check command
├── skill/
│   ├── mod.rs
│   ├── manifest.rs   # SKILL.md parsing
│   ├── frontmatter.rs # YAML frontmatter types
│   └── validator.rs  # Validation logic
├── templates/
│   ├── mod.rs        # Template registry and rendering
│   ├── hello_world.rs
│   ├── minimal.rs
│   ├── full.rs
│   └── script_based.rs
├── lang.rs           # Script language definitions
├── config.rs         # Configuration handling
└── output.rs         # Output formatting (text, json, sarif)
```

## References

- [Agent Skills Specification](https://agentskills.io/specification)
- [SARIF Output Format](https://sarifweb.azurewebsites.net/)
- [SPDX License List](https://spdx.org/licenses/)
