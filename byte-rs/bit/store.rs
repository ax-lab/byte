use super::SymbolSet;

/// Provides storage for compilation data.
///
/// This provides [`Arena`] semantics: values are never deallocated and remain
/// valid for the lifetime of the [`Store`].
#[derive(Default)]
pub struct Store {
	symbols: SymbolSet,
}

impl Store {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn symbols(&self) -> &SymbolSet {
		&self.symbols
	}
}
