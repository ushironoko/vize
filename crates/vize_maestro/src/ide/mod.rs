//! IDE features for the LSP server.
//!
//! This module provides core IDE functionality including:
//! - Diagnostics aggregation from multiple sources
//! - Hover information provider
//! - Code completion provider
//! - Go to definition
//! - Find references
//! - Code actions (quick fixes)
//! - Type checking and type information
//! - Rename refactoring
//! - Semantic tokens
//! - Code lens
//! - Workspace symbols

pub mod code_action;
pub mod code_lens;
pub mod completion;
pub mod definition;
pub mod diagnostics;
pub mod hover;
pub mod references;
pub mod rename;
pub mod semantic_tokens;
pub mod type_service;
pub mod workspace_symbols;

pub use code_action::CodeActionService;
pub use code_lens::CodeLensService;
pub use completion::{trigger_characters, CompletionService, TRIGGER_CHARACTERS};
pub use definition::{BindingKind, BindingLocation, DefinitionService};
pub use diagnostics::{sources, DiagnosticBuilder, DiagnosticService, Severity};
pub use hover::{HoverBuilder, HoverService};
pub use references::ReferencesService;
pub use rename::RenameService;
pub use semantic_tokens::{SemanticTokensService, TokenModifier, TokenType};
pub use type_service::TypeService;
pub use workspace_symbols::WorkspaceSymbolsService;

use tower_lsp::lsp_types::Url;

use crate::server::ServerState;
use crate::virtual_code::{find_block_at_offset, BlockType, VirtualDocuments};

/// Context for IDE operations.
pub struct IdeContext<'a> {
    /// Server state
    pub state: &'a ServerState,
    /// Document URI
    pub uri: &'a Url,
    /// Document content
    pub content: String,
    /// Cursor offset in the document
    pub offset: usize,
    /// Which block the cursor is in
    pub block_type: Option<BlockType>,
    /// Virtual documents for this file
    pub virtual_docs: Option<dashmap::mapref::one::Ref<'a, Url, VirtualDocuments>>,
}

impl<'a> IdeContext<'a> {
    /// Create a new IDE context.
    pub fn new(state: &'a ServerState, uri: &'a Url, offset: usize) -> Option<Self> {
        let doc = state.documents.get(uri)?;
        let content = doc.text();

        // Parse SFC to determine block type
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string(),
            ..Default::default()
        };

        let block_type = if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&content, options) {
            find_block_at_offset(&descriptor, offset)
        } else {
            None
        };

        let virtual_docs = state.get_virtual_docs(uri);

        Some(Self {
            state,
            uri,
            content,
            offset,
            block_type,
            virtual_docs,
        })
    }

    /// Check if cursor is in template block.
    #[inline]
    pub fn is_in_template(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Template))
    }

    /// Check if cursor is in script block.
    #[inline]
    pub fn is_in_script(&self) -> bool {
        matches!(
            self.block_type,
            Some(BlockType::Script) | Some(BlockType::ScriptSetup)
        )
    }

    /// Check if cursor is in style block.
    #[inline]
    pub fn is_in_style(&self) -> bool {
        matches!(self.block_type, Some(BlockType::Style(_)))
    }
}
