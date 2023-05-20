pub struct Error {
	message: String,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.message)
	}
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<error: {}>", self.message)
	}
}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self {
			message: format!("io error: {value}"),
		}
	}
}

impl From<std::fmt::Error> for Error {
	fn from(value: std::fmt::Error) -> Self {
		Self {
			message: format!("{value}"),
		}
	}
}

impl From<&str> for Error {
	fn from(value: &str) -> Self {
		Self {
			message: format!("{value}"),
		}
	}
}

impl From<String> for Error {
	fn from(value: String) -> Self {
		Self { message: value }
	}
}

impl From<Error> for std::fmt::Error {
	fn from(_: Error) -> Self {
		std::fmt::Error
	}
}
