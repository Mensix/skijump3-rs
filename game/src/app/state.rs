use crate::content::ContentStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::save::{load_initial_config, SaveManager, SaveRef};
use crate::store::{GameStateRef, Resources, ResourcesRef};
use engine::oxide::Font;
use std::cell::RefCell;
use std::rc::Rc;

const HISCORES_TOML: &str = "hiscores.toml";

pub(super) struct AppState {
    pub(super) resources: ResourcesRef,
    pub(super) save_manager: SaveRef,
    pub(super) state: GameStateRef,
}

impl AppState {
    pub(super) fn load(
        files: Rc<FileStore>,
        font: Font,
        content_store: ContentStore,
    ) -> Result<Self, String> {
        let langbase = Rc::new(content_store.langbase);
        let config = load_initial_config(&files, &langbase);
        let save_manager: SaveRef = Rc::new(SaveManager::new(files.clone()));
        let records = load_records(&files)?;
        let profiles = save_manager.load_players();

        let resources: ResourcesRef = Rc::new(Resources::new(
            font,
            langbase,
            content_store.namesets,
            content_store.hills,
            files,
        ));
        let state: GameStateRef = Rc::new(RefCell::new(crate::store::GameState::new(
            records, profiles, config,
        )));

        Ok(Self {
            resources,
            save_manager,
            state,
        })
    }
}

fn load_records(files: &FileStore) -> Result<RecordStore, String> {
    let records_data = files.read(HISCORES_TOML).map_err(|e| e.to_string())?;
    RecordStore::from_toml_bytes(&records_data).map_err(|e| e.to_string())
}
