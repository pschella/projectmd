//! pmd-core: parser for projectmd. Pure functions only (no filesystem, no threads) so the
//! crate can be compiled to WebAssembly.

pub mod date;
pub mod fmt;
pub mod graph;
pub mod contract;
pub mod iface;
pub mod model;
pub mod parse;
pub mod schedule;

pub use date::{parse_date, Civil, DateParse};
pub use graph::{is_remote, normalize, project_links, resolve, split_fragment, Graph, Link, LinkKind};
pub use iface::{in_interface, interface_items, is_empty_promise, is_private, IfaceItem};
pub use contract::{contract_diags, contracts_of, deliverable_state, project_state, Child, Contract, State};
pub use model::*;
pub use parse::parse;
pub use schedule::{add_forecasts, schedule, span_of_text, Interface, Interfaces};
