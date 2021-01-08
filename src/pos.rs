/// A position in the parsed data
pub trait Pos: Ord + Copy {
    /// The initial position
    fn zero() -> Self;
}

impl Pos for usize {
    #[inline]
    fn zero() -> Self {
        0
    }
}
