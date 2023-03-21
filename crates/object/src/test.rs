macro_rules! assert_read {
    ($type:ident::$reader:ident($input:literal $( ( as $input_type:ty ) )? ) <=> $built_type:expr ) => {
        // Read as big endian.
        {
            let input = ( $input $( as $input_type )? ).to_be_bytes();
            let value = $type::$reader::<crate::BigEndian, ()>(&input);

            assert_eq!(value, Ok((&[] as &[u8], $built_type)));
        }

        // Read as little endian.
        {
            let input = ( $input $( as $input_type )? ).to_le_bytes();
            let value = $type::$reader::<crate::LittleEndian, ()>(&input);

            assert_eq!(value, Ok((&[] as &[u8], $built_type)));
        }
    };
}

macro_rules! assert_read_write {
    ($type:ident::$reader:ident($input:literal $( ( as $input_type:ty ) )? $( ~ $real_input:literal $( ( as $real_input_type:ty ) )? )? ) <=> $built_type:expr ) => {
        // Read as big endian.
        {
            let input = ( $input $( as $input_type )? ).to_be_bytes();
            let value = $type::$reader::<crate::BigEndian, ()>(&input);

            assert_eq!(value, Ok((&[] as &[u8], $built_type)), "read as big endian");
        }

        // Read as little endian.
        {
            let input = ( $input $( as $input_type )? ).to_le_bytes();
            let value = $type::$reader::<crate::LittleEndian, ()>(&input);

            assert_eq!(value, Ok((&[] as &[u8], $built_type)), "read as little endian");
        }

        // Write as big endian.
        {
            let mut buffer = Vec::new();

            $built_type.write::<crate::BigEndian, _>(&mut buffer).unwrap();

            #[allow(unused)]
            let input = ( $input $( as $input_type )? ).to_be_bytes();
            $(
                let input = ( $real_input $( as $real_input_type )? ).to_be_bytes();
            )?

            assert_eq!(buffer, input, "write as big endian");
        }

        // Write as little endian.
        {
            let mut buffer = Vec::new();

            $built_type.write::<crate::LittleEndian, _>(&mut buffer).unwrap();

            #[allow(unused)]
            let input = ( $input $( as $input_type )? ).to_le_bytes();
            $(
                let input = ( $real_input $( as $real_input_type )? ).to_le_bytes();
            )?

            assert_eq!(buffer, input, "write as little endian");
        }
    };
}
