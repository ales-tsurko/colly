use super::pattern::Pattern;
use super::{ValueWrapper, Value};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mixer {
    tracks: Vec<Track>,
}

impl Mixer {
    pub fn track(&mut self, index: usize) -> Option<&mut Track> {
        if index > self.tracks.len() {
            return None;
        }
        
        Some(&mut self.tracks[index])
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    slots: Vec<Slot>,
}

impl Track {
    pub fn slot(&mut self, index: usize) -> Option<&mut Slot> {
        if index > self.slots.len() {
            return None;
        }

        Some(&mut self.slots[index])
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Slot {
    pattern: Pattern,
}