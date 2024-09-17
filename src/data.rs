use std::{collections::HashMap, error::Error, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::word::Word;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LearningData {
    words_map: HashMap<String, Word>,
    words_count: usize,
    finished_words_count: usize,
}

impl Default for LearningData {
    fn default() -> Self {
        LearningData {
            words_map: HashMap::new(),
            words_count: 0,
            finished_words_count: 0,
        }
    }
}

impl LearningData {
    pub fn read_file(&mut self, path: &PathBuf) -> LearningData {
        if let Ok(content) = fs::read_to_string(path) {
            serde_json::from_str(&content).unwrap_or(LearningData::default())
        } else {
            LearningData::default()
        }
    }

    pub fn save_file(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let serialized = serde_json::to_string(&self)
            .unwrap_or(String::new());
        fs::write(path, serialized)?;

        Ok(())
    }

    pub fn update(&mut self, words_map: &HashMap<String, Word>) -> &mut Self {
        for key in words_map.keys() {
            self.words_map.insert(key.clone(), words_map.get(key).unwrap().clone());
        }
        self.words_count = self.words_map.len();

        let mut finished_count = 0;
        for value in self.words_map.values() {
            if value.proficiency >= 100 {
                finished_count += 1;
            }
        }
        self.finished_words_count = finished_count;

        self
    }

    pub fn _get_all_words_map(&self) -> HashMap<String, Word> {
        self.words_map.clone()
    }

    pub fn get_words_map(&self, count: usize, max_proficiency: isize) -> HashMap<String, Word> {
        let mut new_words_map: HashMap<String, Word> = self.words_map.iter().filter_map(|(key, value)| {
            if value.proficiency < max_proficiency {
                Some((key.clone(), value.clone()))
            } else {
                None
            }
        })
        .collect();

        if new_words_map.len() > count {
            let key_list: Vec<String> = new_words_map
                .keys()
                .map(|k| {k.clone()})
                .collect();
            for key in &key_list[count..] {
                new_words_map.remove(key);
            }
        }

        new_words_map
    }

    pub fn get_words_count(&self) -> usize {
        self.words_count
    }

    pub fn get_finished_words_count(&self) -> usize {
        self.finished_words_count
    }
}
