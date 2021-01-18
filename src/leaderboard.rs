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

    #[cfg(test)]
    pub fn from_raw(entries: Vec<LeaderboardEntry>) -> Self {
        Leaderboard { entries }
    }

    pub fn load<P: AsRef<std::path::Path>>(file_name: P) -> Result<Self, &'static str> {
        let mut lines = Vec::new();

        let file = std::fs::File::open(file_name).map_err(|_| "File not found")?;
        use std::io::BufRead;
        for line in std::io::BufReader::new(file).lines() {
            let checked_line = line.map_err(|_| "unreadable line in file")?;
            lines.push(checked_line);
        }

        if lines.len() > Leaderboard::max_entries() {
            return Err("Too many lines in file");
        }

        let mut last_score = std::usize::MAX;
        let mut entries = Vec::new();
        for line in lines {
            let mut line_elements = line.split(" ");
            let name = line_elements.next().ok_or("missing name entry in file")?;
            let score: usize = line_elements
                .next()
                .ok_or("missing score entry in file")?
                .parse()
                .map_err(|_| "invalid score entry in file")?;

            if score > last_score {
                return Err("Leaderboard file corrupted");
            }

            last_score = score;
            entries.push(LeaderboardEntry::new(name, score));
        }

        Ok(Leaderboard { entries })
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, file_name: P) {
        let file = std::fs::File::create(&file_name).unwrap();
        let mut writer = std::io::LineWriter::new(file);

        use std::io::Write;
        writer.write_all(self.serialize().as_bytes()).unwrap();
    }

    pub fn serialize(&self) -> String {
        let mut serialized_board = String::new();
        for entry in &self.entries {
            serialized_board += &format!("{} {}\n", entry.name, entry.score);
        }
        serialized_board
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
