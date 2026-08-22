#![allow(missing_docs)]

use branchsense_core::{DocumentId, QualifiedName, RevisionId, SymbolId};
use branchsense_extractor_java::JavaExtractor;
use branchsense_graph::{EdgeKind, SemanticGraph};
use branchsense_java::JavaParser;
use branchsense_parser::{DocumentVersion, ParseInput, Parser, ParserConfiguration};
use branchsense_query::{Query, QueryError, QueryNode, QueryOptions};
use branchsense_semantic::DocumentFactSet;

fn extract(path: &str, source: &str) -> DocumentFactSet {
    let parser = JavaParser::new(ParserConfiguration::default()).expect("Java grammar loads");
    let parsed = parser
        .parse_source(ParseInput::new(path, source, DocumentVersion::initial()))
        .expect("source parses");
    DocumentFactSet::new(
        DocumentId::new(path).expect("document ID"),
        JavaExtractor::new().extract(parsed.document()).expect("facts extract").facts().clone(),
    )
}

fn fixture() -> SemanticGraph {
    SemanticGraph::from_documents(
        RevisionId::new("revision:query-fixture").expect("revision ID"),
        vec![
            extract("User.java", "package billing; public class User {}"),
            extract(
                "PaymentValidator.java",
                "package billing; public class PaymentValidator { public void validate() {} }",
            ),
            extract(
                "PaymentService.java",
                "package billing; import billing.PaymentValidator; public class PaymentService { private PaymentValidator validator; public void process() { validator.validate(); } }",
            ),
            extract(
                "PaymentController.java",
                "package billing; import billing.PaymentService; public class PaymentController { private PaymentService service; public void submit() { service.process(); } }",
            ),
        ],
    )
    .expect("fixture graph builds")
}

fn symbol_id(query: &Query<'_>, name: &str) -> SymbolId {
    query.symbols_by_name(name).items()[0].id().clone()
}

#[test]
fn queries_follow_the_real_java_pipeline() {
    let graph = fixture();
    let query = Query::new(&graph);
    let process = symbol_id(&query, "process");

    let incoming_calls = query.callers(&process, QueryOptions::new()).expect("callers");
    assert!(incoming_calls.is_empty());

    let outgoing_calls = query.callees(&process, QueryOptions::new()).expect("callees");
    assert!(outgoing_calls.items().iter().any(|result| result.kind() == EdgeKind::Calls
        && matches!(result.target(), QueryNode::Unresolved { .. })));

    let qualified = query
        .symbol_by_qualified_name(
            &QualifiedName::new("billing.PaymentService.process()").expect("name"),
        )
        .expect("qualified symbol");
    assert_eq!(qualified.id(), &process);
}

#[test]
fn lookup_and_traversal_are_deterministic_and_bounded() {
    let graph = fixture();
    let query = Query::new(&graph);
    let process = symbol_id(&query, "process");
    let all = query.symbols_by_name("process");
    assert_eq!(all.len(), 1);
    let limited = query
        .dependency_tree(&process, 2, QueryOptions::new().with_limit(1))
        .expect("bounded dependencies");
    assert_eq!(limited.len(), 0);
    assert!(matches!(
        query.symbol_by_qualified_name(&QualifiedName::new("missing.Type").expect("name")),
        Err(QueryError::SymbolNotFound { .. })
    ));
}
