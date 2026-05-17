//! Render request and template metadata.
//!
//! Rendering is represented as validated metadata plus borrowed output. Real
//! filesystem and publication work belongs in host tooling layered above this
//! crate.

use crate::{validate_title, DocsError, DocsResult, DocumentControl};

/// Required sections from the shared documentation template.
pub const TEMPLATE_SECTIONS: &[&str] = &[
    "Document Control",
    "Purpose",
    "Scope",
    "Requirements",
    "Design / Procedure",
    "Interfaces",
    "Acceptance Criteria",
    "Risks and Open Questions",
];

/// Render output format.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderFormat {
    /// Markdown output.
    Markdown = 1,
    /// HTML fragment output.
    HtmlFragment = 2,
    /// Plain text output.
    PlainText = 3,
    /// JSON summary output.
    JsonSummary = 4,
}

impl RenderFormat {
    /// Stable format label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Markdown => "markdown",
            Self::HtmlFragment => "html_fragment",
            Self::PlainText => "plain_text",
            Self::JsonSummary => "json_summary",
        }
    }
}

/// Render options supplied by host tooling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    /// Output format.
    pub format: RenderFormat,
    /// Include generated table of contents when supported.
    pub include_toc: bool,
    /// Redact sensitive sections before export.
    pub redact_sensitive: bool,
    /// Maximum output bytes. Zero means unspecified.
    pub max_output_bytes: usize,
}

impl RenderOptions {
    /// Conservative markdown defaults.
    pub const MARKDOWN: Self = Self {
        format: RenderFormat::Markdown,
        include_toc: false,
        redact_sensitive: true,
        max_output_bytes: 0,
    };

    /// Creates options for a format.
    pub const fn new(format: RenderFormat) -> Self {
        Self {
            format,
            ..Self::MARKDOWN
        }
    }

    /// Sets the maximum output byte count.
    pub const fn with_max_output_bytes(mut self, max_output_bytes: usize) -> Self {
        self.max_output_bytes = max_output_bytes;
        self
    }

    /// Validates option combinations.
    pub const fn validate(self) -> DocsResult<()> {
        if self.include_toc && matches!(self.format, RenderFormat::JsonSummary) {
            return Err(DocsError::UnsupportedFormat);
        }
        Ok(())
    }
}

/// Borrowed render request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderRequest<'a> {
    /// Document control metadata.
    pub control: DocumentControl<'a>,
    /// Markdown source body.
    pub body: &'a str,
    /// Render options.
    pub options: RenderOptions,
}

impl<'a> RenderRequest<'a> {
    /// Creates a render request.
    pub const fn new(control: DocumentControl<'a>, body: &'a str, options: RenderOptions) -> Self {
        Self {
            control,
            body,
            options,
        }
    }

    /// Validates control metadata, body presence, and options.
    pub fn validate(self) -> DocsResult<()> {
        self.control.validate()?;
        if self.body.is_empty() {
            return Err(DocsError::MissingField);
        }
        self.options.validate()?;
        if self.options.max_output_bytes != 0 && self.body.len() > self.options.max_output_bytes {
            return Err(DocsError::FieldTooLong);
        }
        Ok(())
    }

    /// Produces a borrowed rendered-document record.
    pub fn render(self) -> DocsResult<RenderedDocument<'a>> {
        self.validate()?;
        Ok(RenderedDocument {
            title: self.control.title,
            format: self.options.format,
            body: self.body,
            bytes: self.body.len(),
            redacted: self.options.redact_sensitive,
        })
    }
}

/// Rendered document metadata with borrowed output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderedDocument<'a> {
    /// Rendered title.
    pub title: &'a str,
    /// Output format.
    pub format: RenderFormat,
    /// Borrowed rendered body.
    pub body: &'a str,
    /// Output byte length.
    pub bytes: usize,
    /// Whether sensitive sections were redacted.
    pub redacted: bool,
}

impl<'a> RenderedDocument<'a> {
    /// Validates rendered output metadata.
    pub fn validate(self) -> DocsResult<()> {
        validate_title(self.title)?;
        if self.body.is_empty() || self.bytes != self.body.len() {
            return Err(DocsError::MissingField);
        }
        Ok(())
    }
}

/// Template descriptor for spec, ADR, RFC, API, and runbook documents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemplateDescriptor<'a> {
    /// Template name.
    pub name: &'a str,
    /// Template sections.
    pub sections: &'static [&'static str],
}

impl<'a> TemplateDescriptor<'a> {
    /// Creates a template descriptor.
    pub const fn new(name: &'a str) -> Self {
        Self {
            name,
            sections: TEMPLATE_SECTIONS,
        }
    }

    /// Validates template metadata.
    pub fn validate(self) -> DocsResult<()> {
        validate_title(self.name)?;
        if self.sections.is_empty() {
            return Err(DocsError::MissingSection);
        }
        Ok(())
    }
}

/// Module boundary descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderDescriptor<'a> {
    /// Human-readable descriptor name.
    pub name: &'a str,
    /// Descriptor version.
    pub version: u32,
}

impl<'a> RenderDescriptor<'a> {
    /// Creates a render descriptor.
    pub const fn new(name: &'a str, version: u32) -> Self {
        Self { name, version }
    }
}
