macro_rules! assert_read {
    ( $type:ident :: $reader:ident ( $input:tt ) <=> $built_type:expr ) => {
        // Read as big endian.
        {
            let input = $input.to_be_bytes();
            let value = $type::$reader::<crate::BigEndian, ()>(&input);

            assert_eq!(value, Ok((&[] as &[u8], $built_type)));
        }

        // Read as little endian.
        {
            let input = $input.to_le_bytes();
            let value = $type::$reader::<crate::LittleEndian, ()>(&input);

            assert_eq!(value, Ok((&[] as &[u8], $built_type)));
        }
    };
}

macro_rules! assert_read_write {
    (
        $reader_type:ident :: $reader:ident (
            $( ;to_bytes; $to_bytes:expr )?
            $( ;bytes; $bytes:expr )?
            $( , $args:expr )*
            $(,)*
        )
        <=>
        (
            $built_type:expr
        )
        <=>
        Write< $writer_read_from:ty >
    ) => {
        // Big endian.
        {
            let rust_value = $built_type;

            $( let bytes = $bytes )?
            $( let bytes = $to_bytes.to_be_bytes(); )?

            let read_value = $reader_type::$reader::<crate::BigEndian, ()>(&bytes $( , $args )* );

            assert_eq!(read_value, Ok((&[] as &[u8], $built_type)), "read as big endian");

            let mut written_value = Vec::new();

            < _ as Write< $writer_read_from >>::write::<crate::BigEndian, _>(&rust_value, &mut written_value).unwrap();

            assert_eq!(written_value, bytes, "write as big endian");
        }

        // Little endian.
        {
            let rust_value = $built_type;

            $( let bytes = $bytes )?
            $( let bytes = $to_bytes.to_le_bytes(); )?

            let read_value = $reader_type::$reader::<crate::LittleEndian, ()>(&bytes $( , $args )* );

            assert_eq!(read_value, Ok((&[] as &[u8], $built_type)), "read as little endian");

            let mut written_value = Vec::new();

            < _ as Write< $writer_read_from >>::write::<crate::LittleEndian, _>(&rust_value, &mut written_value).unwrap();

            assert_eq!(written_value, bytes, "write as little endian");
        }
    };
}
