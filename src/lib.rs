//! # lau-voice
//!
//! Voice interface for the Lau platform. Kids talk to their game world and it responds.
//! Provides TTS/STT abstractions, voice command parsing, conversation memory, and a friendly
//! game assistant that explains git concepts in kid-friendly language.

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// VoiceCommand — parsed from speech
// ---------------------------------------------------------------------------

/// A parsed voice command from speech input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VoiceCommand {
    /// "build a tower here"
    Build {
        what: String,
        where_pos: Option<(f64, f64, f64)>,
    },
    /// "go north" / "walk to the mountain"
    Move { direction: String },
    /// "make a character named Sparky"
    Create { entity_type: String, name: String },
    /// "teach Sparky to fly"
    Teach { target: String, skill: String },
    /// "let's check out the crystal cave"
    Explore { target: String },
    /// "save my world"
    Save { message: String },
    /// "undo that" / "go back"
    Undo,
    /// "try something crazy" / "make a save point called risky"
    Branch { name: String },
    /// "keep the changes" / "merge my experiment"
    Merge { branch: String },
    /// "show me Sparky's history" / "what changed?"
    Show { what: String },
    /// Couldn't parse the speech input
    Unknown { raw: String },
}

// ---------------------------------------------------------------------------
// VoiceParser — converts raw text to VoiceCommand
// ---------------------------------------------------------------------------

/// Parses raw speech text into structured `VoiceCommand`s.
pub struct VoiceParser;

impl VoiceParser {
    /// Parse a raw speech transcript into a `VoiceCommand`.
    pub fn parse(text: &str) -> VoiceCommand {
        let lower = text.to_lowercase();
        let trimmed = lower.trim();

        // Branch — check before build/create to catch "make a save point"
        if let Some(cmd) = Self::try_branch(trimmed) {
            return cmd;
        }

        // Merge
        if let Some(cmd) = Self::try_merge(trimmed) {
            return cmd;
        }

        // Build
        if let Some(cmd) = Self::try_build(trimmed) {
            return cmd;
        }

        // Move
        if let Some(cmd) = Self::try_move(trimmed) {
            return cmd;
        }

        // Create — "make a character named Sparky"
        if let Some(cmd) = Self::try_create(trimmed) {
            return cmd;
        }

        // Teach
        if let Some(cmd) = Self::try_teach(trimmed) {
            return cmd;
        }

        // Explore
        if let Some(cmd) = Self::try_explore(trimmed) {
            return cmd;
        }

        // Save
        if let Some(cmd) = Self::try_save(trimmed) {
            return cmd;
        }

        // Undo
        if let Some(cmd) = Self::try_undo(trimmed) {
            return cmd;
        }

        // Show
        if let Some(cmd) = Self::try_show(trimmed) {
            return cmd;
        }

        VoiceCommand::Unknown {
            raw: text.to_string(),
        }
    }

    fn try_build(text: &str) -> Option<VoiceCommand> {
        let build_triggers = ["build", "construct", "place", "put"];
        let wanna_build = ["i wanna build", "i want to build", "i wanna make", "i want to make", "let's build", "lets build"];

        for phrase in wanna_build {
            if let Some(stripped) = text.strip_prefix(phrase) {
                let what = stripped.trim().trim_matches(|c: char| c == 'a' || c == ' ').trim();
                if !what.is_empty() {
                    return Some(VoiceCommand::Build { what: what.to_string(), where_pos: None });
                }
            }
        }

        for trigger in build_triggers {
            if let Some(stripped) = text.strip_prefix(trigger) {
                let rest = stripped.trim();
                // Strip leading article
                let what = rest.trim_start_matches("a ").trim_start_matches("an ").trim();
                if !what.is_empty() {
                    let where_pos = Self::extract_position(what);
                    return Some(VoiceCommand::Build { what: what.to_string(), where_pos });
                }
            } else if text.contains(&format!("{} ", trigger)) {
                let idx = text.find(trigger)?;
                let rest = text[idx + trigger.len()..].trim();
                let what = rest.trim_start_matches("a ").trim_start_matches("an ").trim();
                if !what.is_empty() {
                    let where_pos = Self::extract_position(what);
                    return Some(VoiceCommand::Build { what: what.to_string(), where_pos });
                }
            }
        }

        None
    }

    fn try_move(text: &str) -> Option<VoiceCommand> {
        // Must NOT match "go back" (that's Undo)
        if text.starts_with("go back") || text == "go back" {
            return None;
        }

        let move_triggers = [
            "go to", "walk to", "move to", "run to", "head to",
            "take me to", "let's go to", "lets go to",
            "go ", "walk ", "move ", "run ", "head ", "travel ",
        ];

        for trigger in move_triggers {
            if let Some(stripped) = text.strip_prefix(trigger) {
                let rest = stripped.trim();
                let direction = rest
                    .trim_start_matches("the ")
                    .trim_start_matches("to the ")
                    .to_string();
                if !direction.is_empty() {
                    return Some(VoiceCommand::Move { direction });
                }
            }
        }

        None
    }

    fn try_create(text: &str) -> Option<VoiceCommand> {
        // Longer triggers first to avoid partial matches
        let create_triggers = [
            "i wanna make a ", "i wanna make an ",
            "make a ", "make an ",
            "create a ", "create an ", "create ",
            "spawn a ", "spawn an ", "spawn ",
            "summon a ", "summon an ", "summon ",
        ];

        for trigger in create_triggers {
            if text.starts_with(trigger) || (trigger.ends_with(' ') && text.contains(trigger)) {
                let idx = text.find(trigger)?;
                let rest = text[idx + trigger.len()..].trim();
                // Try to find "named X" or "called X"
                let (entity_type, name) = if let Some(name) = Self::extract_name(rest) {
                    // Strip the " named X" or " called X" suffix from entity_type
                    let entity;
                    if let Some(pos) = rest.find(" named ") {
                        entity = rest[..pos].trim().to_string();
                    } else if let Some(pos) = rest.find(" called ") {
                        entity = rest[..pos].trim().to_string();
                    } else {
                        entity = rest.to_string();
                    }
                    (entity, name)
                } else {
                    (rest.to_string(), String::new())
                };

                if !entity_type.is_empty() {
                    return Some(VoiceCommand::Create { entity_type, name });
                }
            }
        }

        None
    }

    fn try_teach(text: &str) -> Option<VoiceCommand> {
        let teach_triggers = ["teach ", "train "];

        for trigger in teach_triggers {
            if let Some(stripped) = text.strip_prefix(trigger) {
                let rest = stripped.trim();

                // "teach Sparky to fly" or "teach sparky how to fly"
                // Check "how to" before "to" to avoid splitting "how" into target
                let (target, skill) = if let Some(how_pos) = rest.find(" how to ") {
                    let target = rest[..how_pos].trim().to_string();
                    let skill = rest[how_pos + 8..].trim().to_string();
                    (target, skill)
                } else if let Some(to_pos) = rest.find(" to ") {
                    let target = rest[..to_pos].trim().to_string();
                    let skill = rest[to_pos + 4..].trim().to_string();
                    (target, skill)
                } else {
                    return None;
                };

                if !target.is_empty() && !skill.is_empty() {
                    return Some(VoiceCommand::Teach { target, skill });
                }
            }
        }

        None
    }

    fn try_explore(text: &str) -> Option<VoiceCommand> {
        let explore_triggers = [
            "explore", "check out", "let's go to", "lets go to",
            "visit", "look at", "investigate", "discover",
        ];

        for trigger in explore_triggers {
            if text.starts_with(trigger) || text.contains(&format!("{} ", trigger)) {
                let idx = text.find(trigger)?;
                let rest = text[idx + trigger.len()..].trim()
                    .trim_start_matches("the ")
                    .trim_start_matches("a ")
                    .trim();
                if !rest.is_empty() {
                    return Some(VoiceCommand::Explore { target: rest.to_string() });
                }
            }
        }

        None
    }

    fn try_save(text: &str) -> Option<VoiceCommand> {
        let save_triggers = ["save", "keep this", "remember this"];

        for trigger in save_triggers {
            if text.starts_with(trigger) || text.contains(&format!("{} ", trigger)) {
                let idx = text.find(trigger)?;
                let rest = text[idx + trigger.len()..].trim()
                    .trim_start_matches("my ")
                    .trim_start_matches("the ")
                    .trim_start_matches("this ")
                    .trim();
                let message = if rest.is_empty() {
                    "my progress".to_string()
                } else {
                    rest.to_string()
                };
                return Some(VoiceCommand::Save { message });
            }
        }

        None
    }

    fn try_undo(text: &str) -> Option<VoiceCommand> {
        let undo_phrases = [
            "undo", "go back", "take it back", "revert",
            "never mind", "nevermind", "oops", "cancel that",
        ];

        for phrase in undo_phrases {
            if text.contains(phrase) {
                return Some(VoiceCommand::Undo);
            }
        }

        None
    }

    fn try_branch(text: &str) -> Option<VoiceCommand> {
        let branch_triggers = [
            "make a save point", "create a save point",
            "new adventure", "try something", "experiment",
            "branch", "save point",
        ];

        for trigger in branch_triggers {
            if text.contains(trigger) {
                // Try to extract a name after "called" or "named"
                let rest = text;
                let name = Self::extract_name(rest)
                    .or_else(|| Self::extract_after(trigger, rest))
                    .unwrap_or_else(|| "experiment".to_string());
                return Some(VoiceCommand::Branch { name });
            }
        }

        None
    }

    fn try_merge(text: &str) -> Option<VoiceCommand> {
        let merge_triggers = [
            "merge", "keep the changes", "combine",
            "bring together", "merge my",
        ];

        for trigger in merge_triggers {
            if text.contains(trigger) {
                let idx = text.find(trigger)?;
                let rest = text[idx + trigger.len()..].trim()
                    .trim_start_matches("the ")
                    .trim_start_matches("my ")
                    .trim_start_matches("changes ")
                    .trim_start_matches("from ")
                    .trim();
                let branch = if rest.is_empty() {
                    "experiment".to_string()
                } else {
                    rest.to_string()
                };
                return Some(VoiceCommand::Merge { branch });
            }
        }

        None
    }

    fn try_show(text: &str) -> Option<VoiceCommand> {
        let show_triggers = [
            "show me", "what changed", "show", "tell me about",
            "what's new", "whats new", "history of", "log of",
        ];

        for trigger in show_triggers {
            if text.starts_with(trigger) || text.contains(&format!("{} ", trigger)) {
                let idx = text.find(trigger)?;
                let rest = text[idx + trigger.len()..].trim()
                    .trim_start_matches("the ")
                    .trim_start_matches("me ")
                    .trim_start_matches("'s ")
                    .trim();
                let what = if rest.is_empty() {
                    "history".to_string()
                } else {
                    rest.to_string()
                };
                return Some(VoiceCommand::Show { what });
            }
        }

        None
    }

    /// Extract a name from "named X" or "called X" patterns.
    fn extract_name(text: &str) -> Option<String> {
        for sep in ["named ", "called "] {
            if let Some(idx) = text.find(sep) {
                let name = text[idx + sep.len()..].trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        None
    }

    /// Extract text after a trigger phrase, as a fallback name.
    fn extract_after(trigger: &str, text: &str) -> Option<String> {
        let idx = text.find(trigger)?;
        let rest = text[idx + trigger.len()..].trim()
            .trim_start_matches("called ")
            .trim_start_matches("named ")
            .trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    }

    /// Extract a position like "at 1 2 3" or "here" → None.
    fn extract_position(_text: &str) -> Option<(f64, f64, f64)> {
        // Position extraction from natural language is complex;
        // "here" means current position (None), explicit coords would be parsed here.
        None
    }
}

// ---------------------------------------------------------------------------
// Emotion & VoiceResponse
// ---------------------------------------------------------------------------

/// Emotion tags for voice responses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Emotion {
    Happy,
    Excited,
    Thinking,
    Encouraging,
    Celebrating,
    Gentle,
    Playful,
}

impl fmt::Display for Emotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Emotion::Happy => write!(f, "😊"),
            Emotion::Excited => write!(f, "🎉"),
            Emotion::Thinking => write!(f, "🤔"),
            Emotion::Encouraging => write!(f, "💪"),
            Emotion::Celebrating => write!(f, "🥳"),
            Emotion::Gentle => write!(f, "🌸"),
            Emotion::Playful => write!(f, "😄"),
        }
    }
}

/// A voice response to speak back to the player.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceResponse {
    pub speak: Speak,
}

/// Inner speak content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Speak {
    pub text: String,
    pub emotion: Emotion,
}

impl VoiceResponse {
    /// Create a new voice response.
    pub fn new(text: impl Into<String>, emotion: Emotion) -> Self {
        VoiceResponse {
            speak: Speak { text: text.into(), emotion },
        }
    }
}

// ---------------------------------------------------------------------------
// ConversationMemory
// ---------------------------------------------------------------------------

/// A single exchange in the conversation.
pub type Exchange = (String, VoiceCommand, VoiceResponse);

/// Tracks conversation history between the player and the game assistant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMemory {
    history: Vec<Exchange>,
}

impl ConversationMemory {
    /// Create a new empty conversation memory.
    pub fn new() -> Self {
        ConversationMemory { history: Vec::new() }
    }

    /// Record a new exchange.
    pub fn record(&mut self, raw_text: String, command: VoiceCommand, response: VoiceResponse) {
        self.history.push((raw_text, command, response));
    }

    /// Get the last command, if any.
    pub fn last_command(&self) -> Option<&VoiceCommand> {
        self.history.last().map(|(_, cmd, _)| cmd)
    }

    /// Get the most recent N exchanges for context.
    pub fn context_window(&self, n: usize) -> &[Exchange] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }

    /// Total number of exchanges recorded.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Whether the memory is empty.
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.history.clear();
    }
}

impl Default for ConversationMemory {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// GameAssistant
// ---------------------------------------------------------------------------

/// The friendly voice companion for kids in the Lau game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAssistant {
    pub name: String,
    pub memory: ConversationMemory,
}

impl GameAssistant {
    /// Create a new game assistant with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        GameAssistant {
            name: name.into(),
            memory: ConversationMemory::new(),
        }
    }

    /// Generate a greeting for the player.
    pub fn greet(&self) -> VoiceResponse {
        VoiceResponse::new(
            format!("Hey there! I'm {}! What should we build today? 🌟", self.name),
            Emotion::Excited,
        )
    }

    /// Generate an encouraging response for an attempt.
    pub fn encourage(&self, attempt: usize) -> VoiceResponse {
        let messages = [
            "That's a great idea! Let's try it!",
            "Ooh, I like where this is going! Keep going!",
            "You're getting really good at this!",
            "Awesome! Let me help you with that!",
            "Every adventure starts with one step — let's take it together!",
        ];
        let msg = messages[attempt % messages.len()];
        VoiceResponse::new(msg, Emotion::Encouraging)
    }

    /// Respond to a parsed voice command with a kid-friendly, contextual response.
    pub fn respond(&mut self, command: VoiceCommand) -> VoiceResponse {
        let response = match &command {
            VoiceCommand::Build { what, where_pos: _ } => VoiceResponse::new(
                format!("Let's build {}! This is gonna be awesome! 🏗️", what),
                Emotion::Excited,
            ),
            VoiceCommand::Move { direction } => VoiceResponse::new(
                format!("On our way to {}! Adventure awaits! 🗺️", direction),
                Emotion::Excited,
            ),
            VoiceCommand::Create { entity_type, name } => {
                if name.is_empty() {
                    VoiceResponse::new(
                        format!("Creating a new {}! What should we name it?", entity_type),
                        Emotion::Happy,
                    )
                } else {
                    VoiceResponse::new(
                        format!("{} the {} is born! Welcome to the world! ✨", name, entity_type),
                        Emotion::Celebrating,
                    )
                }
            }
            VoiceCommand::Teach { target, skill } => VoiceResponse::new(
                format!("Teaching {} to {}... They're a fast learner! 📚", target, skill),
                Emotion::Encouraging,
            ),
            VoiceCommand::Explore { target } => VoiceResponse::new(
                format!("Let's check out {}! I wonder what we'll find! 🔍", target),
                Emotion::Playful,
            ),
            VoiceCommand::Save { message } => VoiceResponse::new(
                format!("Saved! \"{}\" — your world is safe and sound! 💾", message),
                Emotion::Happy,
            ),
            VoiceCommand::Undo => VoiceResponse::new(
                "No worries! Let's go back to how things were. Everyone makes mistakes — that's how we learn! 🔄",
                Emotion::Gentle,
            ),
            VoiceCommand::Branch { name } => VoiceResponse::new(
                format!(
                    "Adventure save point \"{}\" created! Now we can try wild ideas and come back if we need to! This is like having a time machine! ⏳",
                    name
                ),
                Emotion::Excited,
            ),
            VoiceCommand::Merge { branch } => VoiceResponse::new(
                format!(
                    "Merging the fun stuff from \"{}\"! Your experiment worked — the cool changes are part of the world now! 🎊",
                    branch
                ),
                Emotion::Celebrating,
            ),
            VoiceCommand::Show { what } => VoiceResponse::new(
                format!("Let me show you {}! Here's what's been happening... 📋", what),
                Emotion::Thinking,
            ),
            VoiceCommand::Unknown { raw } => VoiceResponse::new(
                format!("Hmm, I'm not sure what \"{}\" means, but I'd love to help! Can you try saying it a different way?", raw),
                Emotion::Gentle,
            ),
        };

        response
    }

    /// Process raw speech text: parse it, respond, and record the exchange.
    pub fn process(&mut self, text: &str) -> VoiceResponse {
        let command = VoiceParser::parse(text);
        let raw = text.to_string();
        let response = self.respond(command.clone());
        self.memory.record(raw, command, response.clone());
        response
    }
}

// ---------------------------------------------------------------------------
// TranscriptFormatter — narrates game actions
// ---------------------------------------------------------------------------

/// Converts game actions (git concepts) into kid-friendly narration.
pub struct TranscriptFormatter;

impl TranscriptFormatter {
    /// Narrate a commit: "You built a crystal tower! 🏰"
    pub fn narrate_commit(hash: &str, message: &str) -> String {
        let short_hash = if hash.len() > 7 { &hash[..7] } else { hash };
        let emoji = Self::emoji_for_action(message);
        format!("You {}! {} [{}]", message.trim_end_matches('.'), emoji, short_hash)
    }

    /// Narrate a diff: "Sparky learned to glow in the dark!"
    pub fn narrate_diff(diff: &str) -> String {
        if diff.trim().is_empty() {
            return "Nothing changed — everything is exactly the same!".to_string();
        }

        let mut changes = Vec::new();
        for line in diff.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                let added = line[1..].trim();
                if !added.is_empty() {
                    changes.push(added.to_string());
                }
            }
        }

        if changes.is_empty() {
            return "Some things shifted around behind the scenes!".to_string();
        }

        if changes.len() == 1 {
            return format!("{} appeared in the world! ✨", changes[0]);
        }

        let last = changes.pop().unwrap();
        format!(
            "{} and {} all showed up in the world! 🌟",
            changes.join(", "),
            last
        )
    }

    /// Narrate a merge: "Your experiment worked! The flying castle is real!"
    pub fn narrate_merge(branch: &str) -> String {
        format!(
            "Your experiment \"{}\" worked! The fun stuff is now part of your world! 🎊",
            branch
        )
    }

    fn emoji_for_action(message: &str) -> &'static str {
        let lower = message.to_lowercase();
        if lower.contains("build") || lower.contains("tower") || lower.contains("castle") {
            "🏰"
        } else if lower.contains("create") || lower.contains("spawn") {
            "✨"
        } else if lower.contains("teach") || lower.contains("learn") {
            "📚"
        } else if lower.contains("explore") || lower.contains("discover") {
            "🔍"
        } else if lower.contains("fix") || lower.contains("repair") {
            "🔧"
        } else if lower.contains("destroy") || lower.contains("remove") {
            "💥"
        } else {
            "🌟"
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- VoiceParser tests ---

    #[test]
    fn parse_build_simple() {
        let cmd = VoiceParser::parse("build a tower");
        assert_eq!(cmd, VoiceCommand::Build { what: "tower".to_string(), where_pos: None });
    }

    #[test]
    fn parse_build_kid_language() {
        let cmd = VoiceParser::parse("I wanna build a castle");
        assert_eq!(cmd, VoiceCommand::Build { what: "castle".to_string(), where_pos: None });
    }

    #[test]
    fn parse_build_lets() {
        let cmd = VoiceParser::parse("let's build a spaceship");
        assert_eq!(cmd, VoiceCommand::Build { what: "spaceship".to_string(), where_pos: None });
    }

    #[test]
    fn parse_move_direction() {
        let cmd = VoiceParser::parse("go north");
        assert_eq!(cmd, VoiceCommand::Move { direction: "north".to_string() });
    }

    #[test]
    fn parse_walk_to() {
        let cmd = VoiceParser::parse("walk to the mountain");
        assert_eq!(cmd, VoiceCommand::Move { direction: "mountain".to_string() }); // strips "to the"
    }

    #[test]
    fn parse_create_named() {
        let cmd = VoiceParser::parse("make a character named Sparky");
        assert_eq!(cmd, VoiceCommand::Create {
            entity_type: "character".to_string(),
            name: "sparky".to_string(), // lowercase normalized
        });
    }

    #[test]
    fn parse_create_called() {
        let cmd = VoiceParser::parse("create a dragon called Blaze");
        assert_eq!(cmd, VoiceCommand::Create {
            entity_type: "dragon".to_string(),
            name: "blaze".to_string(), // lowercase normalized
        });
    }

    #[test]
    fn parse_teach() {
        let cmd = VoiceParser::parse("teach Sparky to fly");
        assert_eq!(cmd, VoiceCommand::Teach {
            target: "sparky".to_string(), // lowercase normalized
            skill: "fly".to_string(),
        });
    }

    #[test]
    fn parse_teach_how_to() {
        let cmd = VoiceParser::parse("teach Blaze how to breathe fire");
        assert_eq!(cmd, VoiceCommand::Teach {
            target: "blaze".to_string(), // lowercase normalized
            skill: "breathe fire".to_string(),
        });
    }

    #[test]
    fn parse_explore() {
        let cmd = VoiceParser::parse("let's check out the crystal cave");
        assert_eq!(cmd, VoiceCommand::Explore { target: "crystal cave".to_string() });
    }

    #[test]
    fn parse_save() {
        let cmd = VoiceParser::parse("save my world");
        assert_eq!(cmd, VoiceCommand::Save { message: "world".to_string() });
    }

    #[test]
    fn parse_save_simple() {
        let cmd = VoiceParser::parse("save");
        assert_eq!(cmd, VoiceCommand::Save { message: "my progress".to_string() });
    }

    #[test]
    fn parse_undo() {
        let cmd = VoiceParser::parse("undo that");
        assert_eq!(cmd, VoiceCommand::Undo);
    }

    #[test]
    fn parse_go_back() {
        let cmd = VoiceParser::parse("go back");
        assert_eq!(cmd, VoiceCommand::Undo);
    }

    #[test]
    fn parse_oops() {
        let cmd = VoiceParser::parse("oops I didn't mean that");
        assert_eq!(cmd, VoiceCommand::Undo);
    }

    #[test]
    fn parse_branch_named() {
        let cmd = VoiceParser::parse("make a save point called risky");
        assert_eq!(cmd, VoiceCommand::Branch { name: "risky".to_string() });
    }

    #[test]
    fn parse_branch_try_something() {
        let cmd = VoiceParser::parse("try something crazy");
        assert_eq!(cmd, VoiceCommand::Branch { name: "crazy".to_string() });
    }

    #[test]
    fn parse_merge() {
        let cmd = VoiceParser::parse("keep the changes");
        assert_eq!(cmd, VoiceCommand::Merge { branch: "experiment".to_string() });
    }

    #[test]
    fn parse_merge_explicit() {
        let cmd = VoiceParser::parse("merge my adventure");
        assert_eq!(cmd, VoiceCommand::Merge { branch: "adventure".to_string() });
    }

    #[test]
    fn parse_show() {
        let cmd = VoiceParser::parse("show me Sparky's history");
        if let VoiceCommand::Show { what } = cmd {
            assert!(what.contains("sparky"));
        } else {
            panic!("expected Show, got {:?}", cmd);
        }
    }

    #[test]
    fn parse_what_changed() {
        let cmd = VoiceParser::parse("what changed?");
        assert!(matches!(cmd, VoiceCommand::Show { .. }));
    }

    #[test]
    fn parse_unknown() {
        let cmd = VoiceParser::parse("blargle flargle snort");
        assert!(matches!(cmd, VoiceCommand::Unknown { ref raw } if raw == "blargle flargle snort"));
    }

    // --- ConversationMemory tests ---

    #[test]
    fn memory_record_and_retrieve() {
        let mut mem = ConversationMemory::new();
        assert!(mem.is_empty());

        let cmd = VoiceCommand::Build { what: "tower".to_string(), where_pos: None };
        let resp = VoiceResponse::new("Building!", Emotion::Excited);
        mem.record("build a tower".to_string(), cmd.clone(), resp);

        assert_eq!(mem.len(), 1);
        assert_eq!(mem.last_command(), Some(&cmd));
    }

    #[test]
    fn memory_context_window() {
        let mut mem = ConversationMemory::new();
        for i in 0..5 {
            let cmd = VoiceCommand::Build { what: format!("thing {}", i), where_pos: None };
            mem.record(format!("build thing {}", i), cmd, VoiceResponse::new("ok", Emotion::Happy));
        }

        let ctx = mem.context_window(2);
        assert_eq!(ctx.len(), 2);
        // Should be the last two
        assert_eq!(ctx[0].0, "build thing 3");
        assert_eq!(ctx[1].0, "build thing 4");
    }

    #[test]
    fn memory_context_window_larger_than_history() {
        let mem = ConversationMemory::new();
        let ctx = mem.context_window(10);
        assert!(ctx.is_empty());
    }

    // --- GameAssistant tests ---

    #[test]
    fn assistant_greet() {
        let assistant = GameAssistant::new("Lau");
        let resp = assistant.greet();
        assert!(resp.speak.text.contains("Lau"));
        assert_eq!(resp.speak.emotion, Emotion::Excited);
    }

    #[test]
    fn assistant_encourage() {
        let assistant = GameAssistant::new("Lau");
        let resp = assistant.encourage(0);
        assert!(resp.speak.text.contains("great idea"));
        assert_eq!(resp.speak.emotion, Emotion::Encouraging);
    }

    #[test]
    fn assistant_encourage_cycles() {
        let assistant = GameAssistant::new("Lau");
        let r0 = assistant.encourage(0);
        let r1 = assistant.encourage(1);
        // Should produce different messages
        assert_ne!(r0.speak.text, r1.speak.text);
    }

    #[test]
    fn assistant_respond_build() {
        let mut assistant = GameAssistant::new("Lau");
        let cmd = VoiceCommand::Build { what: "crystal tower".to_string(), where_pos: None };
        let resp = assistant.respond(cmd);
        assert!(resp.speak.text.contains("crystal tower"));
    }

    #[test]
    fn assistant_respond_create_named() {
        let mut assistant = GameAssistant::new("Lau");
        let cmd = VoiceCommand::Create {
            entity_type: "dragon".to_string(),
            name: "Blaze".to_string(),
        };
        let resp = assistant.respond(cmd);
        assert!(resp.speak.text.contains("Blaze"));
        assert!(resp.speak.text.contains("born"));
        assert_eq!(resp.speak.emotion, Emotion::Celebrating);
    }

    #[test]
    fn assistant_respond_undo() {
        let mut assistant = GameAssistant::new("Lau");
        let resp = assistant.respond(VoiceCommand::Undo);
        assert!(resp.speak.text.contains("go back"));
        assert_eq!(resp.speak.emotion, Emotion::Gentle);
    }

    #[test]
    fn assistant_respond_branch() {
        let mut assistant = GameAssistant::new("Lau");
        let resp = assistant.respond(VoiceCommand::Branch { name: "risky".to_string() });
        assert!(resp.speak.text.contains("risky"));
        assert!(resp.speak.text.contains("time machine"));
    }

    #[test]
    fn assistant_respond_merge() {
        let mut assistant = GameAssistant::new("Lau");
        let resp = assistant.respond(VoiceCommand::Merge { branch: "adventure".to_string() });
        assert!(resp.speak.text.contains("adventure"));
        assert_eq!(resp.speak.emotion, Emotion::Celebrating);
    }

    #[test]
    fn assistant_respond_unknown() {
        let mut assistant = GameAssistant::new("Lau");
        let resp = assistant.respond(VoiceCommand::Unknown { raw: "asdf".to_string() });
        assert!(resp.speak.text.contains("not sure"));
        assert_eq!(resp.speak.emotion, Emotion::Gentle);
    }

    #[test]
    fn assistant_process_full_pipeline() {
        let mut assistant = GameAssistant::new("Lau");
        let resp = assistant.process("build a castle");
        assert!(resp.speak.text.contains("castle"));
        assert_eq!(assistant.memory.len(), 1);

        let resp2 = assistant.process("save my world");
        assert!(resp2.speak.text.contains("Saved"));
        assert_eq!(assistant.memory.len(), 2);
        assert!(matches!(assistant.memory.last_command(), Some(VoiceCommand::Save { .. })));
    }

    // --- TranscriptFormatter tests ---

    #[test]
    fn narrate_commit() {
        let result = TranscriptFormatter::narrate_commit("abc123def", "built a crystal tower");
        assert!(result.contains("built a crystal tower"));
        assert!(result.contains("abc123d"));
        assert!(result.contains("🏰"));
    }

    #[test]
    fn narrate_commit_short_hash() {
        let result = TranscriptFormatter::narrate_commit("abc", "explored the cave");
        assert!(result.contains("[abc]"));
    }

    #[test]
    fn narrate_diff_with_additions() {
        let diff = "+Sparky learned to glow\n-some old thing";
        let result = TranscriptFormatter::narrate_diff(diff);
        assert!(result.contains("Sparky learned to glow"));
    }

    #[test]
    fn narrate_diff_empty() {
        let result = TranscriptFormatter::narrate_diff("");
        assert!(result.contains("Nothing changed"));
    }

    #[test]
    fn narrate_diff_multiple_additions() {
        let diff = "+a unicorn\n+a rainbow\n+a castle";
        let result = TranscriptFormatter::narrate_diff(diff);
        assert!(result.contains("unicorn") && result.contains("rainbow") && result.contains("castle"));
    }

    #[test]
    fn narrate_merge() {
        let result = TranscriptFormatter::narrate_merge("flying castle");
        assert!(result.contains("flying castle"));
        assert!(result.contains("experiment"));
    }

    // --- Serde tests ---

    #[test]
    fn serde_voice_command() {
        let cmd = VoiceCommand::Build { what: "tower".to_string(), where_pos: Some((1.0, 2.0, 3.0)) };
        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: VoiceCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, parsed);
    }

    #[test]
    fn serde_voice_response() {
        let resp = VoiceResponse::new("Hello!", Emotion::Playful);
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: VoiceResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, parsed);
    }

    #[test]
    fn serde_conversation_memory() {
        let mut mem = ConversationMemory::new();
        mem.record("build".to_string(), VoiceCommand::Build { what: "x".to_string(), where_pos: None }, VoiceResponse::new("ok", Emotion::Happy));
        let json = serde_json::to_string(&mem).unwrap();
        let parsed: ConversationMemory = serde_json::from_str(&json).unwrap();
        assert_eq!(mem.len(), parsed.len());
    }
}
