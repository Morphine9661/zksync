//! Module with utilities for prover scaler service.

// Built-in deps
use std::time::{Duration, Instant};
// Workspace deps
use models::config_options::parse_env;
use storage::ConnectionPool;

/// Disable the prover service after 5 minutes with no blocks to generate.
const PROVER_DISABLE_THRESHOLD: Duration = Duration::from_secs(5 * 60);

/// Scaler oracle provides information for prover scaler
/// service about required amount of provers for server
/// to operate optimally.
///
/// Important note: this oracle does not maintain it state in a separate
/// thread, the state is updated upon request. For oracle to operate
/// optimally, it is expected that `provers_required` method will be polled
/// with a predictable (and not too big) intervals.
pub struct ScalerOracle {
    /// Last moment in time when a block for proving was
    /// available.
    ///
    /// As shutting the prover down is an expensive operation,
    /// we don't want to do it every time we have no blocks.
    /// Instead, we wait for some time to ensure that load level
    /// decreased, and only then report the scaler that it should
    /// reduce amount of provers.
    last_time_with_blocks: Instant,

    /// Database access to gather the information about amount of
    /// pending blocks.
    db: ConnectionPool,
}

impl ScalerOracle {
    pub fn new(db: ConnectionPool) -> Self {
        Self {
            last_time_with_blocks: Instant::now(),
            db,
        }
    }

    /// Decides how many prover entities should be created depending on the amount of existing
    /// entities and the amount of pending blocks.
    pub fn provers_required(&mut self, working_provers_count: u32) -> Result<u32, failure::Error> {
        // Currently the logic of this method is very simple:
        // If we have no provers and there are no pending blocks, do not start the prover.
        // If we have an active prover and it's been a while since we've had a block, stop the prover.
        // Otherwise, require 1 prover to operate (either start or retain existing one).
        //
        // Later this algorithm will be improved to provide better scalability, but for now the
        // simplest possible solution is preferred.

        let storage = self.db.access_storage()?;
        let pending_jobs = storage.prover_schema().pending_jobs_count()?;
        let idle_provers: u64 = parse_env("IDLE_PROVERS");
        let provers_required = pending_jobs + idle_provers;

        Ok(provers_required as u32)
    }
}
