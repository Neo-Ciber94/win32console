use std::fmt::{Display, Formatter, Error};
use std::ops::Div;
use std::ops::Mul;
use winapi::um::wincon::SMALL_RECT;

/// Represents a [https://docs.microsoft.com/en-us/windows/console/small-rect-str].
///
/// link: [https://docs.microsoft.com/en-us/windows/console/small-rect-str]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Hash)]
pub struct SmallRect {
    pub left: i16,
    pub top: i16,
    pub right: i16,
    pub bottom: i16,
}

impl SmallRect {
    /// Creates a new `SmallRect`.
    pub fn new(left: i16, top: i16, right: i16, bottom: i16) -> Self{
        SmallRect{ left, top, right, bottom}
    }

    /// Creates a `SmallRect` from this instance with a new `left` value.
    pub fn with_left(&self, left: i16) -> Self{
        SmallRect{
            left,
            top: self.top,
            right: self.right,
            bottom: self.bottom
        }
    }

    /// Creates a `SmallRect` from this instance with a new `top` value.
    pub fn with_top(&self, top: i16) -> Self{
        SmallRect{
            left: self.left,
            top,
            right: self.right,
            bottom: self.bottom
        }
    }

    /// Creates a `SmallRect` from this instance with a new `right` value.
    pub fn with_right(&self, right: i16) -> Self{
        SmallRect{
            left: self.left,
            top: self.top,
            right,
            bottom: self.bottom
        }
    }

    /// Creates a `SmallRect` from this instance with a new `bottom` value.
    pub fn with_bottom(&self, bottom: i16) -> Self{
        SmallRect{
            left: self.left,
            top: self.top,
            right: self.right,
            bottom
        }
    }
}

impl Display for SmallRect{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_fmt(format_args!("[Left: {}, Top: {}, Right: {}, Bottom: {}]", self.left, self.top, self.right, self.bottom))
    }
}

impl From<SMALL_RECT> for SmallRect {
    fn from(rect: SMALL_RECT) -> Self {
        SmallRect {
            left: rect.Left,
            top: rect.Top,
            right: rect.Right,
            bottom: rect.Bottom,
        }
    }
}

impl Into<SMALL_RECT> for SmallRect {
    fn into(self) -> SMALL_RECT {
        SMALL_RECT {
            Left: self.left,
            Top: self.top,
            Right: self.right,
            Bottom: self.bottom,
        }
    }
}

impl Div<i16> for SmallRect{
    type Output = SmallRect;

    fn div(self, rhs: i16) -> Self::Output {
       SmallRect{
           left: self.left / rhs,
           top: self.top / rhs,
           right: self.right / rhs,
           bottom: self.bottom / rhs,
       }
    }
}

impl Mul<i16> for SmallRect{
    type Output = SmallRect;

    fn mul(self, rhs: i16) -> Self::Output {
        SmallRect{
            left: self.left * rhs,
            top: self.top * rhs,
            right: self.right * rhs,
            bottom: self.bottom * rhs,
        }
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn small_rect_mul_test(){
        let a = SmallRect::new(1, 2, 3, 4);
        assert_eq!(SmallRect::new(2, 4, 6, 8), a * 2);
    }

    #[test]
    fn small_rect_div_test(){
        let a = SmallRect::new(2, 4, 6, 8);
        assert_eq!(SmallRect::new(1, 2, 3, 4), a / 2);
    }
}