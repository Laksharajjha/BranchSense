#![allow(missing_docs)]

use branchsense_core::{DocumentId, RevisionId};
use branchsense_extractor_java::JavaExtractor;
use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use branchsense_semantic::{DocumentFactSet, SemanticFact, SymbolKind};

use branchsense_graph::{EdgeKind, SemanticGraph};

fn extract(path: &str, source: &str) -> DocumentFactSet {
    let parser = JavaParser::new(ParserConfiguration::default()).expect("Java grammar loads");
    let parsed = parser
        .parse_source(ParseInput::new(path, source, DocumentVersion::initial()))
        .expect("source parses");
    let facts =
        JavaExtractor::new().extract(parsed.document()).expect("facts extract").facts().clone();
    DocumentFactSet::new(DocumentId::new(path).expect("document ID"), facts)
}

#[test]
fn payment_repository_builds_from_four_java_documents() {
    let documents = vec![
        extract("User.java", "package billing; public class User {}"),
        extract(
            "UserRepository.java",
            "package billing; import billing.User; public class UserRepository { public User find() { return new User(); } }",
        ),
        extract(
            "PaymentService.java",
            "package billing; import billing.UserRepository; public class PaymentService { private UserRepository repository; public void process() { repository.find(); } }",
        ),
        extract(
            "PaymentController.java",
            "package billing; import billing.PaymentService; public class PaymentController { private PaymentService service; public void create() { service.process(); } }",
        ),
    ];
    let graph = SemanticGraph::from_documents(
        RevisionId::new("revision:fixture").expect("revision ID"),
        documents,
    )
    .expect("repository graph builds");

    assert_eq!(graph.statistics().documents(), 4);
    assert!(graph.statistics().symbols() >= 8);
    assert!(graph.edges().any(|edge| edge.kind() == EdgeKind::Imports));
    assert!(graph.edges().any(|edge| edge.kind() == EdgeKind::Calls));
    assert!(graph.edges().any(|edge| edge.kind() == EdgeKind::DependsOn));
    assert!(graph.statistics().unresolved() > 0);
    assert!(
        graph
            .nodes()
            .any(|node| { node.symbol_kind().is_some_and(|kind| kind == SymbolKind::Type) })
    );
    assert!(
        graph
            .document_facts(&DocumentId::new("PaymentController.java").expect("document ID"))
            .is_some()
    );
    assert!(graph.edges().any(|edge| matches!(
        edge.resolution(),
        Some(branchsense_semantic::ResolutionState::Unresolved)
    )));
    assert!(
        graph
            .document_facts(&DocumentId::new("User.java").expect("document ID"))
            .expect("user facts")
            .facts()
            .iter()
            .any(|record| matches!(record.fact(), SemanticFact::Definition(_)))
    );
}
