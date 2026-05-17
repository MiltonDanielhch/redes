//! Ubicación: `crates/email/src/lib.rs`
//!
//! Descripción: Cliente de email con Resend y cola async.
//!
//! ADRs relacionados: 0014 (Notificaciones email)

pub mod client;
pub mod templates;
pub mod queue;
pub mod error;

pub use client::ResendClient;
pub use queue::EmailQueue;
pub use error::EmailError;