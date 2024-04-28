use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  tonic_build::configure()
    .file_descriptor_set_path(out_dir.join("raft_descriptor.bin"))
    .compile(&["proto/raft.proto"], &["proto"])?;
  tonic_build::compile_protos("proto/raft.proto")?;

  tonic_build::configure()
    .file_descriptor_set_path(out_dir.join("raft_api_descriptor.bin"))
    .compile(&["proto/api.proto"], &["proto"])?;
  tonic_build::compile_protos("proto/api.proto")?;

  Ok(())
}
