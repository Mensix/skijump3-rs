pub mod config;
pub(crate) mod custom_cup;
pub mod players;
pub mod records;

use std::rc::Rc;

use crate::data::profile::ProfileStore;
use crate::data::records::RecordStore;

use self::config::Config;
use crate::files::FileStore;

pub type SaveRef = Rc<SaveManager>;

pub(crate) fn parse_toml<T>(data: &[u8]) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let text = std::str::from_utf8(data).map_err(|error| error.to_string())?;
    toml::from_str(text).map_err(|error| error.to_string())
}

pub(crate) fn load_or_default<T, Parse, Serialize, Default>(
    files: &FileStore,
    filename: &str,
    load_saved: bool,
    parse: Parse,
    default: Default,
    serialize: Serialize,
) -> Result<T, String>
where
    Parse: Fn(&[u8]) -> Result<T, String>,
    Default: FnOnce() -> Result<T, String>,
    Serialize: Fn(&T) -> Result<Vec<u8>, String>,
{
    let saved = if load_saved && files.exists_save(filename) {
        match parse(&files.read_save(filename)) {
            Ok(value) => Some(value),
            Err(error) => return Err(format!("{filename}: {error}")),
        }
    } else {
        None
    };
    if let Some(value) = saved {
        return Ok(value);
    }

    let value = default()?;
    if let Ok(data) = serialize(&value) {
        let _ = files.write(filename, &data);
    }
    Ok(value)
}

#[derive(Debug)]
pub struct SaveManager {
    pub files: Rc<FileStore>,
}

impl SaveManager {
    pub fn new(files: Rc<FileStore>) -> Self {
        Self { files }
    }

    pub fn save_config(&self, config: &Config) -> bool {
        let Ok(data) = config.to_toml_bytes() else {
            return false;
        };
        self.files.write("config.toml", &data)
    }

    fn save_bytes(&self, filename: &str, data: &[u8]) -> bool {
        self.files.write(filename, data)
    }

    pub fn save_players(&self, store: &ProfileStore) -> bool {
        let Ok(data) = store.to_toml_bytes() else {
            return false;
        };
        self.save_bytes("players.toml", &data)
    }

    pub fn save_records(&self, store: &RecordStore) -> bool {
        let Ok(data) = store.to_toml_bytes() else {
            return false;
        };
        self.save_bytes("hiscores.toml", &data)
    }

    pub fn save_persistent_state(
        &self,
        config: &Config,
        profiles: &ProfileStore,
        records: &RecordStore,
    ) -> bool {
        let config_saved = self.save_config(config);
        let players_saved = self.save_players(profiles);
        let records_saved = self.save_records(records);
        config_saved & players_saved & records_saved
    }

    pub fn load_players(&self) -> Result<ProfileStore, String> {
        load_or_default(
            &self.files,
            "players.toml",
            true,
            ProfileStore::from_toml_bytes,
            || ProfileStore::from_toml_bytes(&self.files.read("players.toml")),
            ProfileStore::to_toml_bytes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn persistent_state_save_writes_config_profiles_and_records() {
        let save_dir = tempfile::tempdir().unwrap();
        let files = Rc::new(FileStore::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            save_dir.path().to_path_buf(),
        ));
        let manager = SaveManager::new(files.clone());
        let config = Config::default();
        let profiles =
            ProfileStore::from_toml_bytes(&files.read("players.toml")).expect("bundled players");
        let records = RecordStore::cleared_default(&files).expect("bundled records");

        manager.save_persistent_state(&config, &profiles, &records);

        assert_eq!(
            files.read_save("config.toml"),
            config.to_toml_bytes().expect("config serializes")
        );
        assert_eq!(
            files.read_save("players.toml"),
            profiles.to_toml_bytes().expect("players serialize")
        );
        assert_eq!(
            files.read_save("hiscores.toml"),
            records.to_toml_bytes().expect("records serialize")
        );
    }

    #[test]
    fn missing_players_fall_back_to_bundled_roster() {
        let save_dir = tempfile::tempdir().unwrap();
        let files = Rc::new(FileStore::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            save_dir.path().to_path_buf(),
        ));
        let manager = SaveManager::new(files.clone());

        let profiles = manager.load_players().expect("bundled players");

        assert!(!profiles.profiles.is_empty());
        assert_eq!(
            files.read_save("players.toml"),
            profiles.to_toml_bytes().expect("players serialize")
        );
    }

    #[test]
    fn malformed_players_are_rejected_without_overwriting_the_file() {
        let save_dir = tempfile::tempdir().unwrap();
        let files = Rc::new(FileStore::new(
            PathBuf::from("/nonexistent"),
            save_dir.path().to_path_buf(),
        ));
        let invalid = b"format_version = 1\nprofiles = []";
        files.write("players.toml", invalid);
        let manager = SaveManager::new(files.clone());

        let error = manager
            .load_players()
            .expect_err("malformed players must fail");

        assert!(error.starts_with("players.toml: "));
        assert_eq!(files.read_save("players.toml"), invalid);
    }
}
