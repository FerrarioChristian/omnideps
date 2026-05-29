use crate::ir::{Import, QualifiedName};
use std::collections::HashMap;

/// A frame representing a single lexical scope (e.g., a Module, a Struct, a Function, or a `{}` block).
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Local symbols defined exactly in this scope. Maps the local name to its absolute QualifiedName.
    pub symbols: HashMap<String, QualifiedName>,
    /// Jump points (import/use statements) available in this scope.
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

/// The Environment (ρ): a dynamic stack of symbol tables reflecting the current lexical context.
#[derive(Debug, Clone)]
pub struct SymbolStack {
    pub frames: Vec<StackFrame>,
}

impl SymbolStack {
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Pushes a new empty lexical scope onto the stack.
    pub fn push_scope(&mut self) {
        self.frames.push(StackFrame::new());
    }

    /// Pops the top lexical scope from the stack, destroying it.
    pub fn pop_scope(&mut self) {
        self.frames.pop();
    }

    /// Retrieves a mutable reference to the current (top-most) frame.
    pub fn current_frame_mut(&mut self) -> Option<&mut StackFrame> {
        self.frames.last_mut()
    }

    /// Defines a local symbol in the current scope, mapping its local name to its absolute path.
    pub fn define_symbol(&mut self, local_name: String, absolute_path: QualifiedName) {
        if let Some(frame) = self.current_frame_mut() {
            frame.symbols.insert(local_name, absolute_path);
        }
    }

    /// Adds an import (jump point) to the current scope.
    pub fn add_import(&mut self, import: Import) {
        if let Some(frame) = self.current_frame_mut() {
            frame.imports.push(import);
        }
    }

    /// Exposes the frames for iterating top-down during resolution.
    pub fn iter_frames_top_down(&self) -> impl Iterator<Item = &StackFrame> {
        self.frames.iter().rev()
    }
}
