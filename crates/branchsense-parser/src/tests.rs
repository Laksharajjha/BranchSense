use std::{any::Any, sync::Arc};

use branchsense_core::{Language, Position, Range};

use crate::{
    LanguageAdapter, ParseError, ParseInput, ParsedDocument, Parser, ParserConfiguration,
    ParserRegistry, SyntaxTree,
};

#[derive(Debug)]
struct TestTree;

impl SyntaxTree for TestTree {
    fn language(&self) -> Language {
        Language::Java
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
struct TestParser {
    configuration: ParserConfiguration,
}

impl Parser for TestParser {
    fn language(&self) -> Language {
        Language::Java
    }
    fn configuration(&self) -> &ParserConfiguration {
        &self.configuration
    }
    fn parse_source(&self, input: ParseInput) -> Result<crate::ParseResult, ParseError> {
        input.validate(self.configuration.max_source_bytes())?;
        let tree = Arc::new(TestTree);
        Ok(crate::ParseResult::new(
            ParsedDocument::new(input, tree, self.configuration.retain_source()),
            Vec::new(),
        ))
    }
}

struct TestAdapter;

impl LanguageAdapter for TestAdapter {
    fn language(&self) -> Language {
        Language::Java
    }
    fn create_parser(
        &self,
        configuration: &ParserConfiguration,
    ) -> Result<Arc<dyn Parser>, ParseError> {
        Ok(Arc::new(TestParser { configuration: configuration.clone() }))
    }
}

#[test]
fn registry_registers_and_returns_an_adapter_parser() {
    let registry = ParserRegistry::new(ParserConfiguration::default());

    registry.register_adapter(&TestAdapter).expect("adapter registration succeeds");
    let parser = registry.get(Language::Java).expect("registered parser is available");
    let result = parser
        .parse_source(ParseInput::new(
            "Example.java",
            "class Example {}",
            crate::DocumentVersion::new(4),
        ))
        .expect("source parses");

    assert_eq!(result.document().language(), Language::Java);
    assert_eq!(result.document().version().value(), 4);
    assert_eq!(result.document().source(), "class Example {}");
    assert!(result.document().syntax_tree().as_any().downcast_ref::<TestTree>().is_some());
}

#[test]
fn registry_rejects_duplicate_language_registration() {
    let registry = ParserRegistry::new(ParserConfiguration::default());
    registry
        .register(Arc::new(TestParser { configuration: ParserConfiguration::default() }))
        .expect("first registration succeeds");

    let error = registry
        .register(Arc::new(TestParser { configuration: ParserConfiguration::default() }))
        .expect_err("duplicate registration fails");

    assert!(matches!(error, crate::RegistryError::AlreadyRegistered(Language::Java)));
}

#[test]
fn incremental_fallback_is_explicitly_unsupported() {
    let parser = TestParser { configuration: ParserConfiguration::default() };
    let input =
        ParseInput::new("Example.java", "class Example {}", crate::DocumentVersion::initial());
    let document = parser.parse_source(input).expect("source parses").into_document();
    let edit = crate::TextEdit::new(
        Range::new(Position::new(0, 0, 0), Position::new(0, 0, 0)).expect("valid range"),
        "// edit\n",
    );

    let error = parser
        .parse_incremental(&document, &[edit])
        .expect_err("default incremental path is unsupported");

    assert!(matches!(error, ParseError::Unsupported { operation: "incremental parsing", .. }));
}
