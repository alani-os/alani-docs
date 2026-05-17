//! Documentation lint contracts.
//!
//! Lints are bounded and deterministic so CI, release tooling, and no-std
//! tests can share the same finding vocabulary.

use crate::{validate_path, DocsError, DocsResult, MAX_DOC_LABEL_LEN};

/// Lint severity.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LintSeverity {
    /// Informational finding.
    Info = 1,
    /// Warning that should be reviewed.
    Warning = 2,
    /// Error that should fail CI.
    Error = 3,
}

impl LintSeverity {
    /// Stable severity label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Stable lint finding code.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LintCode {
    /// Document control section is missing.
    MissingDocumentControl = 1,
    /// Purpose section is missing.
    MissingPurpose = 2,
    /// Acceptance criteria section is missing.
    MissingAcceptanceCriteria = 3,
    /// Document contains unresolved TODO marker.
    TodoMarker = 4,
    /// Diagram fence or source looks malformed.
    DiagramSyntax = 5,
    /// Document path is invalid.
    InvalidPath = 6,
}

impl LintCode {
    /// Default severity for this code.
    pub const fn severity(self) -> LintSeverity {
        match self {
            Self::TodoMarker => LintSeverity::Warning,
            _ => LintSeverity::Error,
        }
    }

    /// Stable code label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::MissingDocumentControl => "missing_document_control",
            Self::MissingPurpose => "missing_purpose",
            Self::MissingAcceptanceCriteria => "missing_acceptance_criteria",
            Self::TodoMarker => "todo_marker",
            Self::DiagramSyntax => "diagram_syntax",
            Self::InvalidPath => "invalid_path",
        }
    }
}

/// One documentation lint finding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LintFinding<'a> {
    /// Stable finding code.
    pub code: LintCode,
    /// Finding severity.
    pub severity: LintSeverity,
    /// Repository-relative path.
    pub path: &'a str,
    /// One-based line number, or zero when not available.
    pub line: u32,
    /// Stable message label.
    pub message: &'static str,
}

impl<'a> LintFinding<'a> {
    /// Creates a finding with default severity.
    pub const fn new(code: LintCode, path: &'a str, line: u32) -> Self {
        Self {
            code,
            severity: code.severity(),
            path,
            line,
            message: code.label(),
        }
    }

    /// Validates finding metadata.
    pub fn validate(self) -> DocsResult<()> {
        validate_path(self.path)?;
        if self.message.is_empty() || self.message.len() > MAX_DOC_LABEL_LEN {
            return Err(DocsError::MissingField);
        }
        Ok(())
    }
}

/// Fixed-capacity lint report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LintReport<'a, const N: usize> {
    findings: [Option<LintFinding<'a>>; N],
    len: usize,
}

impl<'a, const N: usize> LintReport<'a, N> {
    /// Creates an empty report.
    pub const fn new() -> Self {
        Self {
            findings: [None; N],
            len: 0,
        }
    }

    /// Returns the number of findings.
    pub const fn len(self) -> usize {
        self.len
    }

    /// Returns `true` when the report has no findings.
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Adds a finding.
    pub fn add(&mut self, finding: LintFinding<'a>) -> DocsResult<()> {
        if self.len >= N {
            return Err(DocsError::CapacityExceeded);
        }
        finding.validate()?;
        self.findings[self.len] = Some(finding);
        self.len += 1;
        Ok(())
    }

    /// Returns a finding by index.
    pub fn finding(self, index: usize) -> Option<LintFinding<'a>> {
        if index < self.len {
            self.findings[index]
        } else {
            None
        }
    }

    /// Returns `true` when any finding is an error.
    pub fn has_errors(self) -> bool {
        let mut index = 0;
        while index < self.len {
            if let Some(finding) = self.findings[index] {
                if matches!(finding.severity, LintSeverity::Error) {
                    return true;
                }
            }
            index += 1;
        }
        false
    }
}

impl<'a, const N: usize> Default for LintReport<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Lints a markdown document for the minimum required Alani sections.
pub fn lint_markdown<'a, const N: usize>(
    path: &'a str,
    markdown: &str,
) -> DocsResult<LintReport<'a, N>> {
    let mut report = LintReport::new();
    if validate_path(path).is_err() {
        report.add(LintFinding::new(LintCode::InvalidPath, path, 0))?;
        return Ok(report);
    }
    if !markdown.contains("## Document Control") {
        report.add(LintFinding::new(LintCode::MissingDocumentControl, path, 0))?;
    }
    if !markdown.contains("## Purpose") {
        report.add(LintFinding::new(LintCode::MissingPurpose, path, 0))?;
    }
    if !markdown.contains("## Acceptance Criteria") {
        report.add(LintFinding::new(
            LintCode::MissingAcceptanceCriteria,
            path,
            0,
        ))?;
    }
    if markdown.contains("TODO") || markdown.contains("todo!") {
        report.add(LintFinding::new(LintCode::TodoMarker, path, 0))?;
    }
    if markdown.contains("```mermaid") && !markdown.contains("```") {
        report.add(LintFinding::new(LintCode::DiagramSyntax, path, 0))?;
    }
    Ok(report)
}

/// Module boundary descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintDescriptor<'a> {
    /// Human-readable descriptor name.
    pub name: &'a str,
    /// Descriptor version.
    pub version: u32,
}

impl<'a> LintDescriptor<'a> {
    /// Creates a lint descriptor.
    pub const fn new(name: &'a str, version: u32) -> Self {
        Self { name, version }
    }
}
