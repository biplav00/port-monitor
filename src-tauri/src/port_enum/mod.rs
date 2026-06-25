pub mod assemble;
pub mod filter;
pub mod kill;
pub mod source;
pub mod types;

pub use filter::FilterOpts;
pub use kill::kill;
pub use types::PortEntry;

use anyhow::Result;
use assemble::assemble;
use filter::apply as apply_filter;
use source::collect_raw;

/// Listening ports as the OS sees them (deduped, current-user resolved, sorted)
/// — unfiltered. Used by the integration test.
pub fn list_listening() -> Result<Vec<PortEntry>> {
    Ok(assemble(collect_raw()?))
}

/// The deep entry point: raw scan → assemble → filter. One call answers
/// "what ports show, given these settings". `list_ports` is a thin adapter over it.
pub fn snapshot(opts: FilterOpts) -> Result<Vec<PortEntry>> {
    Ok(apply_filter(&list_listening()?, opts))
}
