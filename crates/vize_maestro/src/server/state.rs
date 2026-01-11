//! Server state management.

use dashmap::DashMap;
use parking_lot::RwLock;
use tower_lsp::lsp_types::Url;

use crate::document::DocumentStore;
use crate::virtual_code::{VirtualCodeGenerator, VirtualDocuments};

/// Server state containing all runtime data.
pub struct ServerState {
    /// Document store for managing open documents
    pub documents: DocumentStore,
    /// Virtual code generator (reusable)
    virtual_gen: RwLock<VirtualCodeGenerator>,
    /// Cached virtual documents per file
    virtual_docs_cache: DashMap<Url, VirtualDocuments>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerState {
    /// Create a new server state.
    pub fn new() -> Self {
        Self {
            documents: DocumentStore::new(),
            virtual_gen: RwLock::new(VirtualCodeGenerator::new()),
            virtual_docs_cache: DashMap::new(),
        }
    }

    /// Generate and cache virtual documents for a document.
    pub fn update_virtual_docs(&self, uri: &Url, content: &str) {
        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string(),
            ..Default::default()
        };

        if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) {
            let base_uri = uri.path();
            let virtual_docs = self.virtual_gen.write().generate(&descriptor, base_uri);
            self.virtual_docs_cache.insert(uri.clone(), virtual_docs);
        }
    }

    /// Get cached virtual documents for a document.
    pub fn get_virtual_docs(
        &self,
        uri: &Url,
    ) -> Option<dashmap::mapref::one::Ref<'_, Url, VirtualDocuments>> {
        self.virtual_docs_cache.get(uri)
    }

    /// Remove cached virtual documents when a document is closed.
    pub fn remove_virtual_docs(&self, uri: &Url) {
        self.virtual_docs_cache.remove(uri);
    }

    /// Clear all cached virtual documents.
    pub fn clear_virtual_docs(&self) {
        self.virtual_docs_cache.clear();
    }
}
