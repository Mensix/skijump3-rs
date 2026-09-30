use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub(crate) fn generated_hill_image_path(index: &str, kind: &str) -> String {
    format!("hills/generated/HILL{index}/{kind}.png")
}

pub(crate) const FALLBACK_HILL_FILENAME: &str = "HILLBASE";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HillInfo {
    pub name: String,
    pub kr: i64,
    pub front_index: String,
    pub back_index: String,
    pub back_brightness: i64,
    pub back_mirror: i64,
    pub vx_final: i64,
    pub pk_hundred: i64,
    pub pl_save_ten_thousand: i64,
    pub author: String,
    pub checksum: i64,
    pub profile_checksum: i64,
    pub terrain_id: String,
    /// Stable identifier for record lookup: `{catalog_id}:{hill_id}`.
    pub record_key: String,
}

impl HillInfo {
    pub fn pk(&self) -> f64 {
        self.pk_hundred as f64 / 100.0
    }

    pub fn pl_save(&self) -> f64 {
        self.pl_save_ten_thousand as f64 / 10_000.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct HillCatalog {
    inner: Rc<RefCell<HillCatalogInner>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct HillCatalogInner {
    hills: Vec<HillInfo>,
    original_count: usize,
}

impl HillCatalog {
    /// `original_count` is the number of built-in hills; hills beyond that
    /// are user-created custom hills and are excluded from competitions.
    pub fn new(hills: Vec<HillInfo>, original_count: usize) -> Self {
        Self {
            inner: Rc::new(RefCell::new(HillCatalogInner {
                hills,
                original_count,
            })),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().hills.len()
    }

    /// Number of built-in hills (excludes custom hills).
    pub fn original_count(&self) -> usize {
        self.inner.borrow().original_count
    }

    pub fn hill(&self, idx: usize) -> Option<Ref<'_, HillInfo>> {
        Ref::filter_map(self.inner.borrow(), |inner| inner.hills.get(idx)).ok()
    }

    pub(crate) fn replace(&self, replacement: HillCatalog) {
        *self.inner.borrow_mut() = replacement.inner.borrow().clone();
    }

    pub fn index_by_record_key(&self, record_key: &str) -> Option<usize> {
        self.inner
            .borrow()
            .hills
            .iter()
            .position(|hill| hill.record_key == record_key)
    }

    pub fn replay_hill_index(
        &self,
        recorded_idx: usize,
        filename: &str,
        profile_checksum: i32,
    ) -> Option<usize> {
        if filename.eq_ignore_ascii_case(FALLBACK_HILL_FILENAME) {
            let hill = self.hill(recorded_idx)?;
            return (hill.profile_checksum == i64::from(profile_checksum)).then_some(recorded_idx);
        }

        let legacy_name = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
        let legacy_stem = legacy_name
            .rsplit_once('.')
            .map_or(legacy_name, |(stem, _)| stem);
        self.inner
            .borrow()
            .hills
            .iter()
            .enumerate()
            .find(|(_, hill)| {
                if hill.profile_checksum != i64::from(profile_checksum) {
                    return false;
                }
                if hill.record_key == filename {
                    return true;
                }
                hill.record_key
                    .split_once(':')
                    .is_some_and(|(catalog_id, hill_id)| {
                        catalog_id.eq_ignore_ascii_case(legacy_stem)
                            || hill_id.eq_ignore_ascii_case(legacy_stem)
                    })
            })
            .map(|(idx, _)| idx)
    }
}

impl PartialEq for HillCatalog {
    fn eq(&self, other: &Self) -> bool {
        *self.inner.borrow() == *other.inner.borrow()
    }
}

impl Eq for HillCatalog {}

pub fn metadata_checksum(hill: &HillInfo) -> i64 {
    let mut hash = 2_166_136_261u32;
    let canonical = format!(
        "{}\u{ff}{}\u{ff}{}\u{ff}{}\u{ff}{}\u{ff}{}\u{ff}{}\u{ff}{}\u{ff}{}\u{ff}{}\u{ff}{}",
        hill.name,
        hill.kr,
        hill.front_index,
        hill.back_index,
        hill.back_brightness,
        hill.back_mirror,
        hill.vx_final,
        hill.pk_hundred,
        hill.pl_save_ten_thousand,
        hill.author,
        hill.profile_checksum,
    );
    for byte in canonical.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    i64::from(hash & 0x7fff_ffff).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hill(record_key: &str, profile_checksum: i64) -> HillInfo {
        HillInfo {
            record_key: record_key.to_string(),
            profile_checksum,
            terrain_id: format!("terrain-{record_key}"),
            ..HillInfo::default()
        }
    }

    #[test]
    fn replay_hills_resolve_builtin_by_index_and_custom_by_stable_key() {
        let catalog = HillCatalog::new(vec![hill("original:0", 111), hill("my-hill:main", 222)], 1);

        assert_eq!(catalog.replay_hill_index(0, "HILLBASE", 111), Some(0));
        assert_eq!(catalog.replay_hill_index(99, "my-hill:main", 222), Some(1));
        assert_eq!(catalog.hill(1).unwrap().terrain_id, "terrain-my-hill:main");
        assert_eq!(catalog.replay_hill_index(0, "HILLBASE", 999), None);
        assert_eq!(catalog.replay_hill_index(1, "my-hill:main", 999), None);
    }

    #[test]
    fn legacy_extra_hill_filename_resolves_converted_catalog_key_with_checksum() {
        let catalog =
            HillCatalog::new(vec![hill("original:A", 111), hill("LEGACY:LEGACY", 222)], 1);

        assert_eq!(catalog.replay_hill_index(99, "legacy.SJH", 222), Some(1));
        assert_eq!(
            catalog.replay_hill_index(99, "C:\\HILLS\\LEGACY.SJH", 222),
            Some(1)
        );
        assert_eq!(catalog.replay_hill_index(99, "legacy.SJH", 999), None);
    }

    #[test]
    fn metadata_checksum_is_nonzero_deterministic_and_covers_profile() {
        let mut info = hill("custom:test", 1234);
        info.name = "Test".to_string();
        info.front_index = "1".to_string();
        let checksum = metadata_checksum(&info);

        assert_ne!(checksum, 0);
        assert_eq!(checksum, metadata_checksum(&info));
        info.profile_checksum += 1;
        assert_ne!(checksum, metadata_checksum(&info));
    }

    #[test]
    fn replacing_a_catalog_updates_all_clones() {
        let catalog = HillCatalog::new(vec![hill("original:0", 1)], 1);
        let shared = catalog.clone();

        catalog.replace(HillCatalog::new(
            vec![hill("original:0", 1), hill("custom:new", 2)],
            1,
        ));

        assert_eq!(shared.len(), 2);
        assert_eq!(shared.hill(1).unwrap().record_key, "custom:new");
    }
}
