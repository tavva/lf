# lf

Unofficial command-line interface for [Langfuse](https://langfuse.com).

## Quick Start

```bash
# Build and install
cargo build --release
cp target/release/lf ~/.local/bin/

# Configure credentials
lf config setup

# List recent traces
lf traces list

# Get a specific trace with observations
lf traces get <trace-id> --with-observations

# Export to JSON
lf traces list --format json > traces.json
```

## Installation

### From Source

Requires [Rust](https://rustup.rs/) 1.70+.

```bash
git clone https://github.com/your-org/lf.git
cd lf
cargo build --release
```

The binary will be at `target/release/lf`.

### Cargo Install

```bash
cargo install --path .
```

## Configuration

### Interactive Setup

```bash
lf config setup
```

### Environment Variables

```bash
export LANGFUSE_PUBLIC_KEY="pk-..."
export LANGFUSE_SECRET_KEY="sk-..."
export LANGFUSE_HOST="https://cloud.langfuse.com"  # optional, this is the default
```

You can also use a `.env` file in your working directory.

### Multiple Profiles

```bash
# Create a production profile
lf config set --profile production --public-key pk-... --secret-key sk-...

# Use it
lf traces list --profile production

# Or via environment
export LANGFUSE_PROFILE=production
```

### Config File

Credentials are stored in `~/.config/langfuse/config.yml` with restrictive permissions (0600).

```bash
lf config show           # Show current profile (keys masked)
lf config list           # List all profiles
```

## Commands

### Traces

```bash
# List traces with filters
lf traces list --limit 20
lf traces list --user-id user123
lf traces list --session-id sess456
lf traces list --name "chat-completion"
lf traces list --tags prod --tags important
lf traces list --from 2024-01-01T00:00:00Z --to 2024-01-31T23:59:59Z

# Get a single trace
lf traces get <trace-id>
lf traces get <trace-id> --with-observations
```

### Sessions

```bash
lf sessions list
lf sessions list --from 2024-01-01T00:00:00Z
lf sessions get <session-id>
```

### Observations

```bash
# List observations (spans, generations, events)
lf observations list
lf observations list --trace-id <trace-id>
lf observations list --type generation
lf observations list --name "gpt-4-call"

# Get a single observation
lf observations get <observation-id>
```

### Scores

```bash
lf scores list
lf scores list --trace-id <trace-id>
lf scores list --name "quality"
lf scores get <score-id>
```

### Metrics

Query aggregated metrics with flexible dimensions:

```bash
# Count traces by day
lf metrics query --view traces --measure count --aggregation count --granularity day

# Average latency of observations
lf metrics query --view observations --measure latency --aggregation avg

# P95 latency grouped by model
lf metrics query --view observations --measure latency --aggregation p95 -d model

# Total tokens with time range
lf metrics query --view observations --measure total-tokens --aggregation sum \
  --from 2024-01-01T00:00:00Z --to 2024-01-31T23:59:59Z
```

**Measures:** `count`, `latency`, `input-tokens`, `output-tokens`, `total-tokens`, `input-cost`, `output-cost`, `total-cost`

**Aggregations:** `count`, `sum`, `avg`, `p50`, `p95`, `p99`, `histogram`

**Granularities:** `auto`, `minute`, `hour`, `day`, `week`, `month`

## Output Formats

All commands support multiple output formats:

```bash
lf traces list --format table      # Default, human-readable
lf traces list --format json       # JSON for scripting
lf traces list --format csv        # CSV for spreadsheets
lf traces list --format markdown   # Markdown tables
```

Write to a file:

```bash
lf traces list --format json --output traces.json
```

## Global Options

These options work with all data commands:

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--profile` | `LANGFUSE_PROFILE` | Configuration profile name |
| `--public-key` | `LANGFUSE_PUBLIC_KEY` | Langfuse public key |
| `--secret-key` | `LANGFUSE_SECRET_KEY` | Langfuse secret key |
| `--host` | `LANGFUSE_HOST` | Langfuse API host |
| `--format` | | Output format (table/json/csv/markdown) |
| `--output` | | Write output to file |
| `--verbose` | | Show verbose output |
| `--limit` | | Maximum results (default: 50) |
| `--page` | | Page number for pagination |

## Examples

### Export traces for a user to CSV

```bash
lf traces list --user-id user@example.com --format csv > user_traces.csv
```

### Get cost breakdown by model

```bash
lf metrics query --view observations --measure total-cost --aggregation sum -d model --format json
```

### Script: Find slow traces

```bash
lf traces list --format json | jq '.[] | select(.latency > 5000) | .id'
```

### CI/CD: Non-interactive setup

```bash
export LANGFUSE_PUBLIC_KEY="pk-..."
export LANGFUSE_SECRET_KEY="sk-..."
lf config setup --non-interactive
```

## Licence

MIT
