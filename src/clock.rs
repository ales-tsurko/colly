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
}

impl Default for Clock {
    fn default() -> Self {
        Clock::new(Bpm::default(), 1920)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cursor {
    position: CursorPosition,
    resolution: u64,
}

impl Cursor {
    pub fn new(resolution: u64) -> Self {
        Cursor {
            position: CursorPosition::default(),
            resolution,
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CursorPosition {
    beat: u64,
    tick: u64,
}

impl CursorPosition {
    pub fn from_relative_position(position: f64, resolution: u64) -> Self {
        let beat = position as u64;
        let tick = ((position - (beat as f64)) * (resolution as f64)).round() as u64;
        CursorPosition {
            beat,
            tick,
        }
    }

    pub fn as_relative_position(&self, resolution: u64) -> f64 {
        (self.beat as f64) + ((self.tick as f64) / (resolution as f64))
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
        let expected = CursorPosition {
            beat: 1,
            tick: 2,
        };

        assert_eq!(Some(expected), cursor.nth(25));

    }
}

