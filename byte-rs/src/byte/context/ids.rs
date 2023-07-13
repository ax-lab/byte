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

	/// Associate a source [`Span`] with this id.
	pub fn at(self, span: Span) -> Self {
		self.set_span(span);
		self
	}

	/// Source [`Span`] for this id.
	pub fn span(&self) -> Span {
		Context::get().read(|data| {
			let span_map = data.ids.span_map.read().unwrap();
			span_map.get(self).cloned().unwrap_or_default()
		})
	}

	pub fn set_span(&self, span: Span) {
		Context::get().read(|data| {
			let mut span_map = data.ids.span_map.write().unwrap();
			span_map.insert(self.clone(), span);
		});
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

//====================================================================================================================//
// Internals
//====================================================================================================================//

#[derive(Default, Clone)]
pub(super) struct ContextIds {
	span_map: Arc<RwLock<HashMap<Id, Span>>>,
}
