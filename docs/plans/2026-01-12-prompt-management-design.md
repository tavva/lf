# Prompt Management Design

Add full CRUD operations for Langfuse prompts to the CLI.

## CLI Structure

```
lf prompts list          # List prompts with filters
lf prompts get           # Get prompt (structured output, --raw for content only)
lf prompts create-text   # Create text prompt from file/stdin
lf prompts create-chat   # Create chat prompt from JSON file/stdin
lf prompts label         # Set labels on a version
lf prompts delete        # Delete prompt or specific version
```

## Langfuse Prompt Model

Prompts have versions and labels:
- A prompt like "welcome-message" can have versions 1, 2, 3...
- Each version has immutable content (cannot edit)
- Labels are tags pointing to versions: "production", "staging", "latest"
- When fetching, request by version number OR label (defaults to "production")

To update a production prompt:
1. Create a new version with new content
2. Move the "production" label to that version

## Command Arguments

### `lf prompts list`

```
--name <name>         Filter by prompt name
--label <label>       Filter by label
--tag <tag>           Filter by tag (repeatable)
--limit <n>           Max results (default 50)
--page <n>            Page number (default 1)
--format <fmt>        Output format (table/json/csv/markdown)
--output <file>       Write to file
```

### `lf prompts get <name>`

```
<name>                Prompt name (required, positional)
--version <n>         Specific version number
--label <label>       Fetch by label (default: "production")
--raw                 Output content only (for piping)
--format <fmt>        Output format (ignored if --raw)
--output <file>       Write to file
```

### `lf prompts create-text --name <name>`

```
--name <name>         Prompt name (required)
--file <path>         Read content from file (or stdin if omitted)
--labels <l>          Labels to apply (repeatable)
--tags <t>            Tags to apply (repeatable)
--config <json>       Model config as JSON string
```

### `lf prompts create-chat --name <name>`

```
--name <name>         Prompt name (required)
--file <path>         Read JSON messages from file (or stdin if omitted)
--labels <l>          Labels to apply (repeatable)
--tags <t>            Tags to apply (repeatable)
--config <json>       Model config as JSON string
```

### `lf prompts label <name> <version>`

```
<name>                Prompt name (required, positional)
<version>             Version number (required, positional)
--labels <l>          Labels to set (repeatable, required)
```

### `lf prompts delete <name>`

```
<name>                Prompt name (required, positional)
--version <n>         Delete specific version only
--label <label>       Delete versions with this label only
```

All commands inherit standard auth flags: `--profile`, `--public-key`, `--secret-key`, `--host`.

## Data Types

### Prompt (response from API)

```rust
struct Prompt {
    name: String,
    version: i32,
    labels: Vec<String>,
    tags: Vec<String>,
    prompt: PromptContent,  // text string or chat messages
    config: Option<Value>,  // model params, temperature, etc.
    created_at: String,
    updated_at: String,
}

enum PromptContent {
    Text(String),
    Chat(Vec<ChatMessage>),
}

struct ChatMessage {
    role: String,      // "system", "user", "assistant"
    content: String,
}
```

### PromptMeta (from list endpoint)

```rust
struct PromptMeta {
    name: String,
    versions: Vec<i32>,
    labels: Vec<String>,
    tags: Vec<String>,
    last_updated_at: String,
}
```

## Client Methods

```rust
// In client.rs
list_prompts(name, label, tag, limit, page) -> Vec<PromptMeta>
get_prompt(name, version, label) -> Prompt
create_text_prompt(name, content, labels, tags, config) -> Prompt
create_chat_prompt(name, messages, labels, tags, config) -> Prompt
update_prompt_labels(name, version, labels) -> Prompt
delete_prompt(name, version, label) -> ()
```

## Implementation

### New files

- `src/commands/prompts.rs` - command definitions and execute logic

### Modified files

- `src/types.rs` - add Prompt, PromptMeta, ChatMessage types
- `src/client.rs` - add prompt API methods
- `src/main.rs` - add Prompts variant to Commands enum
- `src/commands/mod.rs` - export prompts module

### Stdin reading (for create commands)

```rust
fn read_content(file: Option<&str>) -> Result<String> {
    match file {
        Some(path) => std::fs::read_to_string(path),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
```

### Raw output (for get --raw)

- Text prompts: print the string directly
- Chat prompts: print as JSON array (canonical format)

## API Endpoints

All endpoints use `/api/public/v2/prompts` base path with Basic Auth.

| Operation | Method | Path |
|-----------|--------|------|
| List | GET | `/v2/prompts` |
| Get | GET | `/v2/prompts/{name}` |
| Create | POST | `/v2/prompts` |
| Update labels | PATCH | `/v2/prompts/{name}/versions/{version}` |
| Delete | DELETE | `/v2/prompts/{name}` |
