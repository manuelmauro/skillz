# skillz

A CLI tool for [Agent Skills](https://agentskills.io/specification) development.

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/manuelmauro/skillz.git
cd skillz

# Install using make
make install

# Or directly with cargo
cargo install --path .
```

### Requirements

- Rust 1.92.0 (pinned in `rust-toolchain.toml`)

## Usage

### Create a new skill

```bash
# Create a skill with the default template (hello-world)
skillz new my-skill

# Use a specific template
skillz new my-skill --template minimal

# Specify the script language
skillz new my-skill --lang python

# Add a description and license
skillz new my-skill --description "My awesome skill" --license MIT
```

**Available templates:**
- `hello-world` (default) - A minimal working skill with a greeting script
- `minimal` - Bare-bones skill with only SKILL.md
- `full` - Complete skill with all optional directories
- `script-based` - Skill focused on script execution

**Supported languages:** `python`, `bash`, `javascript`, `typescript`

### Validate skills

```bash
# Lint a skill directory
skillz lint path/to/skill

# Lint all skills in current directory
skillz lint .

# Strict mode (treat warnings as errors)
skillz lint --strict .

# Auto-fix simple issues
skillz lint --fix .
```

### Format skills

```bash
# Format SKILL.md files
skillz fmt path/to/skill

# Check formatting without modifying
skillz fmt --check .

# Show diff of changes
skillz fmt --diff .
```

### Run all checks

```bash
# Run lint + format check (ideal for CI)
skillz check .
```

### Output formats

All commands support multiple output formats:

```bash
skillz lint --format text .   # Human-readable (default)
skillz lint --format json .   # JSON output
skillz lint --format sarif .  # SARIF for code scanning integrations
```

## Skill Structure

A valid skill follows this structure:

```
my-skill/
├── SKILL.md           # Required: manifest with YAML frontmatter
├── scripts/           # Optional: executable scripts
├── references/        # Optional: additional documentation
└── assets/            # Optional: static resources
```

### SKILL.md Format

```markdown
---
name: my-skill
description: A brief description of what the skill does.
license: MIT
---

# My Skill

Detailed documentation goes here.
```

## Configuration

Create a `.skillzrc.toml` file for project-specific settings:

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

## Exit Codes

| Code | Meaning                            |
| ---- | ---------------------------------- |
| 0    | Success                            |
| 1    | Validation errors found            |
| 2    | Invalid arguments or configuration |
| 3    | I/O error                          |

## License

MIT OR Apache-2.0
