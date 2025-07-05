# MkDocs dxlog Plugin

A MkDocs plugin that integrates dxlog research projects into your documentation site.

## Features

- **Automatic Discovery**: Finds and includes all research items from your dxlog project
- **Cross-Reference Resolution**: Converts UUID-based cross-references to MkDocs links
- **Flexible Configuration**: Configurable section names, filtering, and organization
- **Status-Based Organization**: Groups items by their research status (active, completed, archived)
- **Multiple Item Types**: Supports hypotheses, literature reviews, and knowledge base entries
- **Overview Pages**: Generates summary pages for each research item type

## Installation

### Using uv (Recommended)

```bash
uv add mkdocs-dxlog-plugin
uv add mkdocs-section-index  # Required for landing page functionality
```

### Using pip

```bash
pip install mkdocs-dxlog-plugin
pip install mkdocs-section-index  # Required for landing page functionality
```

### Development Installation

For development with uv:

```bash
git clone https://github.com/x-gradient/dxlog
cd dxlog/mkdocs-dxlog-plugin
uv sync --all-extras
```

Or with pip:

```bash
git clone https://github.com/x-gradient/dxlog
cd dxlog/mkdocs-dxlog-plugin
pip install -e ".[dev,testing]"
```

## Configuration

Add the plugin to your `mkdocs.yml`:

```yaml
plugins:
  - search
  - section-index  # Required for dxlog landing pages
  - dxlog:
      dxlog_dir: "path/to/your/dxlog/project"  # Path to dxlog project directory
      section_name: "Research"                  # Name for the documentation section
      include_types: ["hypothesis", "literature", "knowledge"]  # Types to include
      show_archived: false                      # Whether to include archived items
```

**Important**: The `section-index` plugin must be included **before** the `dxlog` plugin for the landing page to work correctly.

### Configuration Options

- `dxlog_dir`: Path to your dxlog project directory (default: ".")
- `section_name`: Name for the research section in navigation (default: "Research")
- `include_types`: List of research item types to include (default: all types)
- `show_archived`: Whether to include archived items (default: false)

#### Table View Options (New!)
- `show_table_view`: Use table format for landing page (default: true)
- `table_sort_by`: Sort column - "name", "type", "status", or "date" (default: "date")
- `table_sort_order`: Sort direction - "asc" or "desc" (default: "desc")
- `max_table_rows`: Maximum rows to display (default: 100)

## Usage

1. **Setup dxlog project**: Create a dxlog project using the dxlog CLI
2. **Configure MkDocs**: Add the plugin to your `mkdocs.yml` configuration
3. **Build documentation**: Run `mkdocs build` or `mkdocs serve`

The plugin will automatically:
- Scan your dxlog project directories
- Parse markdown files with YAML frontmatter
- Convert UUID cross-references to internal links
- Generate navigation structure
- Create overview pages for each research item type

## Project Structure

Your dxlog project structure is preserved in the documentation:

```
Research/
├── Hypothesis/
│   ├── Overview
│   └── Individual hypothesis pages
├── Literature/
│   ├── Overview
│   └── Individual literature pages
└── Knowledge/
    ├── Overview
    └── Individual knowledge pages
```

## Example

See the `example/` directory for a complete example of using the plugin with a sample dxlog project.

## Development

### Using uv (Recommended)

```bash
# Clone the repository
git clone https://github.com/x-gradient/dxlog
cd dxlog/mkdocs-dxlog-plugin

# Install all dependencies including dev tools
uv sync --all-extras

# Run linting
uv run ruff check .
uv run ruff format .

# Run type checking
uv run mypy .

# Run tests
uv run pytest

# Build package
uv build

# Test with example
cd example
uv run mkdocs build
uv run mkdocs serve
```

### Using pip

```bash
# Clone the repository
git clone https://github.com/x-gradient/dxlog
cd dxlog/mkdocs-dxlog-plugin

# Install in development mode
pip install -e ".[dev,testing]"

# Run tests
pytest

# Build example
cd example && mkdocs build
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.