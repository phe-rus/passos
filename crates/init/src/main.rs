use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
struct Device {
    hostname: String,
    id: String,
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

fn read_existing(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(contents) => {
            let value = contents.trim();
            if value.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Existing file is empty: {}", path.display()),
                ));
            }
            Ok(Some(value.to_owned()))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn write_new(path: &Path, value: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(format!("{value}\n").as_bytes())?;
    file.sync_all()
}

fn initialize(root: &Path) -> io::Result<Device> {
    let root = fs::canonicalize(root)?;
    if root == Path::new("/") {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Live system initialization is not enabled",
        ));
    }

    let hostname_file = root.join("etc/hostname");
    let id_file = root.join("var/lib/passos/device-id");

    // Reject invalid pre-existing state before creating another file.
    let hostname = read_existing(&hostname_file)?;
    let id = read_existing(&id_file)?;
    if let Some(ref existing) = id
        && !valid_uuid(existing)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Existing device identifier is invalid",
        ));
    }

    let hostname = match hostname {
        Some(existing) => existing,
        None => {
            fs::create_dir_all(root.join("etc"))?;
            let generated = generate_hostname()?;
            write_new(&hostname_file, &generated)?;
            generated
        }
    };

    let id = match id {
        Some(existing) => existing,
        None => {
            fs::create_dir_all(root.join("var/lib/passos"))?;
            let generated = random_uuid()?;
            write_new(&id_file, &generated)?;
            generated
        }
    };

    Ok(Device { hostname, id })
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
}
