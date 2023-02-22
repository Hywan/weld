use std::num::NonZeroUsize;

use smol::{
    block_on,
    channel::{unbounded, Sender},
    Executor,
};
use weld_scheduler::ThreadPool;

use crate::Configuration;

#[derive(Debug)]
pub struct Linker {
    configuration: Configuration,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}

impl Linker {
    pub(crate) fn with_configuration(configuration: Configuration) -> Self {
        Self { configuration }
    }

    #[must_use]
    pub fn link(self) -> Result<(), Error> {
        Ok(())
    }
}
