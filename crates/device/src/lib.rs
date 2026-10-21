use std::{fs, io, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayConnector {
    pub name: String,
    pub state: ConnectionState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeviceSnapshot {
    pub displays: Vec<DisplayConnector>,
}

pub fn discover() -> io::Result<DeviceSnapshot> {
    discover_from(Path::new("/sys"))
}

pub fn discover_from(sys_root: &Path) -> io::Result<DeviceSnapshot> {
    let drm_path = sys_root.join("class/drm");

    if !drm_path.exists() {
        return Ok(DeviceSnapshot::default());
    }

    let mut displays = Vec::new();

    for entry in fs::read_dir(drm_path)? {
        let entry = entry?;
        let path = entry.path();

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        // DRM connector directories expose a status file.
        let status_path = path.join("status");

        if !status_path.is_file() {
            continue;
        }

        let status = fs::read_to_string(status_path)?;

        let state = match status.trim() {
            "connected" => ConnectionState::Connected,
            "disconnected" => ConnectionState::Disconnected,
            _ => ConnectionState::Unknown,
        };

        displays.push(DisplayConnector {
            name: name.to_owned(),
            state,
        });
    }

    displays.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(DeviceSnapshot { displays })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_drm_directory_returns_empty_snapshot() {
        let snapshot = discover_from(Path::new("/nonexistent-pass-device-test"));

        assert!(snapshot.unwrap().displays.is_empty());
    }

    #[test]
    fn default_snapshot_is_empty() {
        let snapshot = DeviceSnapshot::default();

        assert!(snapshot.displays.is_empty());
    }
}
