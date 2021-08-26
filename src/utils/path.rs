//! # Path
//!
//! Path related utilities

use std::path::{Path, PathBuf};

/// ### absolutize
///
/// Absolutize target path if relative.
/// For example:
///
/// ```rust
/// assert_eq!(absolutize(&Path::new("/home/omar"), &Path::new("readme.txt")).as_path(), Path::new("/home/omar/readme.txt"));
/// assert_eq!(absolutize(&Path::new("/home/omar"), &Path::new("/tmp/readme.txt")).as_path(), Path::new("/tmp/readme.txt"));
/// ```
pub fn absolutize(wrkdir: &Path, target: &Path) -> PathBuf {
    match target.is_absolute() {
        true => target.to_path_buf(),
        false => {
            let mut p: PathBuf = wrkdir.to_path_buf();
            p.push(target);
            p
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn absolutize_path() {
        assert_eq!(
            absolutize(&Path::new("/home/omar"), &Path::new("readme.txt")).as_path(),
            Path::new("/home/omar/readme.txt")
        );
        assert_eq!(
            absolutize(&Path::new("/home/omar"), &Path::new("/tmp/readme.txt")).as_path(),
            Path::new("/tmp/readme.txt")
        );
    }
}
