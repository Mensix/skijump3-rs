use crate::content::ContentStore;
use crate::data::records::RecordStore;
use crate::files::FileStore;
use crate::save::{SaveManager, SaveRef};
use crate::store::{Resources, ResourcesRef, Store, StoreRef};
use engine::ui::Font;
use std::rc::Rc;

const HISCORES_TOML: &str = "hiscores.toml";

pub(super) struct GameState {
    pub(super) resources: ResourcesRef,
    pub(super) save_manager: SaveRef,
    pub(super) store: StoreRef,
}

impl GameState {
    pub(super) fn load(
        files: Rc<FileStore>,
        font: Font,
        content_store: ContentStore,
    ) -> Result<Self, String> {
        let langbase = Rc::new(content_store.langbase);
        let save_manager: SaveRef = Rc::new(SaveManager::new(files.clone(), langbase.clone()));
        let records = load_records(&files)?;
        let profiles = save_manager.load_players();

        let resources: ResourcesRef = Rc::new(Resources::new(
            font,
            langbase,
            content_store.namesets,
            content_store.hills,
            files,
            save_manager.clone(),
        ));
        let store: StoreRef = Rc::new(Store::from_loaded_data(records, profiles));
        store.configure_from_save(&save_manager);

        Ok(Self {
            resources,
            save_manager,
            store,
        })
    }

    pub(super) fn starts_with_welcome(&self) -> bool {
        self.save_manager.config.borrow().languagenumber == 255
    }
}

fn load_records(files: &FileStore) -> Result<RecordStore, String> {
    let records_data = files.read(HISCORES_TOML).map_err(|e| e.to_string())?;
    RecordStore::from_toml_bytes(&records_data).map_err(|e| e.to_string())
}
