use std::cmp::{Ord, Ordering};
use std::ops;

const DEFAULT_RESOLUTION: u64 = 1920;

#[derive(Debug, Clone)]
pub struct Clock {
    tempo: Bpm,
    cursor: Cursor,
}

impl Clock {
    pub fn new(tempo: Bpm, resolution: u64) -> Self {
        Clock {
            tempo,
            cursor: Cursor::new(resolution),
        }
    }

    pub fn set_tempo(&mut self, tempo: Bpm) {
        self.tempo = tempo;
    }

    pub fn tempo(&self) -> Bpm {
        self.tempo
    }

    //TODO:
    // pub fn tick_interval(&self)
}

impl Default for Clock {
    fn default() -> Self {
        Clock::new(Bpm::default(), DEFAULT_RESOLUTION)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cursor {
    pub position: CursorPosition,
    resolution: u64,
}

impl Cursor {
    pub fn new(resolution: u64) -> Self {
        Cursor {
            position: CursorPosition::default(),
            resolution,
        }
    }

    pub fn get_resolution(&self) -> u64 {
        self.resolution
    }
}

impl Default for Cursor {
    fn default() -> Cursor {
        Cursor {
            position: CursorPosition::default(),
            resolution: DEFAULT_RESOLUTION,
        }
    }
}

impl Iterator for Cursor {
    type Item = CursorPosition;

    fn next(&mut self) -> Option<Self::Item> {
        self.position.tick += 1;

        if self.position.tick < self.resolution {
            return Some(self.position);
        }

        self.position.beat += 1;
        self.position.tick = 0;

        Some(self.position)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CursorPosition {
    beat: u64,
    tick: u64,
}

pub type Duration = CursorPosition;

impl CursorPosition {
    pub fn from_relative_position(position: f64, resolution: u64) -> Self {
        let beat = position as u64;
        let tick = (position.fract() * (resolution as f64)).round() as u64;
        CursorPosition { beat, tick }
    }

    pub fn as_relative_position(&self, resolution: u64) -> f64 {
        (self.beat as f64) + ((self.tick as f64) / (resolution as f64))
    }
}

impl From<(u64, u64)> for CursorPosition {
    fn from(value: (u64, u64)) -> Self {
        CursorPosition {
            beat: value.0,
            tick: value.1,
        }
    }
}

impl Ord for CursorPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        let beat_cmp = self.beat.cmp(&other.beat);
        match beat_cmp {
            Ordering::Equal => self.tick.cmp(&other.tick),
            _ => beat_cmp,
        }
    }
}

impl PartialOrd for CursorPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(clippy::suspicious_arithmetic_impl)]
impl ops::Add<CursorPosition> for Cursor {
    type Output = CursorPosition;

    fn add(self, rhs: CursorPosition) -> Self::Output {
        let mut beat = self.position.beat + rhs.beat;
        let mut tick = self.position.tick + rhs.tick;
        if tick >= self.resolution {
            let beat_offset = tick / self.resolution;
            beat += beat_offset;
            tick -= beat_offset * self.resolution + 1;
        }

        CursorPosition { beat, tick }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bpm(f64);

impl Bpm {
    pub fn set(&mut self, value: f64) {
        self.0 = value.min(200.0).max(27.0);
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Bpm(117.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn position_from_relative() {
        let result = CursorPosition::from_relative_position(16.98765, 1920);
        let expected = CursorPosition {
            beat: 16,
            tick: 1896,
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn position_as_relative() {
        let position = CursorPosition {
            beat: 16,
            tick: 1896,
        };
        let result = position.as_relative_position(1920);

        assert_relative_eq!(16.9875, result);
    }

    #[test]
    fn increment_cursor() {
        let mut cursor = Cursor::new(24);
        let expected = CursorPosition { beat: 1, tick: 2 };

        assert_eq!(Some(expected), cursor.nth(25));
    }

    #[test]
    fn cursor_offset() {
        let mut cursor = Cursor::new(24);
        cursor.nth(3);
        let offset = CursorPosition {
            beat: 10,
            // such tick value shouldn't be possible when
            // resolution is 24, but for battle-like circumstances...
            tick: 73,
        };
        let result = cursor + offset;

        let expected = CursorPosition { beat: 13, tick: 4 };

        assert_eq!(expected, result);
    }
}
