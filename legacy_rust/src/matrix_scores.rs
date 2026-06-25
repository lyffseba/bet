use directories::ProjectDirs;
use std::fs;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScoreEntry {
    pub score: u32,
    pub name: String,
    pub combo: u32,
    pub level: u32,
}

pub fn get_scores() -> Vec<ScoreEntry> {
    if let Some(proj_dirs) = ProjectDirs::from("xyz", "lyffseba", "bet") {
        let path = proj_dirs.data_dir().join("matrix_scores.txt");
        if let Ok(contents) = fs::read_to_string(&path) {
            let mut scores = Vec::new();
            for line in contents.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() == 4
                    && let (Ok(score), Ok(combo), Ok(level)) = (parts[1].parse(), parts[2].parse(), parts[3].parse()) {
                        scores.push(ScoreEntry {
                            name: parts[0].to_string(),
                            score,
                            combo,
                            level,
                        });
                    }
            }
            scores.sort_by(|a, b| b.score.cmp(&a.score));
            return scores;
        }
    }
    Vec::new()
}

pub fn save_score(entry: ScoreEntry) {
    if let Some(proj_dirs) = ProjectDirs::from("xyz", "lyffseba", "bet") {
        let dir = proj_dirs.data_dir();
        let _ = fs::create_dir_all(dir);
        let path = dir.join("matrix_scores.txt");
        
        let mut scores = get_scores();
        scores.push(entry);
        scores.sort_by(|a, b| b.score.cmp(&a.score));
        scores.truncate(50); // Keep top 50
        
        let mut content = String::new();
        for s in scores {
            content.push_str(&format!("{},{},{},{}\n", s.name.replace(',', ""), s.score, s.combo, s.level));
        }
        let _ = fs::write(&path, content);
    }
}
