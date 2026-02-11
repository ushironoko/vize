//! Import processing utilities.
//!
//! This module handles processing import statements, including
//! removing TypeScript type-only imports and extracting identifiers.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Process import statement to remove TypeScript type-only imports using OXC
/// Returns None if the entire import should be removed, Some(processed) otherwise
pub fn process_import_for_types(import: &str) -> Option<String> {
    let import = import.trim();

    // Parse the import statement with OXC
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parser = Parser::new(&allocator, import, source_type);
    let result = parser.parse();

    if result.errors.is_empty() {
        for stmt in &result.program.body {
            if let Statement::ImportDeclaration(decl) = stmt {
                // Skip type-only imports: import type { ... } from '...'
                if decl.import_kind.is_type() {
                    return None;
                }

                // Check if there are any specifiers
                if let Some(specifiers) = &decl.specifiers {
                    // Filter out type-only specifiers
                    let value_specifiers: Vec<&ImportDeclarationSpecifier> = specifiers
                        .iter()
                        .filter(|spec| match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                !s.import_kind.is_type()
                            }
                            _ => true,
                        })
                        .collect();

                    if value_specifiers.is_empty() {
                        // All specifiers were type imports
                        return None;
                    }

                    if value_specifiers.len() != specifiers.len() {
                        // Some specifiers were filtered out, rebuild the import
                        let source = decl.source.value.as_str();

                        // Separate default/namespace imports from named imports
                        let mut default_part: Option<String> = None;
                        let mut named_parts: Vec<String> = Vec::new();

                        for spec in &value_specifiers {
                            match spec {
                                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                    let imported = s.imported.name().as_str();
                                    let local = s.local.name.as_str();
                                    if imported == local {
                                        named_parts.push(imported.to_string());
                                    } else {
                                        named_parts.push(format!("{} as {}", imported, local));
                                    }
                                }
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                    default_part = Some(s.local.name.to_string());
                                }
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                    default_part =
                                        Some(format!("* as {}", s.local.name.as_str()));
                                }
                            }
                        }

                        let mut new_import = String::with_capacity(64);
                        new_import.push_str("import ");
                        if let Some(ref def) = default_part {
                            new_import.push_str(def);
                            if !named_parts.is_empty() {
                                new_import.push_str(", ");
                            }
                        }
                        if !named_parts.is_empty() {
                            new_import.push_str("{ ");
                            new_import.push_str(&named_parts.join(", "));
                            new_import.push_str(" }");
                        }
                        new_import.push_str(" from '");
                        new_import.push_str(source);
                        new_import.push_str("'\n");
                        return Some(new_import);
                    }
                }
            }
        }
    }

    // Regular import or parse failed, return as-is
    Some(import.to_string() + "\n")
}

/// Extract all identifiers from an import statement (including default imports)
pub fn extract_import_identifiers(import: &str) -> Vec<String> {
    let import = import.trim();
    let mut identifiers = Vec::new();

    // Parse the import statement with OXC
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let parser = Parser::new(&allocator, import, source_type);
    let result = parser.parse();

    if result.errors.is_empty() {
        for stmt in &result.program.body {
            if let Statement::ImportDeclaration(decl) = stmt {
                // Skip type-only imports
                if decl.import_kind.is_type() {
                    continue;
                }

                if let Some(specifiers) = &decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                // Skip type-only specifiers
                                if !s.import_kind.is_type() {
                                    identifiers.push(s.local.name.to_string());
                                }
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                identifiers.push(s.local.name.to_string());
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                identifiers.push(s.local.name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    identifiers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_import_with_type_named_import() {
        // `import Foo, { type Bar }` should become `import Foo from '...'`
        // NOT `import { Foo } from '...'`
        let input = "import AtriumSegmentedTabs, { type AtriumSegmentedTabConfig } from '../AtriumSegmentedTabs/AtriumSegmentedTabs.vue'";
        let result = process_import_for_types(input);
        let output = result.expect("should produce an import");
        assert!(
            output.starts_with("import AtriumSegmentedTabs from"),
            "Default import should be preserved as default import, not named. Got: {}",
            output
        );
        assert!(
            !output.contains("{ AtriumSegmentedTabs }"),
            "Default import should NOT be inside braces. Got: {}",
            output
        );
    }

    #[test]
    fn test_default_import_with_mixed_named_imports() {
        // `import Foo, { type Bar, baz }` should become `import Foo, { baz } from '...'`
        let input = "import Foo, { type Bar, baz } from 'module'";
        let result = process_import_for_types(input);
        let output = result.expect("should produce an import");
        assert!(
            output.contains("import Foo, { baz }"),
            "Should have default + named imports. Got: {}",
            output
        );
    }

    #[test]
    fn test_type_only_import_returns_none() {
        let input = "import type { Foo } from 'bar'";
        let result = process_import_for_types(input);
        assert!(result.is_none(), "Type-only import should return None");
    }

    #[test]
    fn test_all_named_type_imports_returns_none() {
        let input = "import { type Foo, type Bar } from 'baz'";
        let result = process_import_for_types(input);
        assert!(
            result.is_none(),
            "All type-only named imports should return None"
        );
    }

    #[test]
    fn test_normal_import_unchanged() {
        let input = "import { foo, bar } from 'module'";
        let result = process_import_for_types(input);
        assert!(
            result.is_some(),
            "Normal import should be returned as-is"
        );
    }
}
