use alani_docs::{
    docs_catalog, lint_markdown, validate_mermaid_source, AdrIndex, AdrRecord, AdrStatus,
    DiagramAsset, DiagramKind, DiagramLink, DiagramStatus, DocsError, DocumentControl,
    DocumentKind, DocumentStatus, RenderFormat, RenderOptions, RenderRequest, TemplateDescriptor,
    DOCS_SCHEMA_VERSION,
};

fn control() -> DocumentControl<'static> {
    DocumentControl::new(
        "Alani Test Spec",
        "ALANI-TEST-001",
        DocumentKind::Spec,
        DocumentStatus::Draft,
        "docs-team",
        "0.1.0-draft",
        "docs/test.md",
    )
}

#[test]
fn repository_identity_and_catalog_are_stable() {
    let info = alani_docs::component_info();
    assert_eq!(alani_docs::repository_name(), "alani-docs");
    assert_eq!(info.repository, "alani-docs");
    assert_eq!(info.status, alani_docs::ComponentStatus::Experimental);
    assert_eq!(
        alani_docs::module_names(),
        &["adr", "lint", "render", "diagrams"]
    );
    assert_eq!(docs_catalog().schema_version, DOCS_SCHEMA_VERSION);
    assert_eq!(docs_catalog().validate(), Ok(()));
    assert_eq!(docs_catalog().alias, "alani-spec");
}

#[test]
fn document_control_validates_required_metadata() {
    let doc = control();
    assert_eq!(doc.validate(), Ok(()));
    assert_eq!(doc.kind.label(), "spec");
    assert!(doc.status.is_current());

    let bad = DocumentControl::new(
        "Bad",
        "BAD",
        DocumentKind::Spec,
        DocumentStatus::Draft,
        "docs team",
        "0.1.0",
        "/absolute.md",
    );
    assert_eq!(bad.validate(), Err(DocsError::InvalidLabel));
}

#[test]
fn adr_index_rejects_duplicates_and_tracks_current_records() {
    let first = AdrRecord::new(
        1,
        "Use fixed-capacity docs indexes",
        "docs-team",
        AdrStatus::Accepted,
        "Need no_std host contracts.",
        "Use borrowed records.",
        "Host tools own allocation.",
    );
    let second = AdrRecord::new(
        2,
        "Replace old docs renderer",
        "docs-team",
        AdrStatus::Superseded,
        "Renderer changed.",
        "Use new metadata.",
        "Old output remains historical.",
    )
    .superseded_by(3);

    let mut index = AdrIndex::<4>::new();
    index.add(first).unwrap();
    index.add(second).unwrap();
    assert_eq!(index.len(), 2);
    assert_eq!(index.current_count(), 1);
    assert_eq!(index.find(1), Some(first));
    assert_eq!(index.add(first), Err(DocsError::Duplicate));
}

#[test]
fn markdown_lint_reports_missing_sections_and_todo_markers() {
    let report = lint_markdown::<8>(
        "docs/example.md",
        "# Example\n\n## Purpose\n\nTODO: fill this in\n",
    )
    .unwrap();
    assert_eq!(report.len(), 3);
    assert!(report.has_errors());
    assert_eq!(
        report.finding(0).unwrap().code.label(),
        "missing_document_control"
    );

    let clean = lint_markdown::<8>(
        "docs/clean.md",
        "# Clean\n\n## Document Control\n\n## Purpose\n\n## Acceptance Criteria\n",
    )
    .unwrap();
    assert!(clean.is_empty());
}

#[test]
fn render_request_validates_options_and_output_metadata() {
    let request = RenderRequest::new(
        control(),
        "# Alani Test Spec\n\n## Document Control\n\n## Purpose\n",
        RenderOptions::new(RenderFormat::Markdown).with_max_output_bytes(1024),
    );
    let rendered = request.render().unwrap();
    assert_eq!(rendered.title, "Alani Test Spec");
    assert_eq!(rendered.format.label(), "markdown");
    assert_eq!(rendered.validate(), Ok(()));

    let bad_options = RenderOptions {
        format: RenderFormat::JsonSummary,
        include_toc: true,
        redact_sensitive: true,
        max_output_bytes: 0,
    };
    assert_eq!(bad_options.validate(), Err(DocsError::UnsupportedFormat));

    let template = TemplateDescriptor::new("spec");
    assert!(template.sections.contains(&"Acceptance Criteria"));
    assert_eq!(template.validate(), Ok(()));
}

#[test]
fn diagram_assets_validate_mermaid_sources_and_links() {
    let source = "flowchart LR\n  A --> B\n";
    assert_eq!(validate_mermaid_source(source), Ok(()));
    let asset = DiagramAsset::new(
        "docs/assets/test.mmd",
        DiagramKind::Mermaid,
        source,
        DiagramStatus::Reviewed,
    );
    assert_eq!(asset.validate(), Ok(()));

    let link = DiagramLink::new("docs/test.md", "docs/assets/test.mmd").with_anchor("diagram");
    assert_eq!(link.validate(), Ok(()));

    assert_eq!(
        validate_mermaid_source("not a diagram"),
        Err(DocsError::InvalidDiagram)
    );
}
