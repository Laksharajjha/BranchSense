//! Tree-sitter-backed Java parser implementation.

#![allow(clippy::missing_errors_doc)]

use std::{fmt, sync::Mutex};

use branchsense_core::{Language, Position, Range};
use branchsense_parser::{
    DiagnosticSeverity, ParseDiagnostic, ParseError, ParseInput, ParseResult, ParsedDocument,
    Parser, ParserConfiguration, TextEdit,
};
use tree_sitter::{InputEdit, Parser as TreeSitterParser, Point, Tree};

use crate::JavaSyntaxTree;

/// Java parser backed by Tree-sitter and protected for concurrent use.
pub struct JavaParser {
    configuration: ParserConfiguration,
    parser: Mutex<TreeSitterParser>,
}

impl fmt::Debug for JavaParser {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("JavaParser")
            .field("configuration", &self.configuration)
            .finish_non_exhaustive()
    }
}

impl JavaParser {
    /// Creates a configured Java parser and loads the Java grammar.
    pub fn new(configuration: ParserConfiguration) -> Result<Self, ParseError> {
        let mut parser = TreeSitterParser::new();
        parser.set_language(&tree_sitter_java::LANGUAGE.into()).map_err(|error| {
            ParseError::Adapter { message: format!("failed to load Java grammar: {error}") }
        })?;
        Ok(Self { configuration, parser: Mutex::new(parser) })
    }

    fn parse_tree(&self, source: &str, old_tree: Option<&Tree>) -> Result<Tree, ParseError> {
        let mut parser = self
            .parser
            .lock()
            .map_err(|_| ParseError::Adapter { message: "Java parser lock poisoned".into() })?;
        parser.parse(source, old_tree).ok_or_else(|| ParseError::Adapter {
            message: "Tree-sitter returned no Java syntax tree".into(),
        })
    }

    fn result(&self, input: ParseInput, tree: Tree) -> ParseResult {
        let diagnostics = diagnostics(tree.root_node(), self.configuration.max_diagnostics());
        let syntax_tree = std::sync::Arc::new(JavaSyntaxTree::new(tree));
        ParseResult::new(
            ParsedDocument::new(input, syntax_tree, self.configuration.retain_source()),
            diagnostics,
        )
    }
}

impl Parser for JavaParser {
    fn language(&self) -> Language {
        Language::Java
    }

    fn configuration(&self) -> &ParserConfiguration {
        &self.configuration
    }

    fn parse_source(&self, input: ParseInput) -> Result<ParseResult, ParseError> {
        validate_source(&input, &self.configuration)?;
        let tree = self.parse_tree(input.source(), None)?;
        Ok(self.result(input, tree))
    }

    fn parse_incremental(
        &self,
        previous: &ParsedDocument,
        edits: &[TextEdit],
    ) -> Result<ParseResult, ParseError> {
        if !self.configuration.incremental_enabled() {
            return Err(ParseError::Unsupported {
                language: Language::Java,
                operation: "incremental parsing disabled",
            });
        }
        if previous.language() != Language::Java {
            return Err(ParseError::LanguageMismatch {
                expected: Language::Java,
                actual: previous.language(),
            });
        }
        let previous_tree =
            previous.syntax_tree().as_any().downcast_ref::<JavaSyntaxTree>().ok_or_else(|| {
                ParseError::Adapter {
                    message: "parsed document was not produced by the Java adapter".into(),
                }
            })?;
        let (source, tree) = apply_edits(previous.source(), previous_tree.clone_tree(), edits)?;
        let input = ParseInput::new(previous.path(), source, previous.version().next());
        validate_source(&input, &self.configuration)?;
        let tree = self.parse_tree(input.source(), Some(&tree))?;
        Ok(self.result(input, tree))
    }
}

fn validate_source(
    input: &ParseInput,
    configuration: &ParserConfiguration,
) -> Result<(), ParseError> {
    if input.source().len() > configuration.max_source_bytes() {
        return Err(ParseError::SourceTooLarge {
            actual_bytes: input.source().len(),
            max_bytes: configuration.max_source_bytes(),
        });
    }
    Ok(())
}

fn apply_edits(
    source: &str,
    mut tree: Tree,
    edits: &[TextEdit],
) -> Result<(String, Tree), ParseError> {
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| std::cmp::Reverse(edit.range().start().offset()));
    let mut updated = source.to_owned();
    for edit in &ordered {
        let range = edit.range();
        let start = range.start().offset() as usize;
        let end = range.end().offset() as usize;
        if start > end
            || end > updated.len()
            || !updated.is_char_boundary(start)
            || !updated.is_char_boundary(end)
        {
            return Err(ParseError::Adapter {
                message: "incremental edit range is outside UTF-8 source boundaries".into(),
            });
        }
        let old_end_point = point(range.end());
        let new_end_point = new_end_point(point(range.start()), edit.replacement());
        tree.edit(&InputEdit {
            start_byte: start,
            old_end_byte: end,
            new_end_byte: start + edit.replacement().len(),
            start_position: point(range.start()),
            old_end_position: old_end_point,
            new_end_position: new_end_point,
        });
        updated.replace_range(start..end, edit.replacement());
    }
    Ok((updated, tree))
}

fn point(position: Position) -> Point {
    Point { row: position.line() as usize, column: position.column() as usize }
}

fn new_end_point(start: Point, replacement: &str) -> Point {
    let mut lines = replacement.split('\n');
    let last = lines.next_back().unwrap_or_default();
    Point {
        row: start.row + replacement.bytes().filter(|byte| *byte == b'\n').count(),
        column: if replacement.contains('\n') { last.len() } else { start.column + last.len() },
    }
}

fn diagnostics(node: tree_sitter::Node<'_>, limit: usize) -> Vec<ParseDiagnostic> {
    let mut output = Vec::new();
    collect_diagnostics(node, limit, &mut output);
    output
}

fn collect_diagnostics(
    node: tree_sitter::Node<'_>,
    limit: usize,
    output: &mut Vec<ParseDiagnostic>,
) {
    if output.len() >= limit {
        return;
    }
    if node.is_error() || node.is_missing() {
        let start = Position::new(
            coordinate(node.start_position().row),
            coordinate(node.start_position().column),
            coordinate(node.start_byte()),
        );
        let end = Position::new(
            coordinate(node.end_position().row),
            coordinate(node.end_position().column),
            coordinate(node.end_byte()),
        );
        if let Ok(range) = Range::new(start, end) {
            output.push(ParseDiagnostic::new(
                DiagnosticSeverity::Error,
                format!("Java syntax error near {}", node.kind()),
                Some(range),
            ));
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_diagnostics(child, limit, output);
    }
}

fn coordinate(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::JavaParser;
    use branchsense_core::{Position, Range};
    use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration, TextEdit};

    #[test]
    fn parses_valid_java_and_reports_no_errors() {
        let parser = JavaParser::new(ParserConfiguration::default()).expect("grammar loads");
        let result = parser
            .parse_source(ParseInput::new(
                "Example.java",
                "class Example {}",
                DocumentVersion::initial(),
            ))
            .expect("Java parses");
        assert!(!result.has_errors());
        assert!(
            result
                .document()
                .syntax_tree()
                .as_any()
                .downcast_ref::<crate::JavaSyntaxTree>()
                .is_some()
        );
    }

    #[test]
    fn recovers_from_malformed_java() {
        let parser = JavaParser::new(ParserConfiguration::default()).expect("grammar loads");
        let result = parser
            .parse_source(ParseInput::new(
                "Broken.java",
                "class Broken {",
                DocumentVersion::initial(),
            ))
            .expect("recovery returns a tree");
        assert!(result.has_errors());
    }

    #[test]
    fn increments_version_for_incremental_edit() {
        let parser = JavaParser::new(ParserConfiguration::default()).expect("grammar loads");
        let input = ParseInput::new("Example.java", "class Example {}", DocumentVersion::new(3));
        let initial = parser.parse_source(input).expect("Java parses").into_document();
        let end = Position::new(0, 15, 15);
        let edit = TextEdit::new(Range::new(end, end).expect("valid insertion range"), "\n");
        let updated =
            parser.parse_incremental(&initial, &[edit]).expect("incremental parse succeeds");
        assert_eq!(updated.document().version().value(), 4);
    }
}
