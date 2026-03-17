use crate::model::{
    ExtractedRelation, ExtractedRelationKind, ExtractedSymbol, Language, SemanticDocument, Span,
    SymbolKind,
};
use crate::storage::CortexError;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tree_sitter::{Node, Parser, TreeCursor};

pub trait SemanticExtractor: Send + Sync {
    fn language(&self) -> Language;
    fn extract(&self, path: &Path, source: &str) -> Result<SemanticDocument, CortexError>;
}

#[derive(Clone)]
pub struct DefaultExtractorRegistry {
    extractors: BTreeMap<Language, Arc<dyn SemanticExtractor>>,
}

impl Default for DefaultExtractorRegistry {
    fn default() -> Self {
        let extractors: [(Language, Arc<dyn SemanticExtractor>); 28] = [
            (
                Language::JavaScript,
                Arc::new(TreeSitterExtractor::new(
                    Language::JavaScript,
                    ParserLanguage::JavaScript,
                )),
            ),
            (
                Language::TypeScript,
                Arc::new(TreeSitterExtractor::new(
                    Language::TypeScript,
                    ParserLanguage::TypeScript,
                )),
            ),
            (
                Language::Python,
                Arc::new(TreeSitterExtractor::new(
                    Language::Python,
                    ParserLanguage::Python,
                )),
            ),
            (
                Language::Go,
                Arc::new(TreeSitterExtractor::new(Language::Go, ParserLanguage::Go)),
            ),
            (
                Language::Rust,
                Arc::new(TreeSitterExtractor::new(
                    Language::Rust,
                    ParserLanguage::Rust,
                )),
            ),
            (
                Language::Java,
                Arc::new(TreeSitterExtractor::new(
                    Language::Java,
                    ParserLanguage::Java,
                )),
            ),
            (
                Language::Kotlin,
                Arc::new(TreeSitterExtractor::new(
                    Language::Kotlin,
                    ParserLanguage::Kotlin,
                )),
            ),
            (
                Language::CSharp,
                Arc::new(TreeSitterExtractor::new(
                    Language::CSharp,
                    ParserLanguage::CSharp,
                )),
            ),
            (
                Language::C,
                Arc::new(TreeSitterExtractor::new(Language::C, ParserLanguage::C)),
            ),
            (
                Language::Cpp,
                Arc::new(TreeSitterExtractor::new(Language::Cpp, ParserLanguage::Cpp)),
            ),
            (
                Language::Swift,
                Arc::new(TreeSitterExtractor::new(
                    Language::Swift,
                    ParserLanguage::Swift,
                )),
            ),
            (
                Language::ObjectiveC,
                Arc::new(TreeSitterExtractor::new(
                    Language::ObjectiveC,
                    ParserLanguage::ObjectiveC,
                )),
            ),
            (
                Language::Ruby,
                Arc::new(TreeSitterExtractor::new(
                    Language::Ruby,
                    ParserLanguage::Ruby,
                )),
            ),
            (
                Language::Php,
                Arc::new(TreeSitterExtractor::new(Language::Php, ParserLanguage::Php)),
            ),
            (
                Language::Scala,
                Arc::new(TreeSitterExtractor::new(
                    Language::Scala,
                    ParserLanguage::Scala,
                )),
            ),
            (
                Language::Elixir,
                Arc::new(TreeSitterExtractor::new(
                    Language::Elixir,
                    ParserLanguage::Elixir,
                )),
            ),
            (
                Language::Erlang,
                Arc::new(TreeSitterExtractor::new(
                    Language::Erlang,
                    ParserLanguage::Erlang,
                )),
            ),
            (
                Language::Dart,
                Arc::new(TreeSitterExtractor::new(
                    Language::Dart,
                    ParserLanguage::Dart,
                )),
            ),
            (
                Language::Lua,
                Arc::new(TreeSitterExtractor::new(Language::Lua, ParserLanguage::Lua)),
            ),
            (
                Language::R,
                Arc::new(TreeSitterExtractor::new(Language::R, ParserLanguage::R)),
            ),
            (
                Language::Julia,
                Arc::new(TreeSitterExtractor::new(
                    Language::Julia,
                    ParserLanguage::Julia,
                )),
            ),
            (
                Language::Haskell,
                Arc::new(TreeSitterExtractor::new(
                    Language::Haskell,
                    ParserLanguage::Haskell,
                )),
            ),
            (
                Language::Ocaml,
                Arc::new(TreeSitterExtractor::new(
                    Language::Ocaml,
                    ParserLanguage::Ocaml,
                )),
            ),
            (
                Language::Clojure,
                Arc::new(TreeSitterExtractor::new(
                    Language::Clojure,
                    ParserLanguage::Clojure,
                )),
            ),
            (
                Language::Bash,
                Arc::new(TreeSitterExtractor::new(
                    Language::Bash,
                    ParserLanguage::Bash,
                )),
            ),
            (
                Language::Html,
                Arc::new(TreeSitterExtractor::new(
                    Language::Html,
                    ParserLanguage::Html,
                )),
            ),
            (
                Language::Css,
                Arc::new(TreeSitterExtractor::new(Language::Css, ParserLanguage::Css)),
            ),
            (
                Language::Yaml,
                Arc::new(TreeSitterExtractor::new(
                    Language::Yaml,
                    ParserLanguage::Yaml,
                )),
            ),
        ];

        Self {
            extractors: extractors.into_iter().collect(),
        }
    }
}

impl DefaultExtractorRegistry {
    pub fn for_language(
        &self,
        language: Language,
    ) -> Result<Arc<dyn SemanticExtractor>, CortexError> {
        self.extractors.get(&language).cloned().ok_or_else(|| {
            CortexError::Parser(format!("missing extractor for {}", language.as_str()))
        })
    }

    pub fn supported_languages(&self) -> Vec<Language> {
        self.extractors.keys().copied().collect()
    }
}

#[derive(Clone, Copy)]
enum ParserLanguage {
    JavaScript,
    TypeScript,
    Python,
    Go,
    Rust,
    Java,
    Kotlin,
    CSharp,
    C,
    Cpp,
    Swift,
    ObjectiveC,
    Ruby,
    Php,
    Scala,
    Elixir,
    Erlang,
    Dart,
    Lua,
    R,
    Julia,
    Haskell,
    Ocaml,
    Clojure,
    Bash,
    Html,
    Css,
    Yaml,
}

impl ParserLanguage {
    fn configure(self, parser: &mut Parser) -> Result<(), CortexError> {
        match self {
            Self::JavaScript => parser
                .set_language(&tree_sitter_javascript::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::TypeScript => parser
                .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Python => parser
                .set_language(&tree_sitter_python::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Go => parser
                .set_language(&tree_sitter_go::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Rust => parser
                .set_language(&tree_sitter_rust::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Java => parser
                .set_language(&tree_sitter_java::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Kotlin => parser
                .set_language(&tree_sitter_kotlin_codanna::language())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::CSharp => parser
                .set_language(&tree_sitter_c_sharp::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::C => parser
                .set_language(&tree_sitter_c::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Cpp => parser
                .set_language(&tree_sitter_cpp::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Swift => parser
                .set_language(&tree_sitter_swift::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::ObjectiveC => parser
                .set_language(&tree_sitter_objc::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Ruby => parser
                .set_language(&tree_sitter_ruby::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Php => parser
                .set_language(&tree_sitter_php::LANGUAGE_PHP.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Scala => parser
                .set_language(&tree_sitter_scala::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Elixir => parser
                .set_language(&tree_sitter_elixir::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Erlang => parser
                .set_language(&tree_sitter_erlang::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Dart => parser
                .set_language(&tree_sitter_dart::language())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Lua => parser
                .set_language(&tree_sitter_lua::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::R => parser
                .set_language(&tree_sitter_r::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Julia => parser
                .set_language(&tree_sitter_julia::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Haskell => parser
                .set_language(&tree_sitter_haskell::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Ocaml => parser
                .set_language(&tree_sitter_ocaml::LANGUAGE_OCAML.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Clojure => parser
                .set_language(&tree_sitter_clojure::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Bash => parser
                .set_language(&tree_sitter_bash::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Html => parser
                .set_language(&tree_sitter_html::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Css => parser
                .set_language(&tree_sitter_css::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
            Self::Yaml => parser
                .set_language(&tree_sitter_yaml::LANGUAGE.into())
                .map_err(|error| CortexError::Parser(error.to_string())),
        }
    }
}

#[derive(Clone)]
struct TreeSitterExtractor {
    language: Language,
    parser_language: ParserLanguage,
}

impl TreeSitterExtractor {
    fn new(language: Language, parser_language: ParserLanguage) -> Self {
        Self {
            language,
            parser_language,
        }
    }
}

impl SemanticExtractor for TreeSitterExtractor {
    fn language(&self) -> Language {
        self.language
    }

    fn extract(&self, path: &Path, source: &str) -> Result<SemanticDocument, CortexError> {
        let mut parser = Parser::new();
        self.parser_language.configure(&mut parser)?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| CortexError::Parser("failed to produce parse tree".to_owned()))?;

        let mut collector = SymbolCollector::new(self.language, path);
        collector.walk(tree.root_node(), source);
        Ok(collector.finish())
    }
}

struct SymbolCollector<'a> {
    language: Language,
    path: &'a Path,
    symbols: Vec<ExtractedSymbol>,
    relations: Vec<ExtractedRelation>,
    stack: Vec<String>,
}

impl<'a> SymbolCollector<'a> {
    fn new(language: Language, path: &'a Path) -> Self {
        Self {
            language,
            path,
            symbols: Vec::new(),
            relations: Vec::new(),
            stack: Vec::new(),
        }
    }

    fn finish(self) -> SemanticDocument {
        SemanticDocument {
            language: self.language,
            path: self.path.to_path_buf(),
            symbols: self.symbols,
            relations: self.relations,
        }
    }

    fn walk(&mut self, node: Node<'_>, source: &str) {
        if let Some((symbol, skip_name_range)) = self.extract_symbol(node, source) {
            let local_id = symbol.local_id.clone();
            self.symbols.push(symbol);
            self.stack.push(local_id);
            self.walk_children(node, source, skip_name_range);
            self.stack.pop();
            return;
        }

        if self.extract_import(node, source) {
            return;
        }

        if self.extract_call(node, source) {
            self.walk_children(node, source, None);
            return;
        }

        if self.extract_reference(node, source) {
            return;
        }

        self.walk_children(node, source, None);
    }

    fn walk_children(&mut self, node: Node<'_>, source: &str, skip_range: Option<(usize, usize)>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if skip_range
                .is_some_and(|(start, end)| child.start_byte() == start && child.end_byte() == end)
            {
                continue;
            }
            self.walk(child, source);
        }
    }

    fn extract_symbol(
        &self,
        node: Node<'_>,
        source: &str,
    ) -> Option<(ExtractedSymbol, Option<(usize, usize)>)> {
        let (kind, name_node) = match self.language {
            Language::JavaScript | Language::TypeScript => match node.kind() {
                "function_declaration" => (SymbolKind::Function, node.child_by_field_name("name")?),
                "class_declaration" => (SymbolKind::Class, node.child_by_field_name("name")?),
                "method_definition" => (SymbolKind::Method, node.child_by_field_name("name")?),
                "lexical_declaration" | "variable_declaration" => {
                    let declarator = first_named_child_by_kind(node, "variable_declarator")?;
                    let identifier = declarator.child_by_field_name("name")?;
                    let symbol_kind = if parent_keyword_is_const(node, source) {
                        SymbolKind::Constant
                    } else {
                        SymbolKind::Variable
                    };
                    (symbol_kind, identifier)
                }
                _ => return None,
            },
            Language::Python => match node.kind() {
                "function_definition" => (SymbolKind::Function, node.child_by_field_name("name")?),
                "class_definition" => (SymbolKind::Class, node.child_by_field_name("name")?),
                _ => return None,
            },
            Language::Go => match node.kind() {
                "function_declaration" => (SymbolKind::Function, node.child_by_field_name("name")?),
                "method_declaration" => (SymbolKind::Method, node.child_by_field_name("name")?),
                "type_spec" => {
                    let name = node.child_by_field_name("name")?;
                    let symbol_kind = match node.child_by_field_name("type")?.kind() {
                        "interface_type" => SymbolKind::Interface,
                        _ => SymbolKind::Type,
                    };
                    (symbol_kind, name)
                }
                _ => return None,
            },
            Language::Rust => match node.kind() {
                "function_item" => (SymbolKind::Function, node.child_by_field_name("name")?),
                "struct_item" | "enum_item" | "type_item" => {
                    (SymbolKind::Type, node.child_by_field_name("name")?)
                }
                "trait_item" => (SymbolKind::Trait, node.child_by_field_name("name")?),
                "const_item" => (SymbolKind::Constant, node.child_by_field_name("name")?),
                "impl_item" => (SymbolKind::Type, node.child_by_field_name("type")?),
                "mod_item" => (SymbolKind::Module, node.child_by_field_name("name")?),
                _ => return None,
            },
            Language::Java
            | Language::Kotlin
            | Language::CSharp
            | Language::C
            | Language::Cpp
            | Language::Swift
            | Language::ObjectiveC
            | Language::Ruby
            | Language::Php
            | Language::Scala
            | Language::Elixir
            | Language::Erlang
            | Language::Dart
            | Language::Lua
            | Language::R
            | Language::Julia
            | Language::Haskell
            | Language::Ocaml
            | Language::Clojure
            | Language::Bash
            | Language::Html
            | Language::Css
            | Language::Yaml => match node.kind() {
                "function_declaration" | "function_definition" | "method_declaration" => {
                    (SymbolKind::Function, node.child_by_field_name("name")?)
                }
                "method_definition" => (SymbolKind::Method, node.child_by_field_name("name")?),
                "class_declaration" | "class_definition" => {
                    (SymbolKind::Class, node.child_by_field_name("name")?)
                }
                "interface_declaration" | "interface_definition" => {
                    (SymbolKind::Interface, node.child_by_field_name("name")?)
                }
                "struct_declaration" | "struct_item" => {
                    (SymbolKind::Type, node.child_by_field_name("name")?)
                }
                "enum_declaration" | "enum_item" => {
                    (SymbolKind::Type, node.child_by_field_name("name")?)
                }
                "type_alias_declaration" | "type_item" => {
                    (SymbolKind::Type, node.child_by_field_name("name")?)
                }
                "const_declaration" | "constant_declaration" => {
                    (SymbolKind::Constant, node.child_by_field_name("name")?)
                }
                _ => return None,
            },
        };

        let name = node_text(name_node, source)?;
        let parent_local_id = self.stack.last().cloned();
        let local_id = format!(
            "{}:{}:{}:{}",
            kind_label(kind),
            name,
            node.start_position().row + 1,
            node.start_position().column + 1
        );
        let fq_name = parent_local_id
            .as_ref()
            .and_then(|parent| {
                self.symbols
                    .iter()
                    .find(|symbol| symbol.local_id == *parent)
            })
            .map(|parent| {
                format!(
                    "{}::{}",
                    parent.fq_name.as_deref().unwrap_or(&parent.name),
                    name
                )
            })
            .or_else(|| Some(name.clone()));
        let symbol = ExtractedSymbol {
            local_id,
            name,
            fq_name,
            kind,
            span: span_for(node),
            parent_local_id,
        };
        Some((symbol, Some((name_node.start_byte(), name_node.end_byte()))))
    }

    fn extract_import(&mut self, node: Node<'_>, source: &str) -> bool {
        let module_name = match self.language {
            Language::JavaScript | Language::TypeScript => match node.kind() {
                "import_statement" => last_string_literal(node, source),
                _ => None,
            },
            Language::Python => match node.kind() {
                "import_statement" | "import_from_statement" => last_identifier_like(node, source),
                _ => None,
            },
            Language::Go => match node.kind() {
                "import_spec" => last_string_literal(node, source),
                _ => None,
            },
            Language::Rust => match node.kind() {
                "use_declaration" => first_identifier_like(node, source),
                _ => None,
            },
            Language::Java
            | Language::Kotlin
            | Language::CSharp
            | Language::C
            | Language::Cpp
            | Language::Swift
            | Language::ObjectiveC
            | Language::Ruby
            | Language::Php
            | Language::Scala
            | Language::Elixir
            | Language::Erlang
            | Language::Dart
            | Language::Lua
            | Language::R
            | Language::Julia
            | Language::Haskell
            | Language::Ocaml
            | Language::Clojure
            | Language::Bash
            | Language::Html
            | Language::Css
            | Language::Yaml => match node.kind() {
                "import_declaration" | "using_declaration" | "import_from" | "include" => {
                    last_identifier_like(node, source)
                }
                _ => None,
            },
        };

        let Some(target_name) = module_name else {
            return false;
        };

        self.relations.push(ExtractedRelation {
            kind: ExtractedRelationKind::Import,
            source_local_id: self.stack.last().cloned(),
            target_name,
            span: span_for(node),
            reason: "import".to_owned(),
        });
        true
    }

    fn extract_call(&mut self, node: Node<'_>, source: &str) -> bool {
        let target_name = match node.kind() {
            "call_expression" | "call" | "function_call" | "invocation_expression" => node
                .child_by_field_name("function")
                .or_else(|| node.child_by_field_name("callee"))
                .or_else(|| node.child_by_field_name("name"))
                .and_then(|function| last_identifier_in_subtree(function, source)),
            _ => None,
        };

        let Some(target_name) = target_name else {
            return false;
        };

        self.relations.push(ExtractedRelation {
            kind: ExtractedRelationKind::Call,
            source_local_id: self.stack.last().cloned(),
            target_name,
            span: span_for(node),
            reason: "call".to_owned(),
        });
        true
    }

    fn extract_reference(&mut self, node: Node<'_>, source: &str) -> bool {
        if self.stack.is_empty() {
            return false;
        }

        let is_identifier = matches!(
            node.kind(),
            "identifier"
                | "type_identifier"
                | "field_identifier"
                | "property_identifier"
                | "variable_identifier"
                | "constant_identifier"
                | "scoped_identifier"
        );
        if !is_identifier {
            return false;
        }

        let parent_kind = node.parent().map(|parent| parent.kind());
        if matches!(
            parent_kind,
            Some(
                "function_declaration"
                    | "function_definition"
                    | "class_declaration"
                    | "class_definition"
                    | "method_definition"
                    | "method_declaration"
                    | "type_spec"
                    | "struct_item"
                    | "enum_item"
                    | "trait_item"
                    | "type_item"
                    | "const_item"
                    | "import_spec"
                    | "interface_declaration"
                    | "interface_definition"
                    | "type_alias_declaration"
                    | "struct_declaration"
                    | "enum_declaration"
                    | "const_declaration"
                    | "constant_declaration"
            )
        ) {
            return false;
        }

        let Some(target_name) = node_text(node, source) else {
            return false;
        };

        self.relations.push(ExtractedRelation {
            kind: ExtractedRelationKind::Reference,
            source_local_id: self.stack.last().cloned(),
            target_name,
            span: span_for(node),
            reason: "identifier".to_owned(),
        });
        true
    }
}

fn first_named_child_by_kind<'a>(node: Node<'a>, kind: &str) -> Option<Node<'a>> {
    let mut cursor: TreeCursor<'a> = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn node_text(node: Node<'_>, source: &str) -> Option<String> {
    let text = source.get(node.byte_range())?.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.trim_matches('"').trim_matches('\'').to_owned())
    }
}

fn span_for(node: Node<'_>) -> Span {
    Span {
        start_line: node.start_position().row + 1,
        start_column: node.start_position().column + 1,
        end_line: node.end_position().row + 1,
        end_column: node.end_position().column + 1,
    }
}

fn kind_label(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Method => "method",
        SymbolKind::Class => "class",
        SymbolKind::Type => "type",
        SymbolKind::Interface => "interface",
        SymbolKind::Trait => "trait",
        SymbolKind::Protocol => "protocol",
        SymbolKind::Variable => "variable",
        SymbolKind::Constant => "constant",
        SymbolKind::Module => "module",
        SymbolKind::Package => "package",
    }
}

fn parent_keyword_is_const(node: Node<'_>, source: &str) -> bool {
    node_text(node, source)
        .map(|text| text.starts_with("const "))
        .unwrap_or(false)
}

fn last_string_literal(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter(|child| {
            matches!(
                child.kind(),
                "string" | "interpreted_string_literal" | "string_literal"
            )
        })
        .filter_map(|child| node_text(child, source))
        .last()
}

fn first_identifier_like(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor).find_map(|child| {
        if matches!(
            child.kind(),
            "identifier" | "type_identifier" | "scoped_identifier" | "dotted_name"
        ) {
            node_text(child, source)
        } else {
            first_identifier_like(child, source)
        }
    })
}

fn last_identifier_like(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter_map(|child| {
            if matches!(
                child.kind(),
                "identifier"
                    | "type_identifier"
                    | "dotted_name"
                    | "aliased_import"
                    | "namespace_import"
                    | "scoped_identifier"
                    | "variable_identifier"
                    | "constant_identifier"
            ) {
                node_text(child, source)
            } else {
                last_identifier_like(child, source)
            }
        })
        .last()
}

fn last_identifier_in_subtree(node: Node<'_>, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .filter_map(|child| {
            if matches!(
                child.kind(),
                "identifier"
                    | "type_identifier"
                    | "field_identifier"
                    | "property_identifier"
                    | "scoped_identifier"
                    | "variable_identifier"
                    | "constant_identifier"
            ) {
                node_text(child, source)
            } else {
                last_identifier_in_subtree(child, source)
            }
        })
        .last()
        .or_else(|| {
            if matches!(
                node.kind(),
                "identifier"
                    | "type_identifier"
                    | "field_identifier"
                    | "property_identifier"
                    | "scoped_identifier"
                    | "variable_identifier"
                    | "constant_identifier"
            ) {
                node_text(node, source)
            } else {
                None
            }
        })
}
