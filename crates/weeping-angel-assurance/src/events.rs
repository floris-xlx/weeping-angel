//! Public re-export of immutable ISMS event observations.
//!
//! Persist (when used) is append-only by `eventId`. This module is not a
//! notification bus.

pub use weeping_angel_assurance_ir::{
    EventCauseKind, EventCauseRef, EventId, EventSeverity, EventSubjectKind, EventSubjectRef,
    ISMS_EVENT_SCHEMA, IsmsEvent, IsmsEventKind,
};
