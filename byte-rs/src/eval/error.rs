#[derive(Clone, Debug)]
pub enum Error {
	InvalidNode,
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::InvalidNode => write!(f, "invalid node"),
		}
	}
}
