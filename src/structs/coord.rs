use winapi::um::wincon::COORD;

/// Represents a [COORD] which is the position of the characters cell in the console screen buffer,
/// which origin is (0,0).
///
/// link: [https://docs.microsoft.com/en-us/windows/console/coord-str]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd)]
pub struct Coord {
    /// x axis position
    pub x: i16,
    /// y axis position
    pub y: i16,
}

impl Coord {
    /// A default value coord, where x and y are zero.
    pub const ZERO : Coord = Coord{ x: 0, y: 0 };

    /// Create a new size instance with the given x and y.
    pub fn new(x: i16, y: i16) -> Coord {
        Coord { x, y }
    }
}

impl From<COORD> for Coord {
    fn from(coord: COORD) -> Self {
        Coord::new(coord.X, coord.Y)
    }
}

impl Into<COORD> for Coord{
    fn into(self) -> COORD {
        COORD {
            X: self.x,
            Y: self.y,
        }
    }
}