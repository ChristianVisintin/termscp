//! ## TestHelpers
//!
//! contains helper functions for tests

/**
 * MIT License
 *
 * termscp - Copyright (c) 2021 Christian Visintin
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use crate::fs::{FsDirectory, FsEntry, FsFile, UnixPex};
// ext
use std::fs::File;
#[cfg(feature = "with-containers")]
use std::fs::OpenOptions;
#[cfg(feature = "with-containers")]
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tempfile::NamedTempFile;

pub fn create_sample_file_entry() -> (FsFile, NamedTempFile) {
    // Write
    let tmpfile = create_sample_file();
    (
        FsFile {
            name: tmpfile
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            abs_path: tmpfile.path().to_path_buf(),
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            size: 127,
            ftype: None,    // File type
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        },
        tmpfile,
    )
}

pub fn create_sample_file() -> NamedTempFile {
    // Write
    let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        tmpfile,
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.Mauris ultricies consequat eros,nec scelerisque magna imperdiet metus."
    )
    .unwrap();
    tmpfile
}

/// ### make_file_at
///
/// Make a file with `name` at specified path
pub fn make_file_at(dir: &Path, filename: &str) -> std::io::Result<()> {
    let mut p: PathBuf = PathBuf::from(dir);
    p.push(filename);
    let mut file: File = File::create(p.as_path())?;
    writeln!(
        file,
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.Mauris ultricies consequat eros,nec scelerisque magna imperdiet metus."
    )?;
    Ok(())
}

/// ### make_dir_at
///
/// Make a directory in `dir`
pub fn make_dir_at(dir: &Path, dirname: &str) -> std::io::Result<()> {
    let mut p: PathBuf = PathBuf::from(dir);
    p.push(dirname);
    std::fs::create_dir(p.as_path())
}

#[cfg(feature = "with-containers")]
pub fn write_file(file: &NamedTempFile, writable: &mut Box<dyn Write>) {
    let mut fhnd = OpenOptions::new()
        .create(false)
        .read(true)
        .write(false)
        .open(file.path())
        .ok()
        .unwrap();
    // Read file
    let mut buffer: [u8; 65536] = [0; 65536];
    assert!(fhnd.read(&mut buffer).is_ok());
    // Write file
    assert!(writable.write(&buffer).is_ok());
}

#[cfg(feature = "with-containers")]
pub fn write_ssh_key() -> NamedTempFile {
    let mut tmpfile: NamedTempFile = NamedTempFile::new().unwrap();
    writeln!(
        tmpfile,
        r"-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAABFwAAAAdzc2gtcn
NhAAAAAwEAAQAAAQEAxKyYUMRCNPlb4ZV1VMofrzApu2l3wgP4Ot9wBvHsw/+RMpcHIbQK
9iQqAVp8Z+M1fJyPXTKjoJtIzuCLF6Sjo0KI7/tFTh+yPnA5QYNLZOIRZb8skumL4gwHww
5Z942FDPuUDQ30C2mZR9lr3Cd5pA8S1ZSPTAV9QQHkpgoS8cAL8QC6dp3CJjUC8wzvXh3I
oN3bTKxCpM10KMEVuWO3lM4Nvr71auB9gzo1sFJ3bwebCZIRH01FROyA/GXRiaOtJFG/9N
nWWI/iG5AJzArKpLZNHIP+FxV/NoRH0WBXm9Wq5MrBYrD1NQzm+kInpS/2sXk3m1aZWqLm
HF2NKRXSbQAAA8iI+KSniPikpwAAAAdzc2gtcnNhAAABAQDErJhQxEI0+VvhlXVUyh+vMC
m7aXfCA/g633AG8ezD/5EylwchtAr2JCoBWnxn4zV8nI9dMqOgm0jO4IsXpKOjQojv+0VO
H7I+cDlBg0tk4hFlvyyS6YviDAfDDln3jYUM+5QNDfQLaZlH2WvcJ3mkDxLVlI9MBX1BAe
SmChLxwAvxALp2ncImNQLzDO9eHcig3dtMrEKkzXQowRW5Y7eUzg2+vvVq4H2DOjWwUndv
B5sJkhEfTUVE7ID8ZdGJo60kUb/02dZYj+IbkAnMCsqktk0cg/4XFX82hEfRYFeb1arkys
FisPU1DOb6QielL/axeTebVplaouYcXY0pFdJtAAAAAwEAAQAAAP8u3PFuTVV5SfGazwIm
MgNaux82iOsAT/HWFWecQAkqqrruUw5f+YajH/riV61NE9aq2qNOkcJrgpTWtqpt980GGd
SHWlgpRWQzfIooEiDk6Pk8RVFZsEykkDlJQSIu2onZjhi5A5ojHgZoGGabDsztSqoyOjPq
6WPvGYRiDAR3leBMyp1WufBCJqAsC4L8CjPJSmnZhc5a0zXkC9Syz74Fa08tdM7bGhtvP1
GmzuYxkgxHH2IFeoumUSBHRiTZayGuRUDel6jgEiUMxenaDKXe7FpYzMm9tQZA10Mm4LhK
5rP9nd2/KRTFRnfZMnKvtIRC9vtlSLBe14qw+4ZCl60AAACAf1kghlO3+HIWplOmk/lCL0
w75Zz+RdvueL9UuoyNN1QrUEY420LsixgWSeRPby+Rb/hW+XSAZJQHowQ8acFJhU85So7f
4O4wcDuE4f6hpsW9tTfkCEUdLCQJ7EKLCrod6jIV7hvI6rvXiVucRpeAzdOaq4uzj2cwDd
tOdYVsnmQAAACBAOVxBsvO/Sr3rZUbNtA6KewZh/09HNGoKNaCeiD7vaSn2UJbbPRByF/o
Oo5zv8ee8r3882NnmG808XfSn7pPZAzbbTmOaJt0fmyZhivCghSNzV6njW3o0PdnC0fGZQ
ruVXgkd7RJFbsIiD4dDcF4VCjwWHfTK21EOgJUA5pN6TNvAAAAgQDbcJWRx8Uyhkj2+srb
3n2Rt6CR7kEl9cw17ItFjMn+pO81/5U2aGw0iLlX7E06TAMQC+dyW/WaxQRey8RRdtbJ1e
TNKCN34QCWkyuYRHGhcNc0quEDayPw5QWGXlP4BzjfRUcPxY9cCXLe5wDLYsX33HwOAc59
RorU9FCmS/654wAAABFyb290QDhjNTBmZDRjMzQ1YQECAw==
-----END OPENSSH PRIVATE KEY-----"
    )
    .unwrap();
    tmpfile
}

/// ### make_fsentry
///
/// Create a FsEntry at specified path
pub fn make_fsentry(path: PathBuf, is_dir: bool) -> FsEntry {
    match is_dir {
        true => FsEntry::Directory(FsDirectory {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            abs_path: path,
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        }),
        false => FsEntry::File(FsFile {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            abs_path: path,
            last_change_time: SystemTime::UNIX_EPOCH,
            last_access_time: SystemTime::UNIX_EPOCH,
            creation_time: SystemTime::UNIX_EPOCH,
            size: 127,
            ftype: None,    // File type
            symlink: None,  // UNIX only
            user: Some(0),  // UNIX only
            group: Some(0), // UNIX only
            unix_pex: Some((UnixPex::from(6), UnixPex::from(4), UnixPex::from(4))), // UNIX only
        }),
    }
}

/// ### create_file_ioers
///
/// Open a file with two handlers, the first is to read, the second is to write
pub fn create_file_ioers(p: &Path) -> (File, File) {
    (File::open(p).ok().unwrap(), File::create(p).ok().unwrap())
}

mod test {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_utils_test_helpers_sample_file() {
        let (file, _) = create_sample_file_entry();
        assert!(file.symlink.is_none());
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_utils_test_helpers_write_file() {
        let (_, temp) = create_sample_file_entry();
        let tempdest = NamedTempFile::new().unwrap();
        let mut dest: Box<dyn Write> = Box::new(
            OpenOptions::new()
                .create(true)
                .read(false)
                .write(true)
                .open(tempdest.path())
                .ok()
                .unwrap(),
        );
        write_file(&temp, &mut dest);
    }

    #[test]
    #[cfg(feature = "with-containers")]
    fn test_utils_test_helpers_write_ssh_key() {
        let _ = write_ssh_key();
    }

    #[test]
    fn test_utils_test_helpers_make_fsentry() {
        assert_eq!(
            make_fsentry(PathBuf::from("/tmp/omar.txt"), false)
                .unwrap_file()
                .name
                .as_str(),
            "omar.txt"
        );
        assert_eq!(
            make_fsentry(PathBuf::from("/tmp/cards"), true)
                .unwrap_dir()
                .name
                .as_str(),
            "cards"
        );
    }

    #[test]
    fn test_utils_test_helpers_make_samples() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        assert!(make_file_at(tmpdir.path(), "omaroni.txt").is_ok());
        assert!(make_file_at(PathBuf::from("/aaaaa/bbbbb/cccc").as_path(), "readme.txt").is_err());
        assert!(make_dir_at(tmpdir.path(), "docs").is_ok());
        assert!(make_dir_at(PathBuf::from("/aaaaa/bbbbb/cccc").as_path(), "docs").is_err());
    }

    #[test]
    fn test_utils_test_helpers_create_file_ioers() {
        let (_, tmp) = create_sample_file_entry();
        let _ = create_file_ioers(tmp.path());
    }
}
