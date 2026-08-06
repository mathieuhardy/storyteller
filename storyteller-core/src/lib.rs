//! # storyteller-core
//!
//! Business core of **Storyteller**: markdown parsing, link resolution, index.
//! No network I/O, no UI — both binaries (`storyteller-server`,
//! `storyteller-tauri`) are thin wrappers over this crate, which is what keeps
//! their behaviour identical (`docs/architecture.md` §2).
//!
//! ## The five golden rules, in code terms
//!
//! 1. **Markdown is the sole source of truth.** [`project`] reads the files;
//!    everything else derives from what it returns.
//! 2. **Never break a file edited elsewhere.** [`parse`] keeps the body
//!    byte-exact and hands back the original YAML text for surgical rewriting.
//! 3. **The index is a disposable cache.** [`index`] can be deleted and rebuilt
//!    at any time; on divergence, the file wins.
//! 4. **Local-first.** Nothing here opens a socket.
//! 5. **Tolerance for imperfect data.** Broken YAML, unknown type or dangling
//!    link produce [`error::Diagnostic`]s attached to the entry, never a
//!    failed read.
//!
//! ## Typical flow
//!
//! ```no_run
//! use storyteller_core::{index::Index, project::Project};
//!
//! let project = Project::open("/path/to/my-novel")?;
//! let mut index = Index::open(project.root())?;
//! index.rebuild_from_project(&project)?;
//!
//! // Lists and backlinks come from the index…
//! let backlinks = index.backlinks("aria-solane")?;
//! // …but a full entry is re-read from disk, the authority.
//! let entry = project.read_entry("characters/aria-solane.md")?;
//! # Ok::<(), storyteller_core::error::Error>(())
//! ```

pub mod config;
pub mod error;
pub mod index;
pub mod links;
pub mod model;
pub mod normalize;
pub mod parse;
pub mod project;
pub mod snapshot;
pub mod types;
pub mod write;

pub use error::{Diagnostic, Error, Result};
pub use model::{Entry, EntrySummary, Frontmatter};
pub use project::Project;

/// Version of this crate, reported by `GET /api/v1/version`.
pub const CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
