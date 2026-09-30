use crate::content::ContentStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::save::config::Config;
use crate::save::{load_or_default, SaveManager};
use crate::store::{GameState, Resources, ResourcesRef};
use crate::ui::Font;
use std::rc::Rc;

use super::StartupReset;

pub(super) struct AppLoad {
    pub(super) resources: ResourcesRef,
    pub(super) save_manager: Rc<SaveManager>,
    pub(super) state: GameState,
}

pub(super) fn load_app(
    files: Rc<FileStore>,
    font: Font,
    content_store: ContentStore,
    startup_reset: StartupReset,
) -> Result<AppLoad, String> {
    let langbase = Rc::new(content_store.langbase);
    let config = load_config(&files, startup_reset)?;
    langbase.apply_saved_language(config.language);
    let save_manager = Rc::new(SaveManager::new(files.clone()));
    let records = load_records(&files, startup_reset)?;
    let profiles = save_manager.load_players()?;
    let resources = Rc::new(Resources::new(
        font,
        langbase,
        content_store.namesets,
        content_store.hills,
        files,
    ));
    let state = GameState::new(records, profiles, config);
    Ok(AppLoad {
        resources,
        save_manager,
        state,
    })
}

fn load_config(files: &FileStore, startup_reset: StartupReset) -> Result<Config, String> {
    load_or_default(
        files,
        "config.toml",
        startup_reset != StartupReset::Config,
        Config::from_toml_bytes,
        || Ok(Config::default()),
        Config::to_toml_bytes,
    )
}

fn load_records(files: &FileStore, startup_reset: StartupReset) -> Result<RecordStore, String> {
    let default = || {
        let mut records = RecordStore::bundled_default(files)
            .map_err(|error| format!("hiscores.toml: {error}"))?;
        if startup_reset == StartupReset::ZeroRecords {
            for top in &mut records.top {
                top.pos = 0;
                top.score = 0.0;
            }
            for record in records.hill_records.values_mut() {
                record.len = 0.0;
            }
        }
        Ok(records)
    };
    load_or_default(
        files,
        "hiscores.toml",
        matches!(startup_reset, StartupReset::None | StartupReset::Config),
        RecordStore::from_toml_bytes,
        default,
        RecordStore::to_toml_bytes,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn real_assets() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    #[test]
    fn record_resets_are_persisted_immediately() {
        let save_dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(real_assets(), save_dir.path().to_path_buf());
        let stale = RecordStore::default().to_toml_bytes().unwrap();
        files.write("hiscores.toml", &stale);

        let records = load_records(&files, StartupReset::Records).unwrap();
        let expected = RecordStore::bundled_default(&files).unwrap();
        assert_eq!(records, expected);
        assert_eq!(
            files.read_save("hiscores.toml"),
            records.to_toml_bytes().unwrap()
        );

        let records = load_records(&files, StartupReset::ZeroRecords).unwrap();
        let expected = RecordStore::cleared_default(&files).unwrap();
        assert_eq!(records, expected);
        assert_eq!(
            files.read_save("hiscores.toml"),
            records.to_toml_bytes().unwrap()
        );
    }

    #[test]
    fn config_reset_is_persisted_immediately() {
        let save_dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(PathBuf::from("/nonexistent"), save_dir.path().to_path_buf());
        let saved = Config {
            language: 4,
            ..Config::default()
        };
        let stale = saved.to_toml_bytes().unwrap();
        files.write("config.toml", &stale);

        assert_eq!(load_config(&files, StartupReset::None).unwrap().language, 4);
        let config = load_config(&files, StartupReset::Config).unwrap();
        assert_eq!(config.language, Config::default().language);
        assert_eq!(
            files.read_save("config.toml"),
            config.to_toml_bytes().unwrap()
        );
    }

    #[test]
    fn malformed_config_is_rejected_without_overwriting_the_file() {
        let save_dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(PathBuf::from("/nonexistent"), save_dir.path().to_path_buf());
        files.write("config.toml", b"not = [valid");

        let error =
            load_config(&files, StartupReset::None).expect_err("malformed config must fail");

        assert!(error.starts_with("config.toml: "));
        assert_eq!(files.read_save("config.toml"), b"not = [valid");
    }

    #[test]
    fn semantically_invalid_records_are_rejected_without_overwriting_the_file() {
        let save_dir = tempfile::tempdir().unwrap();
        let files = FileStore::new(real_assets(), save_dir.path().to_path_buf());
        let invalid = b"format_version = 1\ntop = []\n[hill_records]\n[hill_goals]\n";
        files.write("hiscores.toml", invalid);

        let error =
            load_records(&files, StartupReset::None).expect_err("invalid records must fail");

        assert!(error.starts_with("hiscores.toml: "));
        assert_eq!(files.read_save("hiscores.toml"), invalid);
    }

    #[cfg(unix)]
    #[test]
    fn config_read_permission_error_does_not_overwrite_original() {
        use std::os::unix::fs::PermissionsExt;
        let save_dir = tempfile::tempdir().unwrap();
        let path = save_dir.path().join("config.toml");
        std::fs::write(&path, b"private").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();
        let files = FileStore::new(PathBuf::from("/nonexistent"), save_dir.path().to_path_buf());

        assert!(load_config(&files, StartupReset::None).is_err());
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"private");
    }
}
