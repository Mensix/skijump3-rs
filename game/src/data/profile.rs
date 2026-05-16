pub const MAX_PROFILES: usize = 20;
pub const MAX_ACTIVE_PROFILES: usize = 10;
pub const NUM_SUITS: usize = 8;
pub const NUM_SKIS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub real_name: String,
    pub suit_color: usize,
    pub ski_color: usize,
    pub replace: usize,
    pub coach_style: usize,
    pub skip_quali: usize,
    pub total_jumps: usize,
    pub world_cups: usize,
    pub legs_won: usize,
    pub world_cups_won: usize,
    pub best_result: String,
    pub best_4h_result: String,
    pub best_wc_jump: usize,
    pub best_wc_hill: String,
    pub best_jump: usize,
    pub best_hill: String,
    pub koth_level: usize,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "SKI JUMPER".to_string(),
            real_name: String::new(),
            suit_color: 0,
            ski_color: 0,
            replace: 0,
            coach_style: 1,
            skip_quali: 0,
            total_jumps: 0,
            world_cups: 0,
            legs_won: 0,
            world_cups_won: 0,
            best_result: "-".to_string(),
            best_4h_result: "-".to_string(),
            best_wc_jump: 0,
            best_wc_hill: String::new(),
            best_jump: 0,
            best_hill: String::new(),
            koth_level: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileStore {
    pub profiles: Vec<Profile>,
    pub active_order: Vec<usize>,
}

impl Default for ProfileStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileStore {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            profiles: vec![Profile::default()],
            active_order: vec![0],
        }
    }

    #[must_use] 
    pub fn num_profiles(&self) -> usize {
        self.profiles.len()
    }

    #[must_use] 
    pub fn has_slot(&self) -> bool {
        self.num_profiles() < MAX_PROFILES
    }

    #[must_use] 
    pub fn order_pos(&self, profile_index: usize) -> Option<usize> {
        self.active_order
            .iter()
            .position(|&idx| idx == profile_index)
    }

    pub fn add_to_order(&mut self, profile_index: usize) {
        if self.order_pos(profile_index).is_none() && self.active_order.len() < MAX_ACTIVE_PROFILES
        {
            self.active_order.push(profile_index);
        }
    }

    pub fn remove_from_order(&mut self, profile_index: usize) {
        self.active_order.retain(|&idx| idx != profile_index);
    }

    pub fn remove_profile(&mut self, profile_index: usize) {
        if self.profiles.len() <= 1 {
            self.profiles[0] = Profile::default();
            self.active_order = vec![0];
            return;
        }

        if profile_index >= self.profiles.len() {
            return;
        }

        self.profiles.remove(profile_index);
        self.active_order.retain(|&idx| idx != profile_index);
        for idx in &mut self.active_order {
            if *idx > profile_index {
                *idx -= 1;
            }
        }
        if self.active_order.is_empty() {
            self.active_order.push(0);
        }
    }
}
