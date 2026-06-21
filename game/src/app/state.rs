use crate::content::ContentStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::save::{load_initial_config, SaveManager};
use crate::store::{GameState, Resources, ResourcesRef};
use engine::oxide::Font;
use std::rc::Rc;

pub(super) struct AppLoad {
    pub(super) resources: ResourcesRef,
    pub(super) save_manager: Rc<SaveManager>,
    pub(super) state: GameState,
}

pub(super) fn load_app(files: Rc<FileStore>, font: Font, content_store: ContentStore) -> AppLoad {
    let langbase = Rc::new(content_store.langbase);
    let config = load_initial_config(&files, &langbase);
    let save_manager = Rc::new(SaveManager::new(files.clone()));
    let records = load_records(&files);
    let profiles = save_manager.load_players();
    let resources = Rc::new(Resources::new(
        font,
        langbase,
        content_store.namesets,
        content_store.hills,
        files,
    ));
    let state = GameState::new(records, profiles, config);
    AppLoad {
        resources,
        save_manager,
        state,
    }
}

fn load_records(files: &FileStore) -> RecordStore {
    let records_data = files.read("hiscores.toml");
    RecordStore::from_toml_bytes(&records_data)
}
