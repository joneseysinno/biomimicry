//! The four DNA primitives that compose every gene cistron.
//!
//! Stable `type_id` values `0..=3` match the infinite-db `Space("primitives")`
//! node ids and must not be renumbered — content hashes depend on them.

/// One of the four primitives that appear as poles of a gene cistron.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u32)]
pub enum Primitive {
    /// Emit outward / absorb before propagation.
    Signal = 0,
    /// Open to match an incoming signal / blocked to mismatch one.
    Receptor = 1,
    /// Activate / suppress gene(s).
    Expression = 2,
    /// Produce / inhibit or reverse output.
    Transduction = 3,
}

impl Primitive {
    /// Stable numeric id for content addressing (`0..=3`).
    #[must_use]
    pub const fn type_id(self) -> u32 {
        self as u32
    }

    /// Reconstruct from a stable type id.
    #[must_use]
    pub const fn from_type_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::Signal),
            1 => Some(Self::Receptor),
            2 => Some(Self::Expression),
            3 => Some(Self::Transduction),
            _ => None,
        }
    }

    /// All primitives in type-id order.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::Signal,
            Self::Receptor,
            Self::Expression,
            Self::Transduction,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_ids_are_stable_0_to_3() {
        assert_eq!(Primitive::Signal.type_id(), 0);
        assert_eq!(Primitive::Receptor.type_id(), 1);
        assert_eq!(Primitive::Expression.type_id(), 2);
        assert_eq!(Primitive::Transduction.type_id(), 3);
        for p in Primitive::all() {
            assert_eq!(Primitive::from_type_id(p.type_id()), Some(p));
        }
    }
}
