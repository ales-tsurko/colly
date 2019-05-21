use super::pattern::Pattern;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mixer {
    tracks: Vec<Track>,
}

impl Mixer {
    pub fn clone_track(&mut self, index: usize) -> Option<Track> {
        if index > self.tracks.len() {
            return None;
        }
        
        Some(self.tracks[index].clone())
    }

    pub fn set_track(&mut self, track: Track) {
        let index = track.index;
        self.tracks[index] = track;
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    index: usize,
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