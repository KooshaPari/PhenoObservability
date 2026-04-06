# AI Prompt Logger

Scrape and log prompts from various AI agent platforms to a database.

## Supported Agents

- Claude (claude.ai, claude.code)
- Cursor Agent
- Forge Code (forgecode.dev/forge)
- Codex (OpenAI)
- Factory Droid
- OpenCode
- Kilo Code

## Usage

```python
from prompt_logger import PromptLogger

logger = PromptLogger()
logger.log_prompt(
    agent="claude",
    prompt="Your prompt text here",
    metadata={"conversation_id": "123"}
)
```

## Database

Uses SQLite by default (`prompts.db`).
