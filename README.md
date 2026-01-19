# skilo

[![CI](https://github.com/manuelmauro/skilo/actions/workflows/ci.yml/badge.svg)](https://github.com/manuelmauro/skilo/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/skilo.svg)](https://crates.io/crates/skilo)

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

## Using the Skilo Skill

Skilo includes a skill that teaches AI coding assistants how to use the CLI. Add it to your project to enable your assistant to create and validate skills:

```bash
# Install using skilo add
skilo add manuelmauro/skilo --skill use-skilo
```

Or manually with curl:

```bash
mkdir -p .claude/skills
curl -fsSL https://raw.githubusercontent.com/manuelmauro/skilo/main/.claude/skills/use-skilo/SKILL.md \
  -o .claude/skills/use-skilo/SKILL.md --create-dirs
```

Once installed, your AI assistant will be able to:
- Install skills from repositories using `skilo add`
- Create new skills using `skilo new`
- Validate skills with `skilo lint`
- Format SKILL.md files with `skilo fmt`
- Extract skill metadata with `skilo read-properties`
- Generate agent prompts with `skilo to-prompt`
- Set up CI workflows for skill validation

### Requirements

- Rust 1.92.0 (pinned in `rust-toolchain.toml`)

## Usage

### Install skills from repositories

```bash
# Install all skills from a GitHub repository
skilo add anthropics/skills

# Install a specific skill by name
skilo add vercel-labs/agent-skills --skill code-review

# Install multiple specific skills
skilo add owner/repo --skill lint-fix --skill test-runner

# List available skills without installing
skilo add anthropics/skills --list

# Install from a specific branch
skilo add owner/repo --branch develop

# Install from a specific tag
skilo add owner/repo --tag v1.0.0

# Non-interactive installation (for CI)
skilo add anthropics/skills --skill lint-fix --yes

# Install from a full URL
skilo add https://github.com/owner/repo

# Install from a direct skill path
skilo add https://github.com/owner/repo/tree/main/skills/my-skill

# Install from SSH URL
skilo add git@github.com:owner/repo.git

# Install from local path
skilo add ./path/to/skills
```

Skills are installed to `.claude/skills/<skill-name>/` by default.

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

### Read skill properties

Extract skill metadata as JSON for programmatic use:

```bash
# Read properties from a single skill (outputs JSON object)
skilo read-properties path/to/skill

# Read properties from multiple skills (outputs JSON array)
skilo read-properties path/to/skills/

# Read from multiple paths
skilo read-properties skill-a skill-b
```

Output includes: `name`, `description`, `license`, `compatibility`, `metadata`, `allowed_tools`, and `path`.

### Generate agent prompts

Generate XML for use in agent system prompts:

```bash
# Generate XML for a single skill
skilo to-prompt path/to/skill

# Generate XML for all skills in a directory
skilo to-prompt path/to/skills/
```

Example output:

```xml
<available_skills>
  <skill>
    <name>my-skill</name>
    <description>A brief description of what the skill does.</description>
    <location>path/to/my-skill/SKILL.md</location>
  </skill>
</available_skills>
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

[add]
default_agent = "claude"  # See supported agents below
confirm = true            # Prompt before installing (false for CI)
validate = true           # Validate skills before installing
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

### Supported Agents

The `default_agent` option supports the following AI coding assistants:

| Agent          | Config Value  | Project Path        | Global Path                     |
| -------------- | ------------- | ------------------- | ------------------------------- |
| Claude Code    | `claude`      | `.claude/skills/`   | `~/.claude/skills/`             |
| OpenCode       | `open-code`   | `.opencode/skill/`  | `~/.config/opencode/skill/`     |
| Codex          | `codex`       | `.codex/skills/`    | `~/.codex/skills/`              |
| Cursor         | `cursor`      | `.cursor/skills/`   | `~/.cursor/skills/`             |
| Amp            | `amp`         | `.agents/skills/`   | `~/.config/agents/skills/`      |
| Kilo Code      | `kilo-code`   | `.kilocode/skills/` | `~/.kilocode/skills/`           |
| Roo Code       | `roo-code`    | `.roo/skills/`      | `~/.roo/skills/`                |
| Goose          | `goose`       | `.goose/skills/`    | `~/.config/goose/skills/`       |
| Gemini CLI     | `gemini`      | `.gemini/skills/`   | `~/.gemini/skills/`             |
| Antigravity    | `antigravity` | `.agent/skills/`    | `~/.gemini/antigravity/skills/` |
| GitHub Copilot | `copilot`     | `.github/skills/`   | `~/.copilot/skills/`            |
| Clawdbot       | `clawdbot`    | `skills/`           | `~/.clawdbot/skills/`           |
| Droid          | `droid`       | `.factory/skills/`  | `~/.factory/skills/`            |
| Windsurf       | `windsurf`    | `.windsurf/skills/` | `~/.codeium/windsurf/skills/`   |

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
        run: cargo install skilo@0.5.0

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
