use std::{io, num::NonZeroUsize};

use async_channel::unbounded;
use futures_lite::future::block_on;
use weld_errors::error;
use weld_file::{FileReader, Picker as FilePicker};
use weld_scheduler::ThreadPool;

use crate::Configuration;

error! {
    #[doc = "Elf64 errors."]
    pub enum Error {
        #[message = "I was not able to create the thread pool."]
        #[formatted_message("I was not able to create the thread pool: {0}.")]
        #[help = "?"]
        ThreadPool(io::Error),

        #[message = "Hmm, it seems like the thread pool's sender channel has been closed prematuraly."]
        #[help = "?"]
        ThreadPoolChannelClosed,

        #[code = E004]
        #[message = "I was not able to parse an object file correctly."]
        #[formatted_message("I was not able to parse an object file correctly: {0}")]
        #[help = "?"]
        ObjectParser(weld_object::errors::Error<()>),
    }
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
