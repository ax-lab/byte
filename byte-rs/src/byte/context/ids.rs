use super::*;

impl Context {
	/// Return a new globally unique [`Id`].
	///
	/// Besides its use as a unique identifier the [`Id`] can also be used to
	/// store associated data in the current [`Context`].
	///
	/// The [`Id`] value is an incrementing non-zero integer.
	pub fn id() -> Id {
		use std::sync::atomic::*;
		static COUNTER: AtomicUsize = AtomicUsize::new(1);
		let id = COUNTER.fetch_add(1, Ordering::SeqCst);
		Id(id)
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id(usize);

impl Id {
	/// Integer value for this id.
	pub fn value(&self) -> usize {
		self.0
	}
}

impl Debug for Id {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "#{}", self.value())
	}
}

impl Display for Id {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "#{}", self.value())
	}
}
