use super::pattern::Pattern;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct Mixer {
    tracks: HashMap<usize, Rc<Track>>,
}

impl Mixer {
    pub fn track(&mut self, index: usize) -> Rc<Track> {
        self.tracks.entry(index).or_default().clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Track {
    slots: HashMap<usize, Rc<Slot>>,
}

impl Track {
    pub fn slot(&mut self, index: usize) -> Rc<Slot> {
        self.slots.entry(index).or_default().clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Slot {
    pattern: Pattern,
}
