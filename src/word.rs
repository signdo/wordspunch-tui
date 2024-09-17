use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Level {
    Repeat = -3,
    Hard = 1,
    Normal = 3,
    Simple = 5,
    Unselected = 0,
}

impl ToString for Level {
    fn to_string(&self) -> String {
        match self {
            Level::Repeat => "Repeat".to_owned(),
            Level::Hard => "Hard".to_owned(),
            Level::Normal => "Normal".to_owned(),
            Level::Simple => "Simple".to_owned(),
            Level::Unselected => "No Selected".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Word {
    pub proficiency: isize,
    pub chinese: String,
    pub last_level: Level,
}

impl Default for Word {
    fn default() -> Self {
        Word {
            proficiency: 0,
            chinese: String::new(),
            last_level: Level::Unselected,
        }
    }
}
