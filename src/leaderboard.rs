#[derive(Debug, PartialEq, Eq, Savefile)]
pub struct LeaderboardEntry {
    pub name: String,
    pub score: usize,
}

#[derive(Debug, PartialEq, Eq, Savefile)]
pub struct Leaderboard {
    entries: Vec<LeaderboardEntry>,
}

impl LeaderboardEntry {
    pub fn new<S: AsRef<str>>(name: S, score: usize) -> LeaderboardEntry {
        assert_eq!(name.as_ref().len(), Leaderboard::entry_name_len());
        LeaderboardEntry {
            name: String::from(name.as_ref()),
            score,
        }
    }
}

impl Leaderboard {
    pub const fn entry_name_len() -> usize {
        3
    }

    pub const fn max_entries() -> usize {
        10
    }

    pub fn new() -> Self {
        Leaderboard { entries: vec![] }
    }

    #[cfg(test)]
    pub fn from_raw(entries: Vec<LeaderboardEntry>) -> Self {
        Leaderboard { entries }
    }

    pub fn load<S: AsRef<str>>(file_name: S) -> Result<Self, String> {
        let load_operation = savefile::load_file(file_name.as_ref(), 0);
        let loaded_leaderboard = load_operation.map_err(|e| format!("{}", e))?;
        Ok(loaded_leaderboard)
    }

    pub fn save<S: AsRef<str>>(&self, file_name: S) {
        // TODO: how would I handle save errors? crash the game? print some diagnostic log?
        savefile::save_file(file_name.as_ref(), 0, self).unwrap()
    }

    #[cfg(test)]
    pub fn serialize(&self) -> Vec<u8> {
        savefile::save_to_mem(0, self).unwrap()
    }

    pub fn get_place_on_leaderboard(&self, score: usize) -> Option<usize> {
        for (i, entry) in self.entries.iter().enumerate() {
            if score > entry.score {
                return Some(i);
            }
        }

        let next_slot = self.entries.len();
        if next_slot < Leaderboard::max_entries() {
            return Some(next_slot);
        }

        None
    }

    pub fn add_score<S: AsRef<str>>(&mut self, name: S, score: usize) {
        let place = self.get_place_on_leaderboard(score);
        assert!(place.is_some());

        let place = place.unwrap();
        self.entries
            .insert(place, LeaderboardEntry::new(name, score));
        if self.entries.len() > Leaderboard::max_entries() {
            self.entries.pop();
        }
    }

    pub fn entry<'a>(&'a self, index: usize) -> Option<&'a LeaderboardEntry> {
        if index < self.entries.len() {
            Some(&self.entries[index])
        } else {
            None
        }
    }
}
