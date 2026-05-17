//! Diagram asset and link metadata.
//!
//! The first implementation focuses on Mermaid assets because the Alani spec
//! bundle already uses `.mmd` diagrams. Other formats are represented but not
//! parsed by this dependency-free crate.

use crate::{validate_path, DocsError, DocsResult, MAX_DOC_LABEL_LEN};

/// Diagram source kind.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagramKind {
    /// Mermaid source.
    Mermaid = 1,
    /// Graphviz DOT source.
    Graphviz = 2,
    /// SVG asset.
    Svg = 3,
}

impl DiagramKind {
    /// Stable kind label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mermaid => "mermaid",
            Self::Graphviz => "graphviz",
            Self::Svg => "svg",
        }
    }
}

/// Diagram review or generation status.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagramStatus {
    /// Draft diagram.
    Draft = 1,
    /// Reviewed source.
    Reviewed = 2,
    /// Generated artifact.
    Generated = 3,
}

impl DiagramStatus {
    /// Stable status label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Reviewed => "reviewed",
            Self::Generated => "generated",
        }
    }
}

/// Borrowed diagram source asset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagramAsset<'a> {
    /// Repository-relative asset path.
    pub path: &'a str,
    /// Diagram kind.
    pub kind: DiagramKind,
    /// Source text or generated artifact text.
    pub source: &'a str,
    /// Review or generation status.
    pub status: DiagramStatus,
}

impl<'a> DiagramAsset<'a> {
    /// Creates a diagram asset.
    pub const fn new(
        path: &'a str,
        kind: DiagramKind,
        source: &'a str,
        status: DiagramStatus,
    ) -> Self {
        Self {
            path,
            kind,
            source,
            status,
        }
    }

    /// Validates path and source metadata.
    pub fn validate(self) -> DocsResult<()> {
        validate_path(self.path)?;
        if self.source.is_empty() {
            return Err(DocsError::MissingField);
        }
        if matches!(self.kind, DiagramKind::Mermaid) {
            validate_mermaid_source(self.source)?;
        }
        Ok(())
    }
}

/// Link between a document and a diagram asset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiagramLink<'a> {
    /// Repository-relative document path.
    pub document_path: &'a str,
    /// Repository-relative diagram asset path.
    pub diagram_path: &'a str,
    /// Optional heading anchor.
    pub anchor: &'a str,
}

impl<'a> DiagramLink<'a> {
    /// Creates a diagram link.
    pub const fn new(document_path: &'a str, diagram_path: &'a str) -> Self {
        Self {
            document_path,
            diagram_path,
            anchor: "",
        }
    }

    /// Sets a heading anchor.
    pub const fn with_anchor(mut self, anchor: &'a str) -> Self {
        self.anchor = anchor;
        self
    }

    /// Validates linked paths and optional anchor metadata.
    pub fn validate(self) -> DocsResult<()> {
        validate_path(self.document_path)?;
        validate_path(self.diagram_path)?;
        if !self.anchor.is_empty() {
            crate::validate_label(self.anchor, MAX_DOC_LABEL_LEN)?;
        }
        Ok(())
    }
}

/// Validates a minimal Mermaid source preamble.
pub fn validate_mermaid_source(source: &str) -> DocsResult<()> {
    if source.is_empty() {
        return Err(DocsError::MissingField);
    }
    let trimmed = source.trim_start();
    if trimmed.starts_with("graph ")
        || trimmed.starts_with("flowchart ")
        || trimmed.starts_with("sequenceDiagram")
        || trimmed.starts_with("stateDiagram")
        || trimmed.starts_with("classDiagram")
        || trimmed.starts_with("erDiagram")
        || trimmed.starts_with("journey")
        || trimmed.starts_with("gantt")
        || trimmed.starts_with("pie")
    {
        Ok(())
    } else {
        Err(DocsError::InvalidDiagram)
    }
}

/// Module boundary descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagramDescriptor<'a> {
    /// Human-readable descriptor name.
    pub name: &'a str,
    /// Descriptor version.
    pub version: u32,
}

impl<'a> DiagramDescriptor<'a> {
    /// Creates a diagram descriptor.
    pub const fn new(name: &'a str, version: u32) -> Self {
        Self { name, version }
    }
}
