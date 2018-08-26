extern crate brotli;
extern crate bytesize;
extern crate failure;
extern crate prost_build;
extern crate prost_simple_rpc_build;
extern crate vergen;
extern crate walkdir;

use std::env;
use std::ffi;
use std::fs;
use std::path;

fn main() {
    prost_build::Config::new()
        .service_generator(Box::new(prost_simple_rpc_build::ServiceGenerator::new()))
        .compile_protos(
            &[
                "src/schema/se/service/service.proto",
                "src/schema/se/slate/schema.proto",
                "src/schema/se/transport/transport.proto",
            ],
            &["src/schema"],
        ).unwrap();

    vergen::vergen(vergen::ConstantsFlags::all()).unwrap();

    if cfg!(feature = "standalone") {
        let build_dir =
            path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("build");

        let out_dir_str = env::var_os("OUT_DIR").unwrap();
        let out_dir = path::Path::new(&out_dir_str);

        create_bundle(&build_dir, out_dir).unwrap();
    }
}

fn create_bundle(build_dir: &path::Path, out_dir: &path::Path) -> Result<(), failure::Error> {
    use std::io::Write;

    let mut data_rs_file = fs::File::create(out_dir.join("core.standalone.rs"))?;

    writeln!(data_rs_file, "const FILES: &[(&str, &[u8], u64)] = &[")?;

    for entry in walkdir::WalkDir::new(build_dir) {
        let entry = entry?;
        let in_path = entry.path();
        let relative_path = in_path.strip_prefix(build_dir)?;
        let out_path = out_dir.join("standalone").join(relative_path);

        println!("cargo:rerun-if-changed={:?}", in_path);

        if entry.file_type().is_dir() {
            if !out_path.exists() {
                fs::create_dir(out_path)?;
            }
        } else if relative_path.extension() != Some(ffi::OsStr::new("map")) {
            let compressed_path = with_brotli_extension(&out_path);
            let in_metadata = in_path.metadata()?;
            let in_size = in_metadata.len();

            if !compressed_path.exists()
                || compressed_path.metadata()?.modified()? < in_metadata.modified()?
            {
                let mut in_file = fs::File::open(&in_path)?;
                let mut compressed_file = fs::File::create(&compressed_path)?;

                brotli::BrotliCompress(
                    &mut in_file,
                    &mut compressed_file,
                    &brotli::enc::BrotliEncoderParams {
                        quality: 11,
                        size_hint: in_size as usize,
                        ..brotli::enc::BrotliEncoderParams::default()
                    },
                )?;
            }

            let compressed_metadata = compressed_path.metadata()?;

            println!(
                ">> compressed {:?}: before: {}, after: {}",
                relative_path,
                bytesize::ByteSize::b(in_metadata.len() as u64),
                bytesize::ByteSize::b(compressed_metadata.len() as u64),
            );

            let size = format!("{}", in_size)
                .chars()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .enumerate()
                .rev()
                .flat_map(|(i, d)| {
                    if i > 0 && i % 3 == 0 {
                        vec![d, '_']
                    } else {
                        vec![d]
                    }
                }).collect::<String>();
            writeln!(data_rs_file,
                     r#"    ({relative_path:?}, include_bytes!(concat!(env!("OUT_DIR"), {data_path:?})), {size}),"#,
                     relative_path = relative_path,
                     data_path = path::Path::new("/standalone").join(with_brotli_extension(&relative_path)),
                     size = size)?
        }
    }
    writeln!(data_rs_file, "];")?;
    Ok(())
}

fn with_brotli_extension(path: &path::Path) -> path::PathBuf {
    path.with_extension(
        &path
            .extension()
            .map(|e| format!("{}.br", e.to_string_lossy()))
            .unwrap_or_else(|| "br".to_owned()),
    )
}
