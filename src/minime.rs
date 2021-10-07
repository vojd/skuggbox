use std::path::PathBuf;
use std::process::Command;

use log::error;
use log::warn;
use which::which;

pub struct Minime {
    preprocessor: PathBuf,
}

impl Minime {
    pub fn new(preprocessor: PathBuf) -> Minime {
        Minime { preprocessor }
    }

    pub fn preprocess(&self, source: PathBuf, camera_integration: bool) -> Option<String> {
        let mut cmd = Command::new(&self.preprocessor);

        if camera_integration {
            cmd.arg("-D");
            cmd.arg("USE_SKUGGBOX_CAMERA");
        }

        let output = cmd
            .arg("-stdout")
            .arg(source)
            .output()
            .expect("Failed to invoke minime-preprocessor");

        if !output.status.success() {
            error!(
                "ERROR: Minime failed with error: {}",
                String::from_utf8(output.stderr).unwrap()
            );
            return None;
        }

        Some(String::from_utf8(output.stdout).unwrap())
    }
}

pub fn find_minime_tool() -> Option<Minime> {
    match which("minime-preprocess") {
        Ok(path) => Some(Minime::new(path)),
        Err(e) => {
            warn!("WARNING: Can't find the minime toolchain: {}", e);
            None
        }
    }
}
