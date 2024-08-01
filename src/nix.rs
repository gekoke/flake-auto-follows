use color_eyre::eyre::{bail, Result};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tracing::{debug, warn};

use crate::metadata::Metadata;

pub struct FlakeRef(pub String);

pub struct JsonMetadata(pub String);

pub fn nix_flake_metadata(flake_ref: &FlakeRef) -> Result<JsonMetadata> {
    let cmd_output = Command::new("nix")
        .arg("flake")
        .arg("metadata")
        .arg(&flake_ref.0)
        .arg("--json")
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    if cmd_output.status.success() {
        Ok(JsonMetadata(String::from_utf8(cmd_output.stdout)?))
    } else {
        bail!("Nix failed with {}", cmd_output.status)
    }
}

pub fn resolve_flake_filepath(flake_ref: &FlakeRef) -> Result<PathBuf> {
    let metadata: Metadata = nix_flake_metadata(flake_ref)?.try_into()?;
    let url = metadata.resolved_url;
    if url.scheme() != "git+file" && url.scheme() != "file" {
        bail!(
            "no local `flake.nix` file can be resolved with given URL scheme '{}'",
            url.scheme()
        )
    }
    debug!("url has path: {}", url.path());

    let dir = url
        .query_pairs()
        .find(|(key, _val)| key == "dir")
        .map(|(_key, val)| val)
        .unwrap_or_default();
    debug!("url has `dir` parameter with value: {}", dir);

    let path = Path::new(url.path()).join(dir.as_ref()).join("flake.nix");
    match path.to_str() {
        Some(path) => debug!("resolved flake filepath {}", path),
        None => warn!("flake filepath was an invalid string"),
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use crate::nix::{nix_flake_metadata, FlakeRef};

    #[test]
    fn test_gets_metadata() {
        let flake_ref = FlakeRef("./test_data".into());

        assert!(nix_flake_metadata(&flake_ref).is_ok());
    }
}
