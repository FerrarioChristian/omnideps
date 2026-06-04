use std::collections::HashMap;
use crate::model::{Import, Query};

/// A single frame representing a lexical scope during the Query Building phase.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Local symbols defined in this scope. Maps a simple name to its algebraic Query.
    pub symbols: HashMap<String, Query>,
    /// Jump points (imports) available in this scope.
    pub imports: Vec<Import>,
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            imports: Vec::new(),
        }
    }
}

/// The Environment (ρ) used during the Query Building phase to apply substitutions.
#[derive(Debug, Clone)]
pub struct SymbolStack {
    pub frames: Vec<StackFrame>,
}

impl SymbolStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Enters a new lexical scope by pushing an empty frame.
    pub fn push_scope(&mut self) {
        self.frames.push(StackFrame::new());
    }

    /// Exits the current lexical scope, destroying all local symbols.
    pub fn pop_scope(&mut self) {
        self.frames.pop();
    }

    /// Retrieves a mutable reference to the current (top-most) frame.
    pub fn current_frame_mut(&mut self) -> Option<&mut StackFrame> {
        self.frames.last_mut()
    }

    /// Registers a local symbol and its corresponding substitution Query in the current frame.
    pub fn define_symbol(&mut self, local_name: String, query: Query) {
        if let Some(frame) = self.current_frame_mut() {
            frame.symbols.insert(local_name, query);
        }
    }

    /// Adds an import to the current frame.
    pub fn add_import(&mut self, import: Import) {
        if let Some(frame) = self.current_frame_mut() {
            frame.imports.push(import);
        }
    }

    /// Iterates through the stack frames from top (most local) to bottom (global).
    pub fn iter_frames_top_down(&self) -> impl Iterator<Item = &StackFrame> {
        self.frames.iter().rev()
    }
}
