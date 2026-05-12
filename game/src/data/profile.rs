pub const MAX_PROFILES: usize = 20;

#[derive(Clone, Debug)]
pub struct Profile {
    pub name: String,
}

impl Default for Profile {
    fn default() -> Self {
        Self { name: String::new() }
    }
}

pub struct ProfileStore {
    pub profiles: Vec<Profile>,
    pub edit_index: usize,
}

impl ProfileStore {
    pub fn new() -> Self {
        let profiles = vec![Profile {
            name: "SKI JUMPER".to_string(),
        }];
        Self { profiles, edit_index: 0 }
    }

    pub fn num_profiles(&self) -> usize {
        self.profiles.len()
    }

    pub fn has_slot(&self) -> bool {
        self.num_profiles() < MAX_PROFILES
    }
}
