#[derive(Debug, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub name: String,
    pub score: usize,
}

#[derive(Debug, PartialEq, Eq)]
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

    pub fn load<S: AsRef<str>>(_file_name: S) -> Option<Self> {
        Some(Leaderboard {
            entries: vec![
                LeaderboardEntry::new("TS1", 2000),
                LeaderboardEntry::new("TS2", 1500),
                LeaderboardEntry::new("TS3", 1000),
                LeaderboardEntry::new("TS4", 500),
            ],
        })
    }

    pub fn save<S: AsRef<str>>(&self, _file_name: S) {
        // unimplemented!();
    }

    pub fn serialize(&self) -> String {
        unimplemented!();
    }

    pub fn get_place_on_leaderboard(&self, score: usize) -> Option<usize> {
        for (i, entry) in self.entries.iter().enumerate() {
            if score > entry.score {
                return Some(i);
            }
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
