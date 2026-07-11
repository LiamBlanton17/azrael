use std::ops::Not;

// Enum for color
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Color {
    #[default] White,
    Black,
}

impl Color {
    #[inline]
    pub fn idx(&self) -> usize {
        *self as usize
    }

    #[inline]
    pub fn flip(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

}

impl Not for Color {
    type Output = Color;
    
    #[inline]
    fn not(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
