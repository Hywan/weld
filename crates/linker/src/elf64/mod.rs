#![allow(unused)]

use std::{
    borrow::Cow,
    io::{self, Write as _},
    num::NonZeroUsize,
    path::PathBuf,
};

use async_channel::unbounded;
use futures_lite::future::block_on;
use weld_errors::error;
use weld_file::{FileReader, Picker as FilePicker};
use weld_object::{
    elf64::{
        Address, Alignment, Data, DataType, Endianness, File, FileType, Machine, OsAbi, Program,
        ProgramFlag, ProgramFlags, ProgramType, Section, SectionFlag, SectionFlags, SectionIndex,
        SectionType, Version,
    },
    BigEndian, LittleEndian, Number, Write,
};
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
        #[message = "I was not able to parse the object file correctly."]
        #[formatted_message("I was not able to parse the `{0}` object file correctly.")]
        #[help = "?"]
        ParsingFile(PathBuf),

        #[code = E005]
        #[message = "I was not able to parse a symbol correctly."]
        #[formatted_message("I was not able to parse a symbol correctly in `{0}`.")]
        #[help = "?"]
        ParsingSymbol(PathBuf),

        #[code = EOO6]
        #[message = "…"]
        #[help = "?"]
        NotRelocatable(PathBuf),
    }
}

pub(crate) fn link(configuration: Configuration) -> Result<(), Error> {
    // SAFETY: It's OK to `unwrap` as 4 is not 0.
    let thread_pool = ThreadPool::new(NonZeroUsize::new(4).unwrap()).map_err(Error::ThreadPool)?;

    let (sender, receiver) = unbounded::<Result<(), Error>>();

    assert_eq!(
        configuration.input_files.len(),
        1,
        "`weld` doesn't not link more than one file for the moment"
    );

    for input_file_name in configuration.input_files {
        let sender = sender.clone();
        let path_to_output_file = configuration.output_file.clone();

        thread_pool
            .execute(async move {
                let work = async move {
                    dbg!(std::thread::current().name());
                    dbg!(&input_file_name);

                    let input_file = FilePicker::open(&input_file_name).unwrap();

                    let file_content = input_file.read_as_bytes().await.unwrap();
                    let bytes: &[u8] = file_content.as_ref();
                    let (rest, mut object_file) = weld_object::elf64::File::read::<()>(bytes)
                        .map_err(|_| Error::ParsingFile(input_file_name.clone()))?;

                    debug_assert!(
                        rest.is_empty(),
                        "The file `{:?}` has not been read until the end: `{:?}`",
                        input_file_name,
                        rest,
                    );

                    if object_file.r#type != FileType::RelocatableFile {
                        // return Err(Error::NotRelocatable(input_file_name));
                    }

                    // TODO: remove
                    object_file.fetch_section_names();

                    let strings_section = object_file.strings_section();

                    let symbols = object_file
                        .sections
                        .iter()
                        .filter(|section| section.r#type == SectionType::SymbolTable)
                        .flat_map(|section| section.data.symbols::<()>(strings_section).unwrap())
                        .map(|result| {
                            result.map_err(|_| Error::ParsingSymbol(input_file_name.clone()))
                        })
                        .collect::<Result<Vec<_>, Error>>()?;

                    dbg!(&object_file);
                    dbg!(&symbols);

                    let mut file = FileBuilder::new(
                        object_file.endianness,
                        object_file.version,
                        object_file.os_abi,
                        object_file.machine,
                        object_file.processor_flags,
                        object_file.sections.len(),
                    );

                    for section in object_file
                        .sections
                        .iter()
                        .filter(|section| section.r#type == SectionType::ProgramData)
                    {
                        file.push_segment(section.flags.into(), section.data.clone().into_owned());
                    }

                    dbg!(&file);
                    let final_bytes = file.build().unwrap();

                    let (rest, test) = weld_object::elf64::File::read::<()>(&final_bytes).unwrap();
                    dbg!(&test);

                    let mut output_file = std::fs::File::options()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(path_to_output_file)
                        .unwrap();

                    output_file.write_all(&final_bytes).unwrap();

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
            let _ = received?;
        }

        Ok(())
    })
}

#[derive(Debug)]
struct FileBuilder {
    endianness: Endianness,
    version: Version,
    os_abi: OsAbi,
    machine: Machine,
    processor_flags: u32,
    segments: Vec<Segment>,
}

impl FileBuilder {
    fn new(
        endianness: Endianness,
        version: Version,
        os_abi: OsAbi,
        machine: Machine,
        processor_flags: u32,
        number_of_segments_hint: usize,
    ) -> Self {
        Self {
            endianness,
            version,
            os_abi,
            machine,
            processor_flags,
            segments: Vec::with_capacity(number_of_segments_hint),
        }
    }

    fn push_segment(&mut self, section_flags: SectionFlags, data: Vec<u8>) {
        let flags = section_flags
            .iter()
            .map(|section_flag| match section_flag {
                SectionFlag::Allocable => ProgramFlag::Read,
                SectionFlag::Executable => ProgramFlag::Execute,
                SectionFlag::Writable => ProgramFlag::Write,
                flag => unimplemented!("unsupported section flag {flag:?}"),
            })
            .collect::<ProgramFlags>();

        self.segments.push(Segment { flags, data });
    }

    fn build(self) -> io::Result<Vec<u8>> {
        match self.endianness {
            Endianness::Big => self.build_with_endianness::<BigEndian>(),
            Endianness::Little => self.build_with_endianness::<LittleEndian>(),
        }
    }

    fn build_with_endianness<N: Number>(self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(256);

        let file_load_va: u64 = 0x1000;

        // Magic.
        buffer.write_all(File::MAGIC)?;
        // Elf64.
        buffer.write_all(File::ELF64)?;
        // Endianness.
        self.endianness.write::<LittleEndian, _>(&mut buffer)?;
        // Version.
        self.version.write::<N, _>(&mut buffer)?;
        // OS ABI.
        self.os_abi.write::<N, _>(&mut buffer)?;
        // Padding (skip).
        buffer.resize(buffer.len() + 8, 0);
        // File type.
        FileType::ExecutableFile.write::<N, _>(&mut buffer)?;
        // Machine.
        self.machine.write::<N, _>(&mut buffer)?;
        // Version bis (skip).
        buffer.resize(buffer.len() + 4, 0);
        // Entry point.
        <_ as Write<u64>>::write::<N, _>(
            &Some(Address(File::SIZE as u64 + Program::SIZE as u64 + file_load_va)),
            &mut buffer,
        )?;
        // Program headers offset.
        <_ as Write<u64>>::write::<N, _>(
            &Address(
                // Right after the file header.
                File::SIZE.into(),
            ),
            &mut buffer,
        )?;
        // Section headers offset.
        <_ as Write<u64>>::write::<N, _>(&Address(0), &mut buffer)?;
        // Processor flags.
        buffer.write_all(&N::write_u32(self.processor_flags))?;
        // File header size.
        buffer.write_all(&N::write_u16(File::SIZE))?;
        // Program header size.
        buffer.write_all(&N::write_u16(Program::SIZE))?;
        // Number of program headers.
        buffer.write_all(&N::write_u16(1))?;
        // Section header size.
        buffer.write_all(&N::write_u16(Section::SIZE))?;
        // Number of section headers.
        buffer.write_all(&N::write_u16(0))?;
        // Section index of the section names.
        <_ as Write<u16>>::write::<N, _>(&SectionIndex::Undefined, &mut buffer)?;

        let number_of_segments = self.segments.len();

        let program_headers = self.segments.into_iter().map(|segment| {
            let segment_size = File::SIZE as u64 + Program::SIZE as u64 + segment.data.len() as u64;

            Program {
                r#type: ProgramType::Load,
                segment_flags: segment.flags,
                offset: Address(0),
                virtual_address: Address(file_load_va),
                physical_address: Some(Address(file_load_va)),
                segment_size_in_file_image: segment_size,
                segment_size_in_memory: segment_size,
                alignment: Alignment::new(0x1000).unwrap(),
                data: Data::new(
                    Cow::Owned(segment.data),
                    DataType::ProgramData,
                    self.endianness.into(),
                    None,
                ),
            }
        });

        let mut segments = Vec::with_capacity(number_of_segments);

        for program_header in program_headers {
            program_header.write::<N, _>(&mut buffer)?;

            segments.push(program_header.data);
        }

        for segment in segments {
            buffer.write_all(&segment);
        }

        Ok(buffer)
    }
}

#[derive(Debug)]
struct Segment {
    flags: ProgramFlags,
    data: Vec<u8>,
}
