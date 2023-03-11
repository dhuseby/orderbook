use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use std::env;

fn build_protos(proto: &str, fdf: &str) -> Result<()> {
    // get the manifest dir
    let manifest_dir: Utf8PathBuf = env::var("CARGO_MANIFEST_DIR")
        .context("$CARGO_MANIFEST_DIR missing?!")?
        .into();

    // path to the proto dir
    let mut proto_dir = manifest_dir.clone();
    proto_dir.push("..");
    proto_dir.push("proto");

    // path to the proto file
    let mut proto_file = proto_dir.clone();
    proto_file.push(proto);

    // path to the src dir
    let mut src_dir = manifest_dir.clone();
    src_dir.push("src");

    // get the output dir
    let out_dir: Utf8PathBuf = env::var("OUT_DIR").context("$OUT_DIR missing?!")?.into();

    // path to the file descriptor file
    let mut fd_file = out_dir.clone();
    fd_file.push(fdf);

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_client(false)
        .build_server(true)
        .file_descriptor_set_path(&fd_file)
        .out_dir(&src_dir)
        .compile(&[proto_file], &[proto_dir])?;

    Ok(())
}

fn main() -> Result<()> {
    build_protos("orderbook.proto", "orderbook_descriptor.bin")?;
    Ok(())
}
