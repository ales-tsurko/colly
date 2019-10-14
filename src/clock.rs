use crate::settings;
use std::cmp::{Ord, Ordering};

pub type Resolution = u64;

#[derive(Debug, Clone)]
pub struct Clock {
    tempo: Bpm,
    cursor: Cursor,
}

impl Clock {
    pub fn new(tempo: Bpm, options: &settings::Clock) -> Self {
        Clock {
            tempo,
            cursor: Cursor::new(options.resolution),
        }
    }

    pub fn set_tempo(&mut self, tempo: Bpm) {
        self.tempo = tempo;
    }

    pub fn tempo(&self) -> Bpm {
        self.tempo
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    //TODO:
    // pub fn tick_interval(&self)
}

impl Default for Clock {
    fn default() -> Self {
        Clock::new(Bpm::default(), &settings::Clock::default())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cursor {
    pub position: CursorPosition,
    resolution: Resolution,
}

impl Cursor {
    pub fn new(resolution: Resolution) -> Self {
        Cursor {
            position: CursorPosition::new(resolution),
            resolution,
        }
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    /// Sets the `Cursor` position's beat and tick to `0`s.
    pub fn reset(&mut self) {
        self.position.beat = 0;
        self.position.tick = 0;
    }
}

impl Default for Cursor {
    fn default() -> Cursor {
        Cursor {
            position: CursorPosition::default(),
            resolution: settings::Clock::default().resolution,
        }
    }
}

impl Iterator for Cursor {
    type Item = CursorPosition;

    fn next(&mut self) -> Option<Self::Item> {
        self.position.increment();
        Some(self.position)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorPosition {
    beat: u64,
    tick: u64,
    resolution: Resolution,
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self {
            beat: 0,
            tick: 0,
            resolution: settings::Clock::default().resolution,
        }
    }
}

impl CursorPosition {
    pub fn new(resolution: Resolution) -> Self {
        Self {
            beat: 0,
            tick: 0,
            resolution,
        }
    }

    pub fn increment(&mut self) {
        self.tick += 1;

        if self.tick >= self.resolution {
            self.beat += 1;
            self.tick = 0;
        }
    }

    /// Construct an instance from ticks with given resolution.
    pub fn from_ticks(ticks: u64, resolution: Resolution) -> Self {
        Self {
            beat: ticks / resolution,
            tick: ticks % resolution,
            resolution,
        }
    }

    /// Constructs an instance from an `f64` value, which considered as
    /// a relative to the zero beat position.
    pub fn from_f64(position: f64, resolution: Resolution) -> Self {
        CursorPosition {
            beat: position as u64,
            tick: (position.fract() * (resolution as f64)).round() as u64,
            resolution,
        }
    }

    /// Interprets `self` as a relative to the zero beat position.
    pub fn as_f64(&self) -> f64 {
        (self.beat as f64) + ((self.tick as f64) / (self.resolution as f64))
    }

    pub fn as_ticks(&self) -> u64 {
        self.beat * self.resolution + self.tick
    }

    pub fn beat(&self) -> u64 {
        self.beat
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }
}

impl std::ops::Add for CursorPosition {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let r_ticks = other.beat * self.resolution + other.tick;
        CursorPosition::from_ticks(self.as_ticks() + r_ticks, self.resolution)
    }
}

impl std::ops::Sub for CursorPosition {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let right_ticks = other.beat * self.resolution + other.tick;
        CursorPosition::from_ticks(
            self.as_ticks().saturating_sub(right_ticks),
            self.resolution,
        )
    }
}

impl std::ops::Add<u64> for CursorPosition {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        CursorPosition::from_ticks(self.as_ticks() + other, self.resolution)
    }
}

impl std::ops::Sub<u64> for CursorPosition {
    type Output = Self;

    fn sub(self, other: u64) -> Self {
        CursorPosition::from_ticks(
            self.as_ticks().saturating_sub(other),
            self.resolution,
        )
    }
}

impl std::ops::Mul<f64> for CursorPosition {
    type Output = Self;

    fn mul(self, other: f64) -> Self {
        CursorPosition::from_ticks(
            (self.as_ticks() as f64 * other).round() as u64,
            self.resolution,
        )
    }
}

impl std::ops::Div<f64> for CursorPosition {
    type Output = Self;

    fn div(self, rhs: f64) -> Self {
        CursorPosition::from_ticks(
            (self.as_ticks() as f64 / rhs).round() as u64,
            self.resolution,
        )
    }
}

impl std::ops::AddAssign for CursorPosition {
    fn add_assign(&mut self, other: Self) {
        let r_ticks = other.beat * self.resolution + other.tick;
        let position =
            Self::from_ticks(self.as_ticks() + r_ticks, self.resolution);
        self.beat = position.beat;
        self.tick = position.tick;
    }
}

impl std::ops::SubAssign for CursorPosition {
    fn sub_assign(&mut self, other: Self) {
        let right_ticks = other.beat * self.resolution + other.tick;
        let position = Self::from_ticks(
            self.as_ticks().saturating_sub(right_ticks),
            self.resolution,
        );
        self.beat = position.beat;
        self.tick = position.tick;
    }
}

impl std::ops::AddAssign<u64> for CursorPosition {
    fn add_assign(&mut self, other: u64) {
        let position =
            Self::from_ticks(self.as_ticks() + other, self.resolution);
        self.beat = position.beat;
        self.tick = position.tick;
    }
}

impl std::ops::SubAssign<u64> for CursorPosition {
    fn sub_assign(&mut self, other: u64) {
        let position = Self::from_ticks(
            self.as_ticks().saturating_sub(other),
            self.resolution,
        );
        self.beat = position.beat;
        self.tick = position.tick;
    }
}

impl std::ops::MulAssign<f64> for CursorPosition {
    fn mul_assign(&mut self, other: f64) {
        let position = Self::from_ticks(
            (self.as_ticks() as f64 * other).round() as u64,
            self.resolution,
        );
        self.beat = position.beat;
        self.tick = position.tick;
    }
}

impl std::ops::DivAssign<f64> for CursorPosition {
    fn div_assign(&mut self, other: f64) {
        let position = Self::from_ticks(
            (self.as_ticks() as f64 / other).round() as u64,
            self.resolution,
        );
        self.beat = position.beat;
        self.tick = position.tick;
    }
}

impl From<(u64, u64, Resolution)> for CursorPosition {
    fn from(value: (u64, u64, Resolution)) -> Self {
        CursorPosition {
            beat: value.0,
            tick: value.1,
            resolution: value.2,
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

pub type Duration = CursorPosition;

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
        let resolution = 1920;
        let result = CursorPosition::from_f64(16.98765, resolution);
        let expected = CursorPosition {
            beat: 16,
            tick: 1896,
            resolution,
        };

        assert_eq!(expected, result);
    }

    #[test]
    fn position_as_relative() {
        let resolution = 1920;
        let position = CursorPosition {
            beat: 16,
            tick: 1896,
            resolution,
        };
        let result = position.as_f64();

        assert_relative_eq!(16.9875, result);
    }

    #[test]
    fn increment_cursor() {
        let resolution = 24;
        let mut cursor_position = CursorPosition::new(resolution);
        let expected = CursorPosition {
            beat: 1,
            tick: 2,
            resolution,
        };

        for _ in 0..=resolution + 1 {
            cursor_position.increment();
        }

        assert_eq!(expected, cursor_position);
    }

    #[test]
    fn cursor_add() {
        let mut cursor = Cursor::new(24);
        cursor.nth(3);
        let offset = CursorPosition {
            beat: 10,
            // such a tick value shouldn't be possible when
            // resolution is 24, but for battle-like circumstances...
            tick: 73,
            resolution: cursor.resolution,
        };

        let expected = CursorPosition {
            beat: 13,
            tick: 5,
            resolution: cursor.resolution,
        };

        assert_eq!(expected, cursor.position + offset);
    }

    #[test]
    fn cursor_add_assign() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 13,
            tick: 5,
            resolution,
        };

        cursor.position += CursorPosition {
            beat: 1,
            tick: 0,
            resolution,
        };

        assert_eq!(expected, cursor.position);
    }

    #[test]
    fn cursor_sub() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let offset = CursorPosition {
            beat: 5,
            // such a tick value shouldn't be possible when
            // resolution is 24, but for battle-like circumstances...
            tick: 73,
            resolution,
        };
        let expected = CursorPosition {
            beat: 4,
            tick: 4,
            resolution,
        };

        assert_eq!(expected, cursor.position - offset);
    }

    #[test]
    fn cursor_sub_assign() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 11,
            tick: 5,
            resolution,
        };

        cursor.position -= CursorPosition {
            beat: 1,
            tick: 0,
            resolution,
        };

        assert_eq!(expected, cursor.position);
    }

    #[test]
    fn cursor_add_u64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 13,
            tick: 5,
            resolution,
        };

        assert_eq!(expected, cursor.position + resolution);
    }

    #[test]
    fn cursor_add_assign_u64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 13,
            tick: 5,
            resolution,
        };

        cursor.position += resolution;

        assert_eq!(expected, cursor.position);
    }

    #[test]
    fn cursor_sub_u64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 11,
            tick: 5,
            resolution,
        };

        assert_eq!(expected, cursor.position - resolution);
    }

    #[test]
    fn cursor_sub_assign_u64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 11,
            tick: 5,
            resolution,
        };

        cursor.position -= resolution;

        assert_eq!(expected, cursor.position);
    }

    #[test]
    fn cursor_mul_f64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 18,
            tick: 8,
            resolution,
        };

        assert_eq!(expected, cursor.position * 1.5);
    }

    #[test]
    fn cursor_mul_assign_f64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 18,
            tick: 8,
            resolution,
        };

        cursor.position *= 1.5;

        assert_eq!(expected, cursor.position);
    }

    #[test]
    fn cursor_div_f64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 24,
            tick: 10,
            resolution,
        };

        assert_eq!(expected, cursor.position / 0.5);
    }

    #[test]
    fn cursor_div_assign_f64() {
        let resolution = 24;
        let mut cursor = Cursor::new(resolution);
        cursor.position = (12, 5, resolution).into();

        let expected = CursorPosition {
            beat: 24,
            tick: 10,
            resolution,
        };

        cursor.position /= 0.5;

        assert_eq!(expected, cursor.position);
    }
}
