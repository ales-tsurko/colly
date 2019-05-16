use super::pattern::Pattern;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mixer {
    tracks: Vec<Track>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    slots: Vec<Slot>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Slot {
    pattern: Pattern,
}
