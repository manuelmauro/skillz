# skilo

A CLI tool for [Agent Skills](https://agentskills.io/specification) development.

## Installation

### From source

```bash
# Clone the repository
git clone https://github.com/manuelmauro/skilo.git
cd skilo

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
skilo new my-skill

# Use a specific template
skilo new my-skill --template minimal

# Specify the script language
skilo new my-skill --lang python

# Add a description and license
skilo new my-skill --description "My awesome skill" --license MIT
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
skilo lint path/to/skill

# Lint all skills in current directory
skilo lint .

# Strict mode (treat warnings as errors)
skilo lint --strict .

# Auto-fix simple issues
skilo lint --fix .
```

### Format skills

```bash
# Format SKILL.md files (includes table alignment)
skilo fmt path/to/skill

# Check formatting without modifying
skilo fmt --check .

# Show diff of changes
skilo fmt --diff .
```

Formatting includes:
- YAML frontmatter normalization
- Markdown table column alignment

### Run all checks

```bash
# Run lint + format check (ideal for CI)
skilo check .
```

### Output formats

All commands support multiple output formats:

```bash
skilo lint --format text .   # Human-readable (default)
skilo lint --format json .   # JSON output
skilo lint --format sarif .  # SARIF for code scanning integrations
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

Create a `.skilorc.toml` file for project-specific settings:

```toml
[lint]
strict = true

[lint.rules]
name_format = true           # E001: Name format validation
name_length = 64             # E002: Name max length (false to disable)
name_directory = true        # E003: Name matches directory
description_required = true  # E004: Description not empty
description_length = 1024    # E005: Description max length (false to disable)
compatibility_length = 500   # E006: Compatibility max length (false to disable)
references_exist = true      # E009: Referenced files exist
body_length = 500            # W001: Max body lines (false to disable)
script_executable = true     # W002: Scripts are executable
script_shebang = true        # W003: Scripts have shebang

[fmt]
sort_frontmatter = true
indent_size = 2
format_tables = true

[new]
default_license = "MIT"
default_template = "hello-world"
default_lang = "python"
```

### Configuring Rules

Rules with thresholds accept `true` (default), `false` (disabled), or a number:

```toml
[lint.rules]
name_directory = false       # Disable for monorepos
script_executable = false    # Disable for Windows
name_length = 128            # Custom max name length
description_length = false   # Disable description length check
body_length = 1000           # Custom max body lines
```

## CI Integration

Add skilo validation to your GitHub Actions workflow:

```yaml
# .github/workflows/skills.yml
name: Validate Skills

on:
  push:
    branches: [main]
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install skilo
        run: cargo install skilo@0.2.0

      - name: Lint skills
        run: skilo lint . --strict

      - name: Check formatting
        run: skilo fmt . --check
```

To upload results to GitHub's Security tab, use SARIF output:

```yaml
      - name: Run skilo check
        run: skilo lint . --format sarif > results.sarif
        continue-on-error: true

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
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
