//! Architecture decision record contracts.
//!
//! ADRs capture reviewed design decisions. This module provides fixed-capacity
//! indexes and validation helpers without owning storage or touching the
//! filesystem.

use crate::{
    validate_label, validate_title, DocsError, DocsResult, DocumentControl, DocumentKind,
    DocumentStatus, MAX_DOC_LABEL_LEN,
};

/// ADR lifecycle status.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdrStatus {
    /// Proposed but not yet accepted.
    Proposed = 1,
    /// Accepted and current.
    Accepted = 2,
    /// Superseded by another ADR.
    Superseded = 3,
    /// Deprecated but retained for history.
    Deprecated = 4,
}

impl AdrStatus {
    /// Stable ADR status label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
        }
    }

    /// Converts ADR status to generic document status.
    pub const fn document_status(self) -> DocumentStatus {
        match self {
            Self::Proposed => DocumentStatus::Review,
            Self::Accepted => DocumentStatus::Accepted,
            Self::Superseded => DocumentStatus::Superseded,
            Self::Deprecated => DocumentStatus::Deprecated,
        }
    }
}

/// Borrowed ADR record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdrRecord<'a> {
    /// Numeric ADR identifier. Zero is invalid.
    pub id: u32,
    /// ADR title.
    pub title: &'a str,
    /// Owning team or role.
    pub owner: &'a str,
    /// ADR status.
    pub status: AdrStatus,
    /// Context summary.
    pub context: &'a str,
    /// Decision summary.
    pub decision: &'a str,
    /// Consequence summary.
    pub consequences: &'a str,
    /// Superseded ADR id, or zero when not applicable.
    pub supersedes: u32,
    /// Superseding ADR id, or zero when not applicable.
    pub superseded_by: u32,
}

impl<'a> AdrRecord<'a> {
    /// Creates an ADR record.
    pub const fn new(
        id: u32,
        title: &'a str,
        owner: &'a str,
        status: AdrStatus,
        context: &'a str,
        decision: &'a str,
        consequences: &'a str,
    ) -> Self {
        Self {
            id,
            title,
            owner,
            status,
            context,
            decision,
            consequences,
            supersedes: 0,
            superseded_by: 0,
        }
    }

    /// Marks this ADR as superseding another ADR.
    pub const fn supersedes(mut self, id: u32) -> Self {
        self.supersedes = id;
        self
    }

    /// Marks this ADR as superseded by another ADR.
    pub const fn superseded_by(mut self, id: u32) -> Self {
        self.superseded_by = id;
        self
    }

    /// Returns the generic document control block for this ADR.
    pub const fn document_control(self, path: &'a str, version: &'a str) -> DocumentControl<'a> {
        DocumentControl::new(
            self.title,
            "adr",
            DocumentKind::Adr,
            self.status.document_status(),
            self.owner,
            version,
            path,
        )
    }

    /// Validates required ADR fields and status relationships.
    pub fn validate(self) -> DocsResult<()> {
        if self.id == 0 {
            return Err(DocsError::MissingField);
        }
        validate_title(self.title)?;
        validate_label(self.owner, MAX_DOC_LABEL_LEN)?;
        validate_title(self.context)?;
        validate_title(self.decision)?;
        validate_title(self.consequences)?;
        if matches!(self.status, AdrStatus::Superseded) && self.superseded_by == 0 {
            return Err(DocsError::InvalidStatus);
        }
        if !matches!(self.status, AdrStatus::Superseded) && self.superseded_by != 0 {
            return Err(DocsError::InvalidStatus);
        }
        Ok(())
    }
}

/// Fixed-capacity ADR index.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdrIndex<'a, const N: usize> {
    records: [Option<AdrRecord<'a>>; N],
    len: usize,
}

impl<'a, const N: usize> AdrIndex<'a, N> {
    /// Creates an empty ADR index.
    pub const fn new() -> Self {
        Self {
            records: [None; N],
            len: 0,
        }
    }

    /// Returns the number of indexed ADRs.
    pub const fn len(self) -> usize {
        self.len
    }

    /// Returns `true` when no ADRs are indexed.
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Adds a validated ADR record.
    pub fn add(&mut self, record: AdrRecord<'a>) -> DocsResult<()> {
        if self.len >= N {
            return Err(DocsError::CapacityExceeded);
        }
        record.validate()?;
        if self.find(record.id).is_some() {
            return Err(DocsError::Duplicate);
        }
        self.records[self.len] = Some(record);
        self.len += 1;
        Ok(())
    }

    /// Finds an ADR by id.
    pub fn find(self, id: u32) -> Option<AdrRecord<'a>> {
        let mut index = 0;
        while index < self.len {
            if let Some(record) = self.records[index] {
                if record.id == id {
                    return Some(record);
                }
            }
            index += 1;
        }
        None
    }

    /// Counts ADRs that are not superseded or deprecated.
    pub fn current_count(self) -> usize {
        let mut count = 0;
        let mut index = 0;
        while index < self.len {
            if let Some(record) = self.records[index] {
                if matches!(record.status, AdrStatus::Proposed | AdrStatus::Accepted) {
                    count += 1;
                }
            }
            index += 1;
        }
        count
    }
}

impl<'a, const N: usize> Default for AdrIndex<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Module boundary descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdrDescriptor<'a> {
    /// Human-readable descriptor name.
    pub name: &'a str,
    /// Descriptor version.
    pub version: u32,
}

impl<'a> AdrDescriptor<'a> {
    /// Creates an ADR descriptor.
    pub const fn new(name: &'a str, version: u32) -> Self {
        Self { name, version }
    }
}
