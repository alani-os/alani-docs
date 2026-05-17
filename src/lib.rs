#![cfg_attr(not(feature = "std"), no_std)]

//! Source-of-truth documentation contracts for the Alani MVK.
//!
//! `alani-docs` owns small, dependency-free APIs for ADR records, document
//! linting, render requests, diagram metadata, and publication evidence. The
//! crate intentionally avoids filesystem access; callers pass borrowed content
//! and paths so host tools and no-std tests use the same validation rules.

pub mod adr;
pub mod diagrams;
pub mod lint;
pub mod render;

pub use adr::{AdrDescriptor, AdrIndex, AdrRecord, AdrStatus};
pub use diagrams::{
    validate_mermaid_source, DiagramAsset, DiagramDescriptor, DiagramKind, DiagramLink,
    DiagramStatus,
};
pub use lint::{lint_markdown, LintCode, LintDescriptor, LintFinding, LintReport, LintSeverity};
pub use render::{
    RenderDescriptor, RenderFormat, RenderOptions, RenderRequest, RenderedDocument,
    TemplateDescriptor, TEMPLATE_SECTIONS,
};

/// Repository name.
pub const REPOSITORY: &str = "alani-docs";

/// Compatibility alias recorded in the repository spec.
pub const ALIAS_ALANI_SPEC: &str = "alani-spec";

/// Crate version.
pub const VERSION: &str = "0.1.0";

/// Public module names exposed by this crate.
pub const MODULES: &[&str] = &["adr", "lint", "render", "diagrams"];

/// Public documentation metadata schema version.
pub const DOCS_SCHEMA_VERSION: &str = "alani.docs.v1";

/// Feature bit for ADR records and indexes.
pub const DOCS_FEATURE_ADR: u64 = 1 << 0;
/// Feature bit for markdown lint findings.
pub const DOCS_FEATURE_LINT: u64 = 1 << 1;
/// Feature bit for render request metadata.
pub const DOCS_FEATURE_RENDER: u64 = 1 << 2;
/// Feature bit for diagram assets and links.
pub const DOCS_FEATURE_DIAGRAMS: u64 = 1 << 3;
/// Feature bit for publication evidence summaries.
pub const DOCS_FEATURE_PUBLICATION: u64 = 1 << 4;

/// All feature bits known by this crate version.
pub const DOCS_KNOWN_FEATURES: u64 = DOCS_FEATURE_ADR
    | DOCS_FEATURE_LINT
    | DOCS_FEATURE_RENDER
    | DOCS_FEATURE_DIAGRAMS
    | DOCS_FEATURE_PUBLICATION;

/// Maximum label length accepted by documentation metadata.
pub const MAX_DOC_LABEL_LEN: usize = 128;

/// Maximum document title length.
pub const MAX_DOC_TITLE_LEN: usize = 160;

/// Maximum repository-relative path length.
pub const MAX_DOC_PATH_LEN: usize = 256;

/// Result alias used by documentation APIs.
pub type DocsResult<T> = Result<T, DocsError>;

/// Error taxonomy for documentation validation and linting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocsError {
    /// A required field was empty or omitted.
    MissingField,
    /// A bounded string field exceeded its documented limit.
    FieldTooLong,
    /// A label or path contained unsupported characters.
    InvalidLabel,
    /// Document status was incompatible with the operation.
    InvalidStatus,
    /// A required section was missing.
    MissingSection,
    /// A diagram source failed syntax checks.
    InvalidDiagram,
    /// A render format or options combination is unsupported.
    UnsupportedFormat,
    /// A fixed-capacity report or index is full.
    CapacityExceeded,
    /// An identifier already exists.
    Duplicate,
    /// Requested record was not found.
    NotFound,
    /// Feature or reserved bits were supplied.
    ReservedBits,
}

impl DocsError {
    /// Stable reason label for lints, tests, and publication evidence.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::MissingField => "missing_field",
            Self::FieldTooLong => "field_too_long",
            Self::InvalidLabel => "invalid_label",
            Self::InvalidStatus => "invalid_status",
            Self::MissingSection => "missing_section",
            Self::InvalidDiagram => "invalid_diagram",
            Self::UnsupportedFormat => "unsupported_format",
            Self::CapacityExceeded => "capacity_exceeded",
            Self::Duplicate => "duplicate",
            Self::NotFound => "not_found",
            Self::ReservedBits => "reserved_bits",
        }
    }
}

/// Implementation maturity marker for generated repository metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentStatus {
    /// API is present as a draft skeleton.
    Draft,
    /// API is implemented enough for host-mode experimentation.
    Experimental,
    /// API is compatible and stable.
    Stable,
}

/// Stable component identity record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentInfo {
    /// Repository name.
    pub repository: &'static str,
    /// Crate version.
    pub version: &'static str,
    /// Current implementation status.
    pub status: ComponentStatus,
}

/// Returns stable component identity metadata.
pub const fn component_info() -> ComponentInfo {
    ComponentInfo {
        repository: REPOSITORY,
        version: VERSION,
        status: ComponentStatus::Experimental,
    }
}

/// Returns the repository name.
pub const fn repository_name() -> &'static str {
    REPOSITORY
}

/// Returns public module names.
pub fn module_names() -> &'static [&'static str] {
    MODULES
}

/// Documentation artifact kind.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentKind {
    /// Architecture or implementation specification.
    Spec = 1,
    /// Architecture decision record.
    Adr = 2,
    /// Request for comments.
    Rfc = 3,
    /// API reference.
    Api = 4,
    /// Operational runbook.
    Runbook = 5,
    /// Repository README.
    Readme = 6,
}

impl DocumentKind {
    /// Stable kind label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Adr => "adr",
            Self::Rfc => "rfc",
            Self::Api => "api",
            Self::Runbook => "runbook",
            Self::Readme => "readme",
        }
    }
}

/// Review status for documentation artifacts.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentStatus {
    /// Initial draft.
    Draft = 1,
    /// Under review.
    Review = 2,
    /// Accepted and current.
    Accepted = 3,
    /// Superseded by a newer document.
    Superseded = 4,
    /// Deprecated but retained for history.
    Deprecated = 5,
}

impl DocumentStatus {
    /// Stable status label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Review => "review",
            Self::Accepted => "accepted",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
        }
    }

    /// Returns `true` when the status represents a current artifact.
    pub const fn is_current(self) -> bool {
        matches!(self, Self::Draft | Self::Review | Self::Accepted)
    }
}

/// Document control block shared by specs, ADRs, RFCs, API docs, and runbooks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocumentControl<'a> {
    /// Document title.
    pub title: &'a str,
    /// Stable document identifier.
    pub document_id: &'a str,
    /// Document kind.
    pub kind: DocumentKind,
    /// Review status.
    pub status: DocumentStatus,
    /// Owning team or role.
    pub owner: &'a str,
    /// Version string.
    pub version: &'a str,
    /// Repository-relative path.
    pub path: &'a str,
}

impl<'a> DocumentControl<'a> {
    /// Creates a document control block.
    pub const fn new(
        title: &'a str,
        document_id: &'a str,
        kind: DocumentKind,
        status: DocumentStatus,
        owner: &'a str,
        version: &'a str,
        path: &'a str,
    ) -> Self {
        Self {
            title,
            document_id,
            kind,
            status,
            owner,
            version,
            path,
        }
    }

    /// Validates required fields, bounded lengths, and label characters.
    pub fn validate(self) -> DocsResult<()> {
        validate_title(self.title)?;
        validate_label(self.document_id, MAX_DOC_LABEL_LEN)?;
        validate_label(self.owner, MAX_DOC_LABEL_LEN)?;
        validate_label(self.version, MAX_DOC_LABEL_LEN)?;
        validate_path(self.path)?;
        Ok(())
    }
}

/// Compact root view of the documentation crate contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocsCatalog {
    /// Repository name.
    pub repository: &'static str,
    /// Compatibility alias.
    pub alias: &'static str,
    /// Crate version.
    pub version: &'static str,
    /// Documentation schema version.
    pub schema_version: &'static str,
    /// Feature bitmap.
    pub features: u64,
    /// Maximum path length.
    pub max_path_len: usize,
}

impl DocsCatalog {
    /// Current documentation catalog.
    pub const CURRENT: Self = Self {
        repository: REPOSITORY,
        alias: ALIAS_ALANI_SPEC,
        version: VERSION,
        schema_version: DOCS_SCHEMA_VERSION,
        features: DOCS_KNOWN_FEATURES,
        max_path_len: MAX_DOC_PATH_LEN,
    };

    /// Validates catalog metadata.
    pub const fn validate(self) -> DocsResult<()> {
        if self.repository.is_empty()
            || self.alias.is_empty()
            || self.version.is_empty()
            || self.schema_version.is_empty()
        {
            return Err(DocsError::MissingField);
        }
        if self.features & !DOCS_KNOWN_FEATURES != 0 {
            return Err(DocsError::ReservedBits);
        }
        if self.max_path_len == 0 {
            return Err(DocsError::MissingField);
        }
        Ok(())
    }
}

/// Current documentation catalog.
pub const DOCS_CATALOG: DocsCatalog = DocsCatalog::CURRENT;

/// Returns the current documentation catalog.
pub const fn docs_catalog() -> DocsCatalog {
    DocsCatalog::CURRENT
}

/// Validates a bounded documentation title.
pub fn validate_title(title: &str) -> DocsResult<()> {
    if title.is_empty() {
        return Err(DocsError::MissingField);
    }
    if title.len() > MAX_DOC_TITLE_LEN {
        return Err(DocsError::FieldTooLong);
    }
    Ok(())
}

/// Validates a bounded documentation label.
pub fn validate_label(label: &str, max_len: usize) -> DocsResult<()> {
    if label.is_empty() {
        return Err(DocsError::MissingField);
    }
    if label.len() > max_len {
        return Err(DocsError::FieldTooLong);
    }
    if !label.bytes().all(is_label_byte) {
        return Err(DocsError::InvalidLabel);
    }
    Ok(())
}

/// Validates a repository-relative path.
pub fn validate_path(path: &str) -> DocsResult<()> {
    validate_label(path, MAX_DOC_PATH_LEN)?;
    if path.starts_with('/') || path.contains("..") {
        return Err(DocsError::InvalidLabel);
    }
    Ok(())
}

fn is_label_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b':'
            | b'_'
            | b'-'
            | b'.'
            | b'/'
            | b'@'
            | b'#'
    )
}
