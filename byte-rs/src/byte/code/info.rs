use super::*;

/// Common information for an [`Expr`] value, such as its unique [`Id`]
/// and source [`Span`].
#[derive(Clone)]
pub struct Info {
	id: Id,
	span: Span,
	solving: Arc<RwLock<bool>>,
}

impl Info {
	pub fn none() -> Self {
		Self {
			id: id(),
			span: Span::default(),
			solving: Default::default(),
		}
	}

	pub fn new(span: Span) -> Self {
		Self {
			id: id(),
			span,
			solving: Default::default(),
		}
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn span(&self) -> &Span {
		&self.span
	}

	/// Used internally to flag a [`Expr::Node`] being solved and prevent
	/// cycles.
	pub(crate) fn solve(&self) -> bool {
		let mut solving = self.solving.write().unwrap();
		if *solving {
			false
		} else {
			*solving = true;
			true
		}
	}
}

impl PartialEq for Info {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Eq for Info {}

impl Hash for Info {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state)
	}
}

impl Debug for Info {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.id)
	}
}
