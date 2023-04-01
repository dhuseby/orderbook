use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use std::{env, fs, io};

fn mkdir(pb: &Utf8PathBuf) -> io::Result<()> {
    if !pb.as_path().is_dir() {
        return fs::create_dir_all(&pb);
    }

    Ok(())
}

fn build_uniffi(s: &str) -> Result<()> {
    // get the output dir
    let out_dir = env::var("OUT_DIR").context("$OUT_DIR missing?!")?;

    // get the manifest dir
    let manifest_dir: Utf8PathBuf = env::var("CARGO_MANIFEST_DIR")
        .context("$CARGO_MANIFEST_DIR missing?!")?
        .into();

    // get the path to the udl file
    let mut udl_dir = manifest_dir.clone();
    udl_dir.push("src");
    let mut udl_file = udl_dir.clone();
    udl_file.push(s);

    println!("cargo:rerun-if-changed={udl_file}");
    println!("cargo:rerun-if-env-changed=UNIFFI_TESTS_DISABLE_EXTENSIONS");

    // generates the orderbook.uniffi.rs file that gets included into the src/lib.rs file. the rust
    // code handles all of the lowering and lifting to/from rust.
    uniffi::generate_component_scaffolding(&udl_file, None, Some(udl_dir.as_ref()), false)?;

    // generate the bindings
    uniffi::generate_bindings(
        &udl_file,
        None,
        vec!["swift"],
        Some(out_dir.as_ref()),
        None,
        false,
    )?;

    // set up the dir structure
    let mut target = manifest_dir.clone();
    target.push("..");
    target.push("target");

    // create the target dir
    mkdir(&target)?;

    // generate the bindings in the "target" folder
    uniffi::generate_bindings(
        &udl_file,
        None,
        vec!["swift"],
        Some(target.as_ref()),
        None,
        false,
    )?;

    Ok(())
}

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
        .build_client(true)
        .build_server(false)
        .file_descriptor_set_path(&fd_file)
        .out_dir(&src_dir)
        .compile(&[proto_file], &[proto_dir])?;

    Ok(())
}

fn main() -> Result<()> {
    build_protos("orderbook.proto", "orderbook_descriptor.bin")?;
    build_uniffi("OrderbookClient.udl")?;
    Ok(())
}
