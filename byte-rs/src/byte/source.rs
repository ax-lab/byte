use std::{ops::Range, path::Path};

use super::*;

//====================================================================================================================//
// Source
//====================================================================================================================//

/// Abstracts an input source for the language.
pub trait IsSource: IsValue {
	/// Name for the source. Used as display in messages.
	fn name(&self) -> &str;

	/// If the source is a file, this is the file absolute path.
	fn path(&self) -> Option<&Path>;

	/// Load the contents of this source as a [`Node`] set.
	fn load(&self) -> Result<NodeSet>;

	/// Returns the raw source text for a given range, if available.
	///
	/// Not all sources are text-based, so this may not be provided. Passing
	/// an out-of-bounds range will also result in [`None`].
	fn get_text(&self, range: Range<usize>) -> Option<&str>;

	/// Tab-size for this source or zero to use the default.
	///
	/// See [`DEFAULT_TAB_SIZE`] for details.
	fn tab_size(&self) -> usize {
		0
	}
}

#[derive(Clone)]
pub struct Source(Arc<dyn IsSource>);

impl Source {
	pub fn as_ref(&self) -> &dyn IsSource {
		self.0.as_ref()
	}
}

impl Deref for Source {
	type Target = dyn IsSource;

	fn deref(&self) -> &Self::Target {
		self.as_ref()
	}
}

impl Display for Source {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}

impl Debug for Source {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "<{} ({})>", self.name(), self.as_ref().type_name())
	}
}
