use std::ops::Range;

#[derive(Debug)]
pub struct Error {
    pub(crate) message: String,
    pub(crate) range: Range<usize>,
}
