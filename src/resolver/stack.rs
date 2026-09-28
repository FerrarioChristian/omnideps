use crate::model::Query;
use std::collections::HashMap;

/// A single lexical scope frame $\sigma_i$ representing local symbol bindings.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Local symbols defined in this scope frame. Maps an unqualified identifier to its algebraic [`Query`].
    pub symbols: HashMap<String, Query>,
}

impl Default for StackFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl StackFrame {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
}

/// The Lexical Symbol Stack ($\Delta = [\sigma_0, \sigma_1, \dots, \sigma_k]$).
///
/// Used during the Query Building phase ($\rho_{\text{build}}$) to rewrite local terms,
/// variables, and parameter references into algebraic resolution queries ([`Query`]).
#[derive(Debug, Clone)]
pub struct SymbolStack {
    pub frames: Vec<StackFrame>,
}

impl Default for SymbolStack {
    fn default() -> Self {
        Self::new()
    }
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

    /// Iterates through the stack frames from top (most local) to bottom (global).
    pub fn iter_frames_top_down(&self) -> impl Iterator<Item = &StackFrame> {
        self.frames.iter().rev()
    }
}
