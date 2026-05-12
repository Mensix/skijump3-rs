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
        let mut profiles = Vec::with_capacity(MAX_PROFILES + 1);
        profiles.push(Profile::default());
        profiles.push(Profile {
            name: "SKI JUMPER".to_string(),
        });
        Self { profiles, edit_index: 1 }
    }

    pub fn num_profiles(&self) -> usize {
        self.profiles.len().saturating_sub(1)
    }

    pub fn has_slot(&self) -> bool {
        self.num_profiles() < MAX_PROFILES
    }
}
