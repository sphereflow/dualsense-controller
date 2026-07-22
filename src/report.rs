use zerocopy::{FromBytes, Immutable, IntoBytes};

#[repr(C, packed)]
#[derive(FromBytes, IntoBytes, Immutable, Debug, Clone, Copy, Default)]
pub struct Report<T, const ID: u8> {
    id: u8,
    pub base: T,
}

impl<T, const ID: u8> Report<T, ID> {
    pub fn new(base: T) -> Report<T, ID> {
        Report { id: ID, base }
    }
}
