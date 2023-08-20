use super::*;

/// Represents an offset value derived from the source code location and used
/// for scoping purposes.
///
/// Each [`Node`] contains offset information, which is then used for name
/// binding and resolution.
///
/// Within a [`Scope`], a symbol or variable declared with [`CodeOffset::At`]
/// is only visible from that offset forward.
///
/// Symbols declared as [`CodeOffset::Static`] are visible anywhere within
/// their given scope.
///
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CodeOffset {
	Static,
	At(usize),
}

impl CodeOffset {
	pub fn value(&self) -> usize {
		match self {
			CodeOffset::Static => 0,
			CodeOffset::At(offset) => *offset,
		}
	}
}

impl Display for CodeOffset {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			CodeOffset::Static => write!(f, "static scope"),
			CodeOffset::At(offset) => write!(f, "offset {offset}"),
		}
	}
}

impl Debug for CodeOffset {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, ":{}", self.value())
	}
}
