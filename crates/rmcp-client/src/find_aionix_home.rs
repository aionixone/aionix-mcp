use dirs::home_dir;
use std::path::PathBuf;

#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
fn override_storage() -> &'static Mutex<Option<PathBuf>> {
    static OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

/// This was copied from aionix-core but aionix-core depends on this crate.
/// TODO: move this to a shared crate lower in the dependency tree.
///
///
/// Returns the path to the Aionix configuration directory, which can be
/// specified by the `AIONIX_HOME` environment variable. If not set, defaults to
/// `~/.aionix`.
///
/// - If `AIONIX_HOME` is set, the value will be canonicalized and this
///   function will Err if the path does not exist.
/// - If `AIONIX_HOME` is not set, this function does not verify that the
///   directory exists.
pub(crate) fn find_aionix_home() -> std::io::Result<PathBuf> {
    #[cfg(test)]
    if let Some(path) = override_storage().lock().unwrap().clone() {
        return Ok(path);
    }

    // Honor the `AIONIX_HOME` environment variable when it is set to allow users
    // (and tests) to override the default location.
    if let Ok(val) = std::env::var("AIONIX_HOME")
        && !val.is_empty()
    {
        return PathBuf::from(val).canonicalize();
    }

    let mut p = home_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        )
    })?;
    p.push(".aionix");
    Ok(p)
}

#[cfg(test)]
pub(crate) struct AionixHomeOverrideGuard {
    previous: Option<PathBuf>,
}

#[cfg(test)]
pub(crate) fn set_aionix_home_override(path: PathBuf) -> AionixHomeOverrideGuard {
    let storage = override_storage();
    let mut guard = storage.lock().unwrap();
    let previous = guard.replace(path);
    drop(guard);
    AionixHomeOverrideGuard { previous }
}

#[cfg(test)]
impl Drop for AionixHomeOverrideGuard {
    fn drop(&mut self) {
        let storage = override_storage();
        let mut guard = storage.lock().unwrap();
        *guard = self.previous.clone();
    }
}
