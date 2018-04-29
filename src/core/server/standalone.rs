use std::io;

use brotli_decompressor;
use bytes;
use futures;
use hyper;

pub struct File {
    pub name: &'static str,
    pub contents: FileStream,
    pub size: u64,
}

pub enum FileStream {
    Static(Option<bytes::Bytes>),
    Brotli(brotli_decompressor::Decompressor<io::Cursor<&'static [u8]>>),
}

pub fn file_exists(name: &str) -> bool {
    FILES.iter().any(|f| f.0 == name)
}

pub fn file(name: &str) -> Option<File> {
    FILES.iter().find(|f| f.0 == name).map(|f| {
        let decompressor = brotli_decompressor::Decompressor::new(io::Cursor::new(f.1), 4096);
        File::new_brotli(f.0, decompressor, f.2)
    })
}

pub fn brotli_compressed_file(name: &str) -> Option<File> {
    FILES
        .iter()
        .find(|f| f.0 == name)
        .map(|f| File::new_static(f.0, f.1, f.2))
}

impl File {
    fn new_static(name: &'static str, contents: &'static [u8], size: u64) -> Self {
        let contents = FileStream::Static(Some(bytes::Bytes::from_static(contents)));
        File {
            name,
            contents,
            size,
        }
    }

    fn new_brotli(
        name: &'static str,
        decompressor: brotli_decompressor::Decompressor<io::Cursor<&'static [u8]>>,
        size: u64,
    ) -> Self {
        let contents = FileStream::Brotli(decompressor);
        File {
            name,
            contents,
            size,
        }
    }
}

impl futures::stream::Stream for FileStream {
    type Item = bytes::Bytes;
    type Error = hyper::Error;

    fn poll(&mut self) -> futures::Poll<Option<Self::Item>, Self::Error> {
        match *self {
            FileStream::Static(ref mut stream) => Ok(futures::Async::Ready(stream.take())),
            FileStream::Brotli(ref mut decompressor) => {
                use std::io::Read;

                let mut bytes = bytes::BytesMut::with_capacity(4096);
                match decompressor.read(&mut bytes) {
                    Err(e) => Err(hyper::Error::Io(e)),
                    Ok(len) => if len == 0 {
                        Ok(futures::Async::Ready(None))
                    } else {
                        bytes.truncate(len);
                        Ok(futures::Async::Ready(Some(bytes.freeze())))
                    },
                }
            }
        }
    }
}

#[cfg(feature = "standalone")]
include!(concat!(env!("OUT_DIR"), "/core.standalone.rs"));

#[cfg(not(feature = "standalone"))]
const FILES: &'static [(&'static str, &'static [u8], u64)] = &[];
