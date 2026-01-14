---
name: use-skilo
description: Creates skills from templates, validates against specification, and formats SKILL.md files. Use when developing, linting, or formatting Agent Skills.
---

# Use Skilo

Skilo is a CLI tool for developing [Agent Skills](https://agentskills.io/specification).

## Installation

```bash
# From crates.io
cargo install skilo

# From source
cargo install --path .
```

## Commands

| Command                 | Description                      |
|-------------------------|----------------------------------|
| `skilo new`             | Create a new skill from template |
| `skilo lint`            | Validate skills against spec     |
| `skilo fmt`             | Format SKILL.md files            |
| `skilo check`           | Run lint + format check          |
| `skilo validate`        | Alias for `lint --strict`        |
| `skilo read-properties` | Output skill metadata as JSON    |
| `skilo to-prompt`       | Generate XML for agent prompts   |

## Creating Skills

```bash
# Basic usage
skilo new my-skill

# With options
skilo new my-skill \
  --description "What the skill does" \
  --lang python \
  --license MIT \
  --template hello-world

# Output to specific directory
skilo new my-skill -o ./skills/
```

**Templates:** `hello-world` (default), `minimal`, `full`, `script-based`

**Languages:** `python`, `bash`, `javascript`, `typescript`

## Validating Skills

```bash
# Lint a single skill
skilo lint path/to/skill

# Lint all skills in directory
skilo lint .

# Strict mode (warnings as errors)
skilo lint --strict .

# JSON output
skilo lint --format json .

# SARIF output (for CI integrations)
skilo lint --format sarif . > results.sarif
```

## Formatting Skills

```bash
# Format in place
skilo fmt .

# Check only (for CI)
skilo fmt --check .

# Show diff
skilo fmt --diff .
```

## Reading Skill Properties

Extract skill metadata as JSON:

```bash
# Single skill (outputs JSON object)
skilo read-properties path/to/skill

# Multiple skills (outputs JSON array)
skilo read-properties path/to/skills/

# Multiple paths
skilo read-properties skill-a skill-b
```

Output fields: `name`, `description`, `license`, `compatibility`, `metadata`, `allowed_tools`, `path`.

## Generating Agent Prompts

Generate `<available_skills>` XML for agent system prompts:

```bash
# Single skill
skilo to-prompt path/to/skill

# All skills in directory
skilo to-prompt path/to/skills/
```

Output:
```xml
<available_skills>
  <skill>
    <name>my-skill</name>
    <description>What the skill does</description>
    <location>path/to/my-skill/SKILL.md</location>
  </skill>
</available_skills>
```

## CI Integration

### GitHub Actions

```yaml
name: Validate Skills

on:
  push:
    paths: ['.claude/skills/**']
  pull_request:
    paths: ['.claude/skills/**']

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2

      - name: Install skilo
        run: cargo install skilo@0.4.0

      - name: Lint skills
        run: skilo lint .claude/skills/

      - name: Check formatting
        run: skilo fmt --check .claude/skills/
```

### SARIF Integration

Upload results to GitHub Code Scanning:

```yaml
- name: Run skilo lint
  run: skilo lint --format sarif . > results.sarif
  continue-on-error: true

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```

### Quick CI Check

Use `skilo check` to run both lint and format check in one command:

```yaml
- name: Validate skills
  run: skilo check --strict .claude/skills/
```

## Configuration

Create `.skilorc.toml` in your project:

```toml
[lint]
strict = false

[lint.rules]
name_format = true        # E001
name_length = 64          # E002 (threshold)
name_directory = true     # E003
description_required = true  # E004
description_length = 1024    # E005 (threshold)
compatibility_length = 500   # E006 (threshold)
references_exist = true      # E009
body_length = 500            # W001 (threshold)
script_executable = true     # W002
script_shebang = true        # W003

[fmt]
sort_frontmatter = true
indent_size = 2
format_tables = true

[new]
default_license = "MIT"
default_template = "hello-world"
default_lang = "bash"
```

### Disabling Rules

```toml
[lint.rules]
name_directory = false     # Disable rule
body_length = false        # Disable threshold rule
description_length = 2048  # Custom threshold
```

## Skill Structure

```
my-skill/
├── SKILL.md        # Required: manifest with YAML frontmatter
├── scripts/        # Optional: executable scripts
├── references/     # Optional: additional docs
└── assets/         # Optional: static resources
```

## SKILL.md Format

```markdown
---
name: my-skill
description: What the skill does
license: MIT
---

# My Skill

Documentation goes here.
```

## Exit Codes

| Code | Meaning                  |
|------|--------------------------|
| 0    | Success                  |
| 1    | Validation errors found  |
| 2    | Invalid arguments/config |
| 3    | I/O error                |
