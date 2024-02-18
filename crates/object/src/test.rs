macro_rules! assert_read_write {
    (
        $subject:ty : Read< $reader_read_from:ty > + Write< $writer_read_from:ty > {
            bytes_value(auto_endian) = $bytes_value:expr,
            rust_value = $rust_value:expr,
            $(,)*
        }
    ) => {
        // Big endian.
        {
            let rust_value = $rust_value;
            let bytes_value = $bytes_value.to_be_bytes();

            let mut written_value = Vec::new();

            <_ as Write<$writer_read_from>>::write::<crate::BigEndian, _>(
                &rust_value,
                &mut written_value,
            )
            .unwrap();

            assert_eq!(written_value, bytes_value, "write as big endian");

            let read_value =
                <$subject as Read<$reader_read_from>>::read::<crate::BigEndian, ()>(&bytes_value);

            assert_eq!(read_value, Ok((&[] as &[u8], rust_value)), "read as big endian");
        }

        // Little endian.
        {
            let rust_value = $rust_value;
            let bytes_value = $bytes_value.to_le_bytes();

            let mut written_value = Vec::new();

            <_ as Write<$writer_read_from>>::write::<crate::LittleEndian, _>(
                &rust_value,
                &mut written_value,
            )
            .unwrap();

            assert_eq!(written_value, bytes_value, "write as little endian");

            let read_value = <$subject as Read<$reader_read_from>>::read::<crate::LittleEndian, ()>(
                &bytes_value,
            );

            assert_eq!(read_value, Ok((&[] as &[u8], rust_value)), "read as little endian");
        }
    };

    (
        $subject:ty : Read< $reader_read_from:ty > + Write< $writer_read_from:ty > {
            bytes_value(big_endian) = $bytes_value:expr,
            rust_value = $rust_value:expr,
            $(,)*
        }
    ) => {
        // Big endian.
        {
            let rust_value = $rust_value;
            let bytes_value = $bytes_value;

            let mut written_value = Vec::new();

            <_ as Write<$writer_read_from>>::write::<crate::BigEndian, _>(
                &rust_value,
                &mut written_value,
            )
            .unwrap();

            assert_eq!(written_value, bytes_value, "write as big endian");

            let read_value =
                <$subject as Read<$reader_read_from>>::read::<crate::BigEndian, ()>(&bytes_value);

            assert_eq!(read_value, Ok((&[] as &[u8], rust_value)), "read as big endian");
        }
    };
}
