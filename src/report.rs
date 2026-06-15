use zerocopy::{FromBytes, Immutable, IntoBytes};

#[allow(dead_code)]
trait ToReport
where
    Self: Sized,
{
    fn to_report<const ID: u8>(self) -> Report<Self, ID> {
        Report { id: ID, base: self }
    }
}

#[repr(C, packed)]
#[allow(dead_code)]
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
