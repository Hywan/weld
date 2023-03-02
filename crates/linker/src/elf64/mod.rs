use std::{io, num::NonZeroUsize};

use async_channel::unbounded;
use futures_lite::future::block_on;
use thiserror::Error;
use weld_file::{FileReader, Picker as FilePicker};
use weld_scheduler::ThreadPool;

use crate::Configuration;

#[derive(Debug, Error)]
pub enum Error {
    #[error("thread pool has failed: {0}")]
    ThreadPool(io::Error),

    #[error("thread pool's sender channel has been closed prematuraly")]
    ThreadPoolChannelClosed,

    #[error("parsing an object has failed: {0}")]
    ObjectParser(weld_object::errors::Error<()>),
}

pub(crate) fn link(configuration: Configuration) -> Result<(), Error> {
    // SAFETY: It's OK to `unwrap` as 4 is not 0.
    let thread_pool = ThreadPool::new(NonZeroUsize::new(4).unwrap()).map_err(Error::ThreadPool)?;

    let (sender, receiver) = unbounded::<Result<(), Error>>();

    for input_file_name in configuration.input_files {
        let sender = sender.clone();

        thread_pool
            .execute(async move {
                let work = async move {
                    dbg!(&input_file_name);
                    let input_file = FilePicker::open(input_file_name).unwrap();

                    let file_content = input_file.read_as_bytes().await.unwrap();
                    let bytes: &[u8] = file_content.as_ref();
                    dbg!(weld_object::elf64::File::parse(bytes).map_err(Error::ObjectParser)?);
                    dbg!(std::thread::current().name());

                    Ok(())
                };

                sender
                    .send(work.await)
                    .await
                    .expect("work' sender channel has been closed prematuraly");
            })
            .map_err(|_| Error::ThreadPoolChannelClosed)?;
    }

    drop(sender);

    block_on(async {
        while let Ok(received) = receiver.recv().await {
            dbg!(&received);
        }

        Ok(())
    })
}
