//! pmd-core: parser for projectmd. Pure functions only (no filesystem, no threads) so the
//! crate can be compiled to WebAssembly.

pub mod date;
pub mod graph;
pub mod lock;
pub mod model;
pub mod parse;
pub mod profile;
pub mod schedule;
pub mod sha256;

pub use date::{parse_date, Civil, DateParse};
pub use graph::{is_remote, normalize, project_links, resolve, split_fragment, Graph, Link, LinkKind};
pub use lock::{build_lock, Lock, LockEntry};
pub use model::*;
pub use parse::parse;
pub use profile::{parse_profile, profile_candidates};
pub use schedule::{schedule, span_of_text, Interface, Interfaces};
pub use sha256::{hash_prefix, sha256_hex};
