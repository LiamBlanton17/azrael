
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
}
