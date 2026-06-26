//! Shared domain types for the nationwide telecom/power outage aggregator.
//!
//! These types are storage-agnostic and connector-agnostic. They model the
//! normalized schema described in the project plan: events, the areas they
//! affect, and the source registry that classifies each provider's
//! publication scope.

pub mod error;
pub mod event;
pub mod source;

pub use error::ConnectorError;
pub use event::{
    CustomerScope, EventArea, EventKind, EventStatus, NormalizedEvent, VisibilityStatus,
};
pub use source::{Family, PublicationScope, RetrievalMode, SourceMeta, SourceType};
