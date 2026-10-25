use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
struct Device {
    hostname: String,
    id: String,
    state: String,
}

fn random_uuid() -> io::Result<String> {
    let uuid = fs::read_to_string("/proc/sys/kernel/random/uuid")?;
    let uuid = uuid.trim();

    if !valid_uuid(uuid) {
        return Err(io::Error::other("Unable to generate a device identifier"));
    }

    Ok(uuid.to_owned())
}

fn valid_uuid(value: &str) -> bool {
    if value.len() != 36 {
        return false;
    }

    value.bytes().enumerate().all(|(index, byte)| {
        if matches!(index, 8 | 13 | 18 | 23) {
            byte == b'-'
        } else {
            byte.is_ascii_hexdigit()
        }
    })
}

fn generate_hostname() -> io::Result<String> {
    let uuid = random_uuid()?;
    let suffix: String = uuid
        .chars()
        .filter(|character| character.is_ascii_hexdigit())
        .take(10)
        .collect();

    Ok(format!("pass-{suffix}"))
}

// Reject known symbolic links under the offline root. This is not yet
// race-free against concurrent modifications by an untrusted process.
fn reject_symlinks(root: &Path, relative: &str) -> io::Result<()> {
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!("Symbolic link in initialization path: {}", current.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn read_existing(path: &Path) -> io::Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("Expected regular file: {}", path.display()),
            ));
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
    let contents = fs::read_to_string(path)?;
    let value = contents.trim();
    if value.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Existing file is empty: {}", path.display()),
        ));
    }
    Ok(Some(value.to_owned()))
}

// Publish a fully written file with hard_link rather than exposing a partial
// destination. An interrupted run may leave an unused .tmp file behind.
fn write_new(path: &Path, value: &str, mode: u32) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::other("Missing parent directory"))?;
    let name = path.file_name().ok_or_else(|| io::Error::other("Missing filename"))?;
    let temp = parent.join(format!(
        ".{}.{}.tmp",
        name.to_string_lossy(),
        random_uuid()?
    ));

    let result: io::Result<()> = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temp)?;
        file.set_permissions(fs::Permissions::from_mode(mode))?;
        file.write_all(format!("{value}\n").as_bytes())?;
        file.sync_all()?;
        fs::hard_link(&temp, path)?;
        fs::File::open(parent)?.sync_all()?;
        Ok(())
    })();

    // An abandoned temp file is harmless and can be removed after a crash.
    match fs::remove_file(&temp) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) if result.is_ok() => return Err(error),
        Err(_) => {}
    }
    result
}

fn initialize(root: &Path) -> io::Result<Device> {
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "The target root cannot be a symbolic link",
        ));
    }
    let root = fs::canonicalize(root)?;
    if root == Path::new("/") {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Live system initialization is not enabled",
        ));
    }

    for path in [
        "etc/hostname",
        "var/lib/passos/device-id",
        "var/lib/passos/provisioning-state",
    ] {
        reject_symlinks(&root, path)?;
    }

    let hostname_file = root.join("etc/hostname");
    let id_file = root.join("var/lib/passos/device-id");
    let state_file = root.join("var/lib/passos/provisioning-state");

    // Validate all existing values before making any changes.
    let hostname = read_existing(&hostname_file)?;
    let id = read_existing(&id_file)?;
    let state = read_existing(&state_file)?;

    if let Some(ref existing) = id
        && !valid_uuid(existing)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Existing device identifier is invalid",
        ));
    }
    if let Some(ref existing) = state
        && existing != "unclaimed"
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unsupported provisioning state",
        ));
    }

    let hostname = match hostname {
        Some(existing) => existing,
        None => {
            fs::create_dir_all(root.join("etc"))?;
            reject_symlinks(&root, "etc/hostname")?;
            let generated = generate_hostname()?;
            write_new(&hostname_file, &generated, 0o644)?;
            generated
        }
    };

    let id = match id {
        Some(existing) => existing,
        None => {
            fs::create_dir_all(root.join("var/lib/passos"))?;
            reject_symlinks(&root, "var/lib/passos/device-id")?;
            let generated = random_uuid()?;
            write_new(&id_file, &generated, 0o600)?;
            generated
        }
    };

    let state = match state {
        Some(existing) => existing,
        None => {
            reject_symlinks(&root, "var/lib/passos/provisioning-state")?;
            write_new(&state_file, "unclaimed", 0o600)?;
            "unclaimed".to_owned()
        }
    };

    Ok(Device { hostname, id, state })
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

    let device = initialize(&PathBuf::from(root))?;
    println!("Device hostname: {}", device.hostname);
    println!("Device identifier: {}", device.id);
    println!("Provisioning state: {}", device.state);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_root() -> PathBuf {
        let name = random_uuid().expect("random UUID");
        let root = env::temp_dir().join(format!("passos-init-test-{name}"));
        fs::create_dir(&root).expect("create test directory");
        root
    }

    fn cleanup(root: PathBuf) {
        fs::remove_dir_all(root).expect("clean test directory");
    }

    #[test]
    fn creates_identity_and_hostname() {
        let root = test_root();
        let device = initialize(&root).expect("initialize");

        assert!(device.hostname.starts_with("pass-"));
        assert!(valid_uuid(&device.id));
        assert_eq!(device.state, "unclaimed");
        assert_eq!(
            fs::read_to_string(root.join("etc/hostname"))
                .expect("hostname")
                .trim(),
            device.hostname
        );
        assert_eq!(
            fs::read_to_string(root.join("var/lib/passos/device-id"))
                .expect("device id")
                .trim(),
            device.id
        );
        assert_eq!(
            fs::read_to_string(root.join("var/lib/passos/provisioning-state"))
                .expect("provisioning state")
                .trim(),
            "unclaimed"
        );
        cleanup(root);
    }

    #[test]
    fn preserves_identity_on_repeat() {
        let root = test_root();
        let first = initialize(&root).expect("first initialization");
        let second = initialize(&root).expect("second initialization");
        assert_eq!(first, second);
        cleanup(root);
    }

    #[test]
    fn preserves_existing_hostname() {
        let root = test_root();
        fs::create_dir_all(root.join("etc")).unwrap();
        fs::write(root.join("etc/hostname"), "custom-host\n").unwrap();

        let device = initialize(&root).expect("initialize");
        assert_eq!(device.hostname, "custom-host");
        assert!(valid_uuid(&device.id));
        cleanup(root);
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
        assert!(!root.join("var/lib/passos/device-id").exists());
        cleanup(root);
    }

    #[test]
    fn rejects_invalid_existing_device_id() {
        let root = test_root();
        fs::create_dir_all(root.join("var/lib/passos")).unwrap();
        fs::write(root.join("var/lib/passos/device-id"), "not-a-uuid\n").unwrap();
        let error = initialize(&root).expect_err("invalid identity must be rejected");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(!root.join("etc/hostname").exists());
        cleanup(root);
    }

    #[test]
    fn validates_uuid_shape() {
        assert!(valid_uuid("4b879a42-536a-4d36-a815-4dd87c21fd01"));
        assert!(!valid_uuid(""));
        assert!(!valid_uuid("not-a-uuid"));
        assert!(!valid_uuid("4b879a42_536a-4d36-a815-4dd87c21fd01"));
    }
    #[test]
    fn recovers_from_interrupted_initialization() {
        let root = test_root();
        fs::create_dir_all(root.join("etc")).unwrap();
        fs::write(root.join("etc/hostname"), "kept-name\n").unwrap();
        let device = initialize(&root).expect("resume interrupted initialization");
        assert_eq!(device.hostname, "kept-name");
        assert!(valid_uuid(&device.id));
        assert_eq!(device.state, "unclaimed");
        cleanup(root);
    }

    #[test]
    fn rejects_invalid_provisioning_state() {
        let root = test_root();
        fs::create_dir_all(root.join("var/lib/passos")).unwrap();
        fs::write(root.join("var/lib/passos/provisioning-state"), "ready\n").unwrap();
        let error = initialize(&root).expect_err("unsupported state");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(!root.join("etc/hostname").exists());
        cleanup(root);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_state_directory() {
        use std::os::unix::fs::symlink;
        let root = test_root();
        let external = test_root();
        fs::create_dir_all(root.join("var/lib")).unwrap();
        symlink(&external, root.join("var/lib/passos")).unwrap();
        let error = initialize(&root).expect_err("must reject symlink");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(!external.join("device-id").exists());
        cleanup(root);
        cleanup(external);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_identity_file() {
        use std::os::unix::fs::symlink;
        let root = test_root();
        fs::create_dir_all(root.join("var/lib/passos")).unwrap();
        symlink("/etc/hostname", root.join("var/lib/passos/device-id")).unwrap();
        let error = initialize(&root).expect_err("must reject symlink");
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        cleanup(root);
    }
}
