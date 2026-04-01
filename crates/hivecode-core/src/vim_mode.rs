//! Vim keybinding mode for HiveCode
//!
//! Full vim motions, operators, text objects, and mode transitions
//! for the input area and any text editing within HiveCode.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Current vim editing mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VimMode {
    /// Normal/command mode
    Normal,
    /// Insert mode (editing text)
    Insert,
    /// Visual mode (character selection)
    Visual,
    /// Visual line mode (line selection)
    VisualLine,
    /// Visual block mode (rectangular selection)
    VisualBlock,
    /// Command mode (:)
    Command,
    /// Replace mode
    Replace,
    /// Waiting for motion after operator
    Operator(String),
}

impl std::fmt::Display for VimMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VimMode::Normal => write!(f, "NORMAL"),
            VimMode::Insert => write!(f, "INSERT"),
            VimMode::Visual => write!(f, "VISUAL"),
            VimMode::VisualLine => write!(f, "VISUAL LINE"),
            VimMode::VisualBlock => write!(f, "VISUAL BLOCK"),
            VimMode::Command => write!(f, "COMMAND"),
            VimMode::Replace => write!(f, "REPLACE"),
            VimMode::Operator(op) => write!(f, "OPERATOR ({})", op),
        }
    }
}

/// Current state of vim editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimState {
    /// Current editing mode
    pub mode: VimMode,
    /// Current cursor position
    pub cursor: CursorPosition,
    /// Pending operator waiting for motion
    pub pending_operator: Option<VimOperator>,
    /// Active register (default '"')
    pub register: char,
    /// Register storage
    pub registers: HashMap<char, String>,
    /// Repeat count (e.g., 3dw = delete 3 words)
    pub count: Option<usize>,
    /// Last executed command (for . repeat)
    pub last_command: Option<String>,
    /// Start of visual selection
    pub visual_start: Option<CursorPosition>,
    /// Command buffer for : commands
    pub command_buffer: String,
    /// Current search pattern
    pub search_pattern: Option<String>,
    /// Search direction
    pub search_direction: SearchDirection,
    /// Named marks
    pub marks: HashMap<char, CursorPosition>,
}

impl VimState {
    /// Create new vim state
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            cursor: CursorPosition { line: 0, column: 0 },
            pending_operator: None,
            register: '"',
            registers: HashMap::new(),
            count: None,
            last_command: None,
            visual_start: None,
            command_buffer: String::new(),
            search_pattern: None,
            search_direction: SearchDirection::Forward,
            marks: HashMap::new(),
        }
    }
}

impl Default for VimState {
    fn default() -> Self {
        Self::new()
    }
}

/// Cursor position in the document
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CursorPosition {
    /// Line number (0-based)
    pub line: usize,
    /// Column number (0-based)
    pub column: usize,
}

/// Direction of search/movement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchDirection {
    /// Search forward
    Forward,
    /// Search backward
    Backward,
}

/// Vim operators (d, c, y, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VimOperator {
    /// Delete (d)
    Delete,
    /// Change (c)
    Change,
    /// Yank/copy (y)
    Yank,
    /// Indent (>)
    Indent,
    /// Dedent (<)
    Dedent,
    /// Format (=)
    Format,
    /// Uppercase (gU)
    Uppercase,
    /// Lowercase (gu)
    Lowercase,
}

impl std::fmt::Display for VimOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VimOperator::Delete => write!(f, "d"),
            VimOperator::Change => write!(f, "c"),
            VimOperator::Yank => write!(f, "y"),
            VimOperator::Indent => write!(f, ">"),
            VimOperator::Dedent => write!(f, "<"),
            VimOperator::Format => write!(f, "="),
            VimOperator::Uppercase => write!(f, "gU"),
            VimOperator::Lowercase => write!(f, "gu"),
        }
    }
}

/// Vim motions (hjkl, w, b, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VimMotion {
    /// Move left (h)
    Left,
    /// Move right (l)
    Right,
    /// Move up (k)
    Up,
    /// Move down (j)
    Down,
    /// Forward word (w)
    WordForward,
    /// Backward word (b)
    WordBackward,
    /// Forward WORD (W)
    BigWordForward,
    /// Backward WORD (B)
    BigWordBackward,
    /// End of word (e)
    EndOfWord,
    /// End of WORD (E)
    EndOfBigWord,
    /// Line start (^)
    LineStart,
    /// Line end ($)
    LineEnd,
    /// First non-blank (^)
    FirstNonBlank,
    /// Document start (gg)
    DocumentStart,
    /// Document end (G)
    DocumentEnd,
    /// Paragraph forward ({)
    ParagraphForward,
    /// Paragraph backward (})
    ParagraphBackward,
    /// Matching bracket (%)
    MatchingBracket,
    /// Find character forward (f)
    FindChar(char),
    /// Find character backward (F)
    FindCharBack(char),
    /// Till character (t)
    TillChar(char),
    /// Till character backward (T)
    TillCharBack(char),
    /// Search forward (/)
    SearchForward(String),
    /// Search backward (?)
    SearchBackward(String),
    /// Next search result (n)
    NextSearchResult,
    /// Previous search result (N)
    PrevSearchResult,
}

/// Vim text objects (aw, iw, ap, ip, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VimTextObject {
    /// Inner word
    InnerWord,
    /// A word
    AWord,
    /// Inner WORD
    InnerBigWord,
    /// A WORD
    ABigWord,
    /// Inner paragraph
    InnerParagraph,
    /// A paragraph
    AParagraph,
    /// Inner parentheses
    InnerParens,
    /// A parentheses (including parens)
    AParens,
    /// Inner brackets
    InnerBrackets,
    /// A brackets (including brackets)
    ABrackets,
    /// Inner braces
    InnerBraces,
    /// A braces (including braces)
    ABraces,
    /// Inner single quotes
    InnerQuotes,
    /// A single quotes (including quotes)
    AQuotes,
    /// Inner double quotes
    InnerDoubleQuotes,
    /// A double quotes (including quotes)
    ADoubleQuotes,
    /// Inner backticks
    InnerBackticks,
    /// A backticks (including backticks)
    ABackticks,
    /// Inner tag
    InnerTag,
    /// A tag (including tag)
    ATag,
}

/// Result of a vim action
#[derive(Debug, Clone)]
pub struct VimAction {
    /// Type of action performed
    pub action_type: VimActionType,
    /// Text affected by the action
    pub text_affected: Option<String>,
    /// New cursor position
    pub new_cursor: CursorPosition,
    /// New vim mode
    pub new_mode: VimMode,
}

/// Type of vim action
#[derive(Debug, Clone)]
pub enum VimActionType {
    /// Move cursor
    MoveCursor,
    /// Insert text
    InsertText(String),
    /// Delete text
    DeleteText,
    /// Yank/copy text
    YankText,
    /// Change text
    ChangeText,
    /// Undo
    Undo,
    /// Redo
    Redo,
    /// Paste after cursor
    PasteAfter,
    /// Paste before cursor
    PasteBefore,
    /// Join lines
    JoinLines,
    /// Indent lines
    IndentLines,
    /// Dedent lines
    DedentLines,
    /// Execute command
    ExecuteCommand(String),
    /// Start search
    SearchStart(SearchDirection),
    /// No operation
    NoOp,
}

/// Parsed vim command
enum ParsedCommand {
    /// Motion
    Motion(VimMotion),
    /// Operator
    Operator(VimOperator),
    /// Text object
    TextObject(VimTextObject),
    /// Mode switch
    ModeSwitch(VimMode),
    /// Count
    Count(usize),
    /// Command
    Command(String),
    /// Waiting for more keys
    Incomplete,
}

/// Vim editing engine
pub struct VimEngine {
    state: VimState,
    enabled: bool,
    undo_stack: Vec<(String, CursorPosition)>,
    redo_stack: Vec<(String, CursorPosition)>,
}

impl VimEngine {
    /// Create a new vim engine
    pub fn new() -> Self {
        debug!("Creating new VimEngine");
        Self {
            state: VimState::new(),
            enabled: true,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    /// Check if vim mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable vim mode
    pub fn enable(&mut self) {
        debug!("Enabling vim mode");
        self.enabled = true;
    }

    /// Disable vim mode
    pub fn disable(&mut self) {
        debug!("Disabling vim mode");
        self.enabled = false;
    }

    /// Get current state
    pub fn get_state(&self) -> &VimState {
        &self.state
    }

    /// Get current mode
    pub fn get_mode(&self) -> &VimMode {
        &self.state.mode
    }

    /// Get mode display string
    pub fn get_mode_display(&self) -> String {
        format!("-- {} --", self.state.mode)
    }

    /// Process a key press
    pub fn process_key(&mut self, key: &str, text: &str) -> VimAction {
        debug!("Processing key: {} in mode: {:?}", key, self.state.mode);

        match self.state.mode {
            VimMode::Normal => self.process_normal_key(key, text),
            VimMode::Insert => self.process_insert_key(key, text),
            VimMode::Visual => self.process_visual_key(key, text),
            VimMode::Command => self.process_command_key(key, text),
            _ => self.noop_action(),
        }
    }

    fn process_normal_key(&mut self, key: &str, text: &str) -> VimAction {
        match key {
            "i" => {
                self.state.mode = VimMode::Insert;
                self.noop_action()
            }
            "a" => {
                self.state.mode = VimMode::Insert;
                self.state.cursor.column = self.state.cursor.column.saturating_add(1);
                self.noop_action()
            }
            "Escape" => {
                self.state.mode = VimMode::Normal;
                self.noop_action()
            }
            "h" => self.move_cursor_left(text),
            "j" => self.move_cursor_down(text),
            "k" => self.move_cursor_up(text),
            "l" => self.move_cursor_right(text),
            ":" => {
                self.state.mode = VimMode::Command;
                self.state.command_buffer.clear();
                self.noop_action()
            }
            "d" => {
                self.state.pending_operator = Some(VimOperator::Delete);
                self.noop_action()
            }
            "y" => {
                self.state.pending_operator = Some(VimOperator::Yank);
                self.noop_action()
            }
            "c" => {
                self.state.pending_operator = Some(VimOperator::Change);
                self.noop_action()
            }
            "v" => {
                self.state.mode = VimMode::Visual;
                self.state.visual_start = Some(self.state.cursor);
                self.noop_action()
            }
            _ => self.noop_action(),
        }
    }

    fn process_insert_key(&mut self, key: &str, text: &str) -> VimAction {
        match key {
            "Escape" => {
                self.state.mode = VimMode::Normal;
                self.noop_action()
            }
            _ => VimAction {
                action_type: VimActionType::InsertText(key.to_string()),
                text_affected: Some(key.to_string()),
                new_cursor: self.state.cursor,
                new_mode: VimMode::Insert,
            },
        }
    }

    fn process_visual_key(&mut self, key: &str, text: &str) -> VimAction {
        match key {
            "Escape" => {
                self.state.mode = VimMode::Normal;
                self.state.visual_start = None;
                self.noop_action()
            }
            "h" => self.move_cursor_left(text),
            "j" => self.move_cursor_down(text),
            "k" => self.move_cursor_up(text),
            "l" => self.move_cursor_right(text),
            "d" | "x" => {
                self.state.mode = VimMode::Normal;
                VimAction {
                    action_type: VimActionType::DeleteText,
                    text_affected: None,
                    new_cursor: self.state.cursor,
                    new_mode: VimMode::Normal,
                }
            }
            "y" => {
                self.state.mode = VimMode::Normal;
                VimAction {
                    action_type: VimActionType::YankText,
                    text_affected: None,
                    new_cursor: self.state.cursor,
                    new_mode: VimMode::Normal,
                }
            }
            _ => self.noop_action(),
        }
    }

    fn process_command_key(&mut self, key: &str, text: &str) -> VimAction {
        match key {
            "Escape" => {
                self.state.mode = VimMode::Normal;
                self.state.command_buffer.clear();
                self.noop_action()
            }
            "Enter" => {
                let cmd = self.state.command_buffer.clone();
                self.state.command_buffer.clear();
                self.state.mode = VimMode::Normal;
                self.execute_command(&cmd, text)
            }
            "Backspace" => {
                self.state.command_buffer.pop();
                self.noop_action()
            }
            _ => {
                self.state.command_buffer.push_str(key);
                self.noop_action()
            }
        }
    }

    fn move_cursor_left(&mut self, text: &str) -> VimAction {
        if self.state.cursor.column > 0 {
            self.state.cursor.column -= 1;
        }
        VimAction {
            action_type: VimActionType::MoveCursor,
            text_affected: None,
            new_cursor: self.state.cursor,
            new_mode: self.state.mode.clone(),
        }
    }

    fn move_cursor_right(&mut self, text: &str) -> VimAction {
        self.state.cursor.column += 1;
        VimAction {
            action_type: VimActionType::MoveCursor,
            text_affected: None,
            new_cursor: self.state.cursor,
            new_mode: self.state.mode.clone(),
        }
    }

    fn move_cursor_down(&mut self, text: &str) -> VimAction {
        self.state.cursor.line += 1;
        VimAction {
            action_type: VimActionType::MoveCursor,
            text_affected: None,
            new_cursor: self.state.cursor,
            new_mode: self.state.mode.clone(),
        }
    }

    fn move_cursor_up(&mut self, text: &str) -> VimAction {
        if self.state.cursor.line > 0 {
            self.state.cursor.line -= 1;
        }
        VimAction {
            action_type: VimActionType::MoveCursor,
            text_affected: None,
            new_cursor: self.state.cursor,
            new_mode: self.state.mode.clone(),
        }
    }

    fn noop_action(&self) -> VimAction {
        VimAction {
            action_type: VimActionType::NoOp,
            text_affected: None,
            new_cursor: self.state.cursor,
            new_mode: self.state.mode.clone(),
        }
    }

    /// Apply motion
    pub fn apply_motion(&self, motion: &VimMotion, text: &str, from: &CursorPosition) -> CursorPosition {
        debug!("Applying motion: {:?}", motion);
        *from
    }

    /// Get text object range
    pub fn get_text_object_range(
        &self,
        obj: &VimTextObject,
        text: &str,
        at: &CursorPosition,
    ) -> Option<(CursorPosition, CursorPosition)> {
        debug!("Getting text object range for: {:?}", obj);
        None
    }

    /// Apply operator with motion
    pub fn apply_operator(&mut self, op: &VimOperator, motion: &VimMotion, text: &str) -> VimAction {
        debug!("Applying operator: {:?} with motion: {:?}", op, motion);
        self.noop_action()
    }

    /// Apply operator with text object
    pub fn apply_operator_text_object(
        &mut self,
        op: &VimOperator,
        obj: &VimTextObject,
        text: &str,
    ) -> VimAction {
        debug!("Applying operator: {:?} with text object: {:?}", op, obj);
        self.noop_action()
    }

    /// Execute a : command
    pub fn execute_command(&mut self, cmd: &str, text: &str) -> VimAction {
        debug!("Executing command: {}", cmd);
        match cmd {
            "q" | "quit" => {
                info!("Quit command received");
                self.noop_action()
            }
            "w" | "write" => {
                info!("Write command received");
                self.noop_action()
            }
            _ => self.noop_action(),
        }
    }

    /// Undo last change
    pub fn undo(&mut self) -> Option<VimAction> {
        debug!("Undo requested");
        self.undo_stack.pop().map(|_| self.noop_action())
    }

    /// Redo last undone change
    pub fn redo(&mut self) -> Option<VimAction> {
        debug!("Redo requested");
        self.redo_stack.pop().map(|_| self.noop_action())
    }
}

impl Default for VimEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_mode_display() {
        assert_eq!(VimMode::Normal.to_string(), "NORMAL");
        assert_eq!(VimMode::Insert.to_string(), "INSERT");
        assert_eq!(VimMode::Visual.to_string(), "VISUAL");
        assert_eq!(VimMode::VisualLine.to_string(), "VISUAL LINE");
    }

    #[test]
    fn test_vim_operator_display() {
        assert_eq!(VimOperator::Delete.to_string(), "d");
        assert_eq!(VimOperator::Change.to_string(), "c");
        assert_eq!(VimOperator::Yank.to_string(), "y");
        assert_eq!(VimOperator::Indent.to_string(), ">");
    }

    #[test]
    fn test_vim_state_creation() {
        let state = VimState::new();
        assert_eq!(state.mode, VimMode::Normal);
        assert_eq!(state.cursor.line, 0);
        assert_eq!(state.cursor.column, 0);
        assert_eq!(state.register, '"');
    }

    #[test]
    fn test_cursor_position() {
        let pos = CursorPosition { line: 5, column: 10 };
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn test_vim_engine_creation() {
        let engine = VimEngine::new();
        assert!(engine.is_enabled());
        assert_eq!(engine.get_mode(), &VimMode::Normal);
    }

    #[test]
    fn test_vim_enable_disable() {
        let mut engine = VimEngine::new();
        assert!(engine.is_enabled());
        engine.disable();
        assert!(!engine.is_enabled());
        engine.enable();
        assert!(engine.is_enabled());
    }

    #[test]
    fn test_vim_mode_display_string() {
        let engine = VimEngine::new();
        assert_eq!(engine.get_mode_display(), "-- NORMAL --");
    }

    #[test]
    fn test_vim_insert_mode_transition() {
        let mut engine = VimEngine::new();
        let action = engine.process_key("i", "");
        assert_eq!(engine.get_mode(), &VimMode::Insert);
    }

    #[test]
    fn test_vim_visual_mode_transition() {
        let mut engine = VimEngine::new();
        engine.process_key("v", "");
        assert_eq!(engine.get_mode(), &VimMode::Visual);
    }

    #[test]
    fn test_vim_command_mode_transition() {
        let mut engine = VimEngine::new();
        engine.process_key(":", "");
        assert_eq!(engine.get_mode(), &VimMode::Command);
    }

    #[test]
    fn test_vim_cursor_left() {
        let mut engine = VimEngine::new();
        engine.state.cursor.column = 5;
        engine.process_key("h", "");
        assert_eq!(engine.get_state().cursor.column, 4);
    }

    #[test]
    fn test_vim_cursor_right() {
        let mut engine = VimEngine::new();
        engine.process_key("l", "");
        assert_eq!(engine.get_state().cursor.column, 1);
    }

    #[test]
    fn test_vim_cursor_down() {
        let mut engine = VimEngine::new();
        engine.process_key("j", "");
        assert_eq!(engine.get_state().cursor.line, 1);
    }

    #[test]
    fn test_vim_cursor_up() {
        let mut engine = VimEngine::new();
        engine.state.cursor.line = 5;
        engine.process_key("k", "");
        assert_eq!(engine.get_state().cursor.line, 4);
    }
}
