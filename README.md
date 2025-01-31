# dxlog

Just as a derivative (dx) measures the rate of change at each point along a path, research progress is made through small, deliberate steps of discovery. dxlog embraces this philosophy, helping teams track their research journey—hypothesis by hypothesis, paper by paper, insight by insight.

A command-line tool for managing research hypotheses, literature reviews, and knowledge bases in small research teams.


## Quick Start

Start by installing the CLI `cargo install --git https://github.com/exploding-gradient/dxlog dxlog-cli`


1. Initialize a new research repository:
```bash
# Initialize dxlog
dxlog init my-research

cd my-research
```

2. Create your first hypothesis:
```bash
dxlog hypothesis new "Impact of quantum noise on error rates" --tags quantum,noise
```

3. Add a literature review:
```bash
dxlog literature new --url https://arxiv.org/abs/2401.12345 --tags quantum,error-correction
```

4. Link related research:
```bash
# Add a reference from hypothesis to literature
dxlog reference add <hypothesis-id> <literature-id>
```

5. Update research status:
```bash
# Mark hypothesis as proven
dxlog hypothesis proven <hypothesis-id>

# Complete literature review
dxlog literature complete <literature-id>
```

## Repository Structure

```
my-research/
├── research-logs/      # Active research items
├── knowledge-base/     # Proven hypotheses and completed reviews
├── archived/          # Archived or obsolete items
└── templates/         # Custom templates for new entries
```

## Common Workflows

### Managing Hypotheses

```bash
# Create new hypothesis
dxlog hypothesis new "Your hypothesis" -t tag1,tag2

# List active hypotheses
dxlog hypothesis list -s active

# Mark as proven/disproven
dxlog hypothesis proven <id>
dxlog hypothesis disproven <id>
```

### Literature Reviews

```bash
# Start review from arXiv
dxlog literature new --url https://arxiv.org/abs/2401.12345

# List in-progress reviews
dxlog literature list -s in_progress

# Complete review
dxlog literature complete <id>
```

### Knowledge Base

```bash
# Create knowledge entry
dxlog knowledge new "Implementation Guide" -t guide,implementation

# List published entries
dxlog knowledge list -s published

# Archive outdated entry
dxlog knowledge archive <id>
```

## Configuration

The `dxlog.toml` file in your repository controls:
- Directory locations
- Template customization
- Git integration
- Date formats

Example configuration:
```toml
date-format = "%Y-%m-%d"
stale-days = 14

[storage]
active-dir = "research-logs"
archive-dir = "archived"
knowledge-base-dir = "knowledge-base"

[templates]
hypothesis = "templates/hypothesis.jinja"
literature = "templates/literature.jinja"
knowledge = "templates/knowledge.jinja"

[git]
enabled = true
auto-commit = false
```

