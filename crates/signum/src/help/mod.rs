//! Signum 3/4 interactive help system.
//!
//! Signum provided help information for its functions through an external `.HLP` file
//! that is loaded at runtime. These associated hardcoded identifiers from the UI elements
//! as well as menu items with help dialog text.
mod hlp;

pub use hlp::{HelpFile, SubTerm, Term};
