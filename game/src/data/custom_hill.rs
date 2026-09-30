use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CustomHillCatalog {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) checksum_version: u32,
    pub(crate) hills: Vec<CustomHill>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CustomHill {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) terrain_index: String,
    pub(crate) kr: i64,
    pub(crate) front_index: String,
    pub(crate) back_index: String,
    pub(crate) back_brightness: i64,
    pub(crate) back_mirror: bool,
    pub(crate) vx_final: i64,
    pub(crate) pk_hundred: i64,
    pub(crate) pl_save_ten_thousand: i64,
    pub(crate) author: String,
    pub(crate) checksum: i64,
    pub(crate) profile_checksum: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    const CURRENT: &str = r#"
id = "TEST"
name = "Test custom hill"
checksum_version = 1

[[hills]]
id = "TEST"
name = "Test"
terrain_index = "1"
kr = 120
front_index = "1"
back_index = "4"
back_brightness = 75
back_mirror = true
vx_final = 140
pk_hundred = 100
pl_save_ten_thousand = 3210
author = "test"
checksum = 1
profile_checksum = 1
"#;

    #[test]
    fn current_schema_is_strict() {
        assert!(toml::from_str::<CustomHillCatalog>(CURRENT).is_ok());
        assert!(toml::from_str::<CustomHillCatalog>(&format!("{CURRENT}\nlegacy = true")).is_err());
        assert!(toml::from_str::<CustomHillCatalog>(
            &CURRENT.replace("checksum_version = 1\n", "")
        )
        .is_err());
        assert!(toml::from_str::<CustomHillCatalog>(
            &CURRENT.replace("terrain_index = \"1\"", "terrain_index = 1")
        )
        .is_err());
    }
}
