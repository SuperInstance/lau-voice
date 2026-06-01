# lau-voice

Voice interface for the Lau game platform — lets kids talk to their game world using natural language. Parses speech into structured commands, generates kid-friendly responses with emotional tone, and maintains conversation memory. Also translates git concepts (commits, branches, merges) into game narration kids actually understand.

## What This Does

`lau-voice` bridges the gap between a child speaking naturally ("I wanna build a castle!") and structured game commands. It:

1. **Parses speech** into typed `VoiceCommand` variants (Build, Move, Create, Teach, Explore, Save, Undo, Branch, Merge, Show)
2. **Responds** with emotionally-tagged `VoiceResponse` text — excited when building, gentle after mistakes, celebrating new creations
3. **Remembers** the conversation in a sliding window, so the assistant has context
4. **Narrates git concepts** as game stories — commits become "You built a crystal tower!", merges become "Your experiment worked!"

The entire pipeline is: `raw speech → VoiceParser::parse() → VoiceCommand → GameAssistant::respond() → VoiceResponse`, with each exchange recorded in `ConversationMemory`.

## Key Idea

Natural language is messy, especially from kids. Instead of requiring exact syntax, `VoiceParser` uses **trigger phrase matching** with a priority cascade. Each command type has a set of trigger words ("build", "construct", "place", "I wanna build"), and the parser tries them in a specific order (Branch before Build, to avoid "make a save point" matching "make"). This is deliberately simple — no ML, no NLP — because it needs to work offline and be auditable.

The git-to-game mapping is the sneaky educational layer: Save = commit, Undo = revert, Branch = save point, Merge = keep the changes. Kids learn version control by playing.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-voice = { git = "https://github.com/SuperInstance/lau-voice" }
```

Requires Rust **2021 edition**.

## Quick Start

```rust
use lau_voice::{GameAssistant, VoiceParser, VoiceCommand, TranscriptFormatter};

// Create the assistant
let mut assistant = GameAssistant::new("Lau");

// Full pipeline: parse speech, respond, remember
let response = assistant.process("I wanna build a castle");
// → "Let's build castle! This is gonna be awesome! 🏗️" (Excited)

// Kid says "oops" → Undo
let response = assistant.process("oops I didn't mean that");
// → "No worries! Let's go back to how things were..." (Gentle)

// Branch = save point
let response = assistant.process("make a save point called risky");
// → "Adventure save point \"risky\" created! ...like having a time machine! ⏳" (Excited)

// Narrate a git commit as a game action
let narration = TranscriptFormatter::narrate_commit("abc1234", "built a crystal tower");
// → "You built a crystal tower! 🏰 [abc1234]"

// Check conversation memory
assert_eq!(assistant.memory.len(), 3);
let last = assistant.memory.last_command(); // Save, Undo, or Branch
```

## API Reference

### VoiceCommand (Enum)

Parsed from speech. Variants:

| Variant | Example Speech | Fields |
|---|---|---|
| `Build` | "build a tower" | `what: String`, `where_pos: Option<(f64,f64,f64)>` |
| `Move` | "go north" | `direction: String` |
| `Create` | "make a dragon called Blaze" | `entity_type: String`, `name: String` |
| `Teach` | "teach Sparky to fly" | `target: String`, `skill: String` |
| `Explore` | "let's check out the cave" | `target: String` |
| `Save` | "save my world" | `message: String` |
| `Undo` | "oops" / "go back" | — |
| `Branch` | "make a save point called risky" | `name: String` |
| `Merge` | "keep the changes" | `branch: String` |
| `Show` | "show me Sparky's history" | `what: String` |
| `Unknown` | (anything unmatched) | `raw: String` |

### VoiceParser

| Method | Signature | Description |
|---|---|---|
| `parse` | `(text: &str) -> VoiceCommand` | Parse raw speech into a command |

### VoiceResponse

| Method | Signature | Description |
|---|---|---|
| `new` | `(text: impl Into<String>, emotion: Emotion) -> Self` | Create a response with text + emotion |

Fields: `speak.text: String`, `speak.emotion: Emotion`

### Emotion (Enum)

`Happy` 😊, `Excited` 🎉, `Thinking` 🤔, `Encouraging` 💪, `Celebrating` 🥳, `Gentle` 🌸, `Playful` 😄

### GameAssistant

| Method | Signature | Description |
|---|---|---|
| `new` | `(name: impl Into<String>) -> Self` | Create assistant with a name |
| `greet` | `(&self) -> VoiceResponse` | Opening greeting |
| `encourage` | `(&self, attempt: usize) -> VoiceResponse` | Cycle through encouragement messages |
| `respond` | `(&mut self, cmd: VoiceCommand) -> VoiceResponse` | Generate contextual response for a command |
| `process` | `(&mut self, text: &str) -> VoiceResponse` | Full pipeline: parse → respond → record |

### ConversationMemory

| Method | Signature | Description |
|---|---|---|
| `new` | `() -> Self` | Empty memory |
| `record` | `(raw, command, response)` | Store an exchange |
| `last_command` | `() -> Option<&VoiceCommand>` | Most recent command |
| `context_window` | `(n: usize) -> &[Exchange]` | Last N exchanges |
| `len` / `is_empty` | — | Standard collection queries |
| `clear` | `()` | Reset history |

### TranscriptFormatter

| Method | Signature | Description |
|---|---|---|
| `narrate_commit` | `(hash, message) -> String` | "You built a tower! 🏰 [abc1234]" |
| `narrate_diff` | `(diff: &str) -> String` | Turns `+` lines into "X appeared in the world! ✨" |
| `narrate_merge` | `(branch: &str) -> String` | "Your experiment worked! 🎊" |

## How It Works

### Parsing Priority

The parser tries command types in this order:

1. **Branch** (must come before Build/Create — "make a save point" has "make")
2. **Merge**
3. **Build**
4. **Move**
5. **Create** ("make a character named X")
6. **Teach**
7. **Explore**
8. **Save**
9. **Undo**
10. **Show**
11. **Unknown** (fallback)

Each `try_*` method checks a set of trigger phrases. If one matches, it returns `Some(VoiceCommand)` immediately. This greedy-first-match approach means trigger order matters, and longer/more-specific phrases must be checked before shorter ones.

### Name Extraction

For `Create` and `Branch` commands, names are extracted from "named X" or "called X" patterns:

```
"make a character named Sparky" → entity_type="character", name="sparky"
"make a save point called risky" → name="risky"
```

All text is lowercased before matching, so casing doesn't matter.

### Git ↔ Game Mapping

| Git Concept | Voice Command | Game Metaphor |
|---|---|---|
| `git commit` | `Save` | "Save your world" |
| `git revert` | `Undo` | "Go back" / "Oops" |
| `git branch` | `Branch` | "Make a save point" |
| `git merge` | `Merge` | "Keep the changes" |
| `git log` | `Show` | "What changed?" |

### Conversation Memory

Each `process()` call records the full exchange: `(raw_text, VoiceCommand, VoiceResponse)`. The `context_window(n)` method returns the last N exchanges for building contextual responses. Memory is bounded only by the conversation length — in production, you'd want a cap.

### TranscriptFormatter Emoji Selection

Commit narration picks emojis by keyword matching on the message:
- "build"/"tower"/"castle" → 🏰
- "create"/"spawn" → ✨
- "teach"/"learn" → 📚
- "explore"/"discover" → 🔍
- "fix"/"repair" → 🔧
- "destroy"/"remove" → 💥
- default → 🌟

### Diff Narration

`narrate_diff` scans for `+` lines (excluding `+++` headers), collects them, and produces:
- 1 addition: "X appeared in the world! ✨"
- Multiple: "X, Y and Z all showed up in the world! 🌟"
- Empty diff: "Nothing changed — everything is exactly the same!"

## Tests

**44 tests** covering:

- **VoiceParser**: Build (simple, kid language, "let's"), Move (directions, "walk to"), Create (named/called), Teach (to/how to), Explore, Save, Undo (including "oops", "go back"), Branch (named, "try something"), Merge, Show, Unknown
- **ConversationMemory**: Record/retrieve, context window, window larger than history
- **GameAssistant**: Greet, encourage (cycling messages), respond to each command type, full `process` pipeline
- **TranscriptFormatter**: Commit narration with hash, diff narration (empty, single, multiple additions), merge narration
- **Serde**: Roundtrips for VoiceCommand, VoiceResponse, ConversationMemory

Run with `cargo test`.

## License

MIT
