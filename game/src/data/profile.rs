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

    // Pascal bestresult / best4result — display strings e.g. "2 (-)"
    pub best_result: String,
    pub best_4h_result: String,

    // Pascal bestwcjump — longest WC jump distance
    pub best_wc_jump: usize,

    // Pascal default (Pascal bestwchill: byte) — hill index for best WC jump
    // Display name looked up from hill catalog at render time
    pub bestwchill: usize,

    // Pascal bestjump — overall longest jump distance
    pub best_jump: usize,

    // Pascal default (Pascal besthill: byte) — hill index for overall best jump
    // Display name looked up from hill catalog at render time
    pub besthill_idx: usize,

    // Pascal besthillfile — hill file/landscape name e.g. "HILLBASE"
    pub besthillfile: String,

    // Pascal bestpoints / best4points (word) — points for best result and best 4H result
    pub bestpoints: usize,
    pub best4points: usize,

    // Pascal kothlevel (byte)
    pub koth_level: usize,

    // Display names populated from hill indices for render convenience.
    // These are NOT part of the Pascal file format — they are derived
    // from bestwchill / besthill_idx and the hill catalog at load time.
    pub best_wc_hill_display: String,
    pub best_hill_display: String,
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
            bestwchill: 0,
            best_jump: 0,
            besthill_idx: 0,
            besthillfile: String::new(),
            bestpoints: 0,
            best4points: 0,
            koth_level: 0,
            best_wc_hill_display: String::new(),
            best_hill_display: String::new(),
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
    pub const fn num_profiles(&self) -> usize {
        self.profiles.len()
    }

    #[must_use] 
    pub const fn has_slot(&self) -> bool {
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
