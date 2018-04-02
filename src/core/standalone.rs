use std::io;

use brotli_decompressor;

pub fn file_exists(name: &str) -> bool {
    FILES.iter().any(|f| f.0 == name)
}

pub fn file(name: &str) -> Option<Vec<u8>> {
    brotli_compressed_file(name).map(|compressed| {
        let mut result = Vec::new();
        brotli_decompressor::BrotliDecompress(&mut io::Cursor::new(compressed), &mut result)
            .unwrap();
        result
    })
}

pub fn brotli_compressed_file(name: &str) -> Option<&'static [u8]> {
    FILES.iter().find(|f| f.0 == name).map(|f| f.1)
}

include!(concat!(env!("OUT_DIR"), "/core.standalone.rs"));
