use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

fn generate_hostname() -> io::Result<String> {
    let uuid = fs::read_to_string("/proc/sys/kernel/random/uuid")?;

    let suffix: String = uuid
        .chars()
        .filter(|character| character.is_ascii_hexdigit())
        .take(10)
        .collect();

    if suffix.len() != 10 {
        return Err(io::Error::other("Unable to generate a device hostname"));
    }

    Ok(format!("pass-{suffix}"))
}

fn initialize(root: &Path) -> io::Result<String> {
    let root = fs::canonicalize(root)?;
    if root == Path::new("/") {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Live system initialization is not enabled",
        ));
    }
    let etc = root.join("etc");
    let hostname_file = etc.join("hostname");
    match fs::read_to_string(&hostname_file) {
        Ok(existing) => {
            let hostname = existing.trim();
            if hostname.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Existing hostname is empty",
                ));
            }
            return Ok(hostname.to_owned());
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    fs::create_dir_all(&etc)?;
    let hostname = generate_hostname()?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&hostname_file)?;

    file.write_all(format!("{hostname}\n").as_bytes())?;
    file.sync_all()?;

    Ok(hostname)
}

fn main() -> io::Result<()> {
    let mut arguments = env::args().skip(1);
    let Some(option) = arguments.next() else {
        eprintln!("Usage: init --root <directory>");
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing target directory",
        ));
    };
    if option != "--root" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Expected --root",
        ));
    }
    let Some(root) = arguments.next() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Missing root directory",
        ));
    };
    if arguments.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unexpected arguments",
        ));
    }
    let root = PathBuf::from(root);
    if root == Path::new("/") {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Initializing a live system is not enabled yet",
        ));
    }
    let hostname = initialize(&root)?;
    println!("Device hostname: {hostname}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_root() -> PathBuf {
        let name = generate_hostname().expect("random hostname");

        let root = env::temp_dir().join(format!("passos-init-test-{name}"));

        fs::create_dir(&root).expect("create test directory");

        root
    }

    #[test]
    fn creates_hostname() {
        let root = test_root();

        let hostname = initialize(&root).expect("initialize");

        assert!(hostname.starts_with("pass-"));

        let stored = fs::read_to_string(root.join("etc/hostname")).expect("read hostname");

        assert_eq!(stored.trim(), hostname);

        fs::remove_dir_all(root).expect("clean test directory");
    }

    #[test]
    fn preserves_existing_hostname() {
        let root = test_root();

        let first = initialize(&root).expect("first initialization");
        let second = initialize(&root).expect("second initialization");

        assert_eq!(first, second);

        fs::remove_dir_all(root).expect("clean test directory");
    }

    #[test]
    fn rejects_live_root() {
        let error = initialize(Path::new("/")).expect_err("live root must be rejected");

        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn rejects_empty_existing_hostname() {
        let root = test_root();

        fs::create_dir_all(root.join("etc")).unwrap();
        fs::write(root.join("etc/hostname"), "\n").unwrap();

        let error = initialize(&root).expect_err("empty hostname must be rejected");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);

        fs::remove_dir_all(root).expect("clean test directory");
    }
}
