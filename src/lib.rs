//! verdictloop: every proposed setup change as a
//! tracked experiment with a baseline, a result, and
//! a verdict (VL-1 to VL-12).
//!
//! Round 2 is the Rust port. The JSON reader, the
//! value model, and the timestamp grammar are
//! consumed from the flighttape crate at a pinned
//! revision (VL-14), never re-implemented here; this
//! crate carries the registry itself. The registry
//! format is verdictloop's own. Round 3 adds the
//! judge (VL-20 to VL-26): the one digest the crate
//! consumes is canonical form 1 over the decision
//! card a contested close puts to the model.

pub mod cli;
pub mod emit;
pub mod helptext;
pub mod judge;
pub mod pyrepr;
pub mod registry;
pub mod repr_table;
pub mod util;

pub use registry::{
    add_result, close_experiment, event_defects, fold, list_view, load_registry, open_experiment,
    show_view, suggested_verdict, RegError, RegistryError,
};

/// The command line entry, in the cli module.
pub fn run_cli() -> i32 {
    // A generous stack, on the flighttape crate's own
    // precedent: parsed values nest as deep as their
    // files, and cloning or dropping a deep value
    // recurses with its depth.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(crate::cli::run_cli)
        .expect("the CLI thread starts");
    child.join().unwrap_or(1)
}
