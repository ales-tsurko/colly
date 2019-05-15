use super::pattern::Pattern;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Mixer {
    tracks: Vec<Track>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Track {
    slots: Vec<Slot>,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Slot {
    pattern: Pattern,
}
