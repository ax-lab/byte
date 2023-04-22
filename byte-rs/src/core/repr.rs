use std::{
	fmt::*,
	sync::{Arc, Mutex},
};

pub trait HasRepr {
	fn output_repr(&self, output: &Repr);

	fn fmt_debug(&self, f: &mut Formatter<'_>) -> Result {
		let output = Repr::new(ReprMode::Debug, ReprFormat::Full);
		self.output_repr(&output);
		write!(f, "{output}")
	}

	fn fmt_display(&self, f: &mut Formatter<'_>) -> Result {
		let output = Repr::new(ReprMode::Display, ReprFormat::Full);
		self.output_repr(&output);
		write!(f, "{output}")
	}
}

#[macro_export]
macro_rules! fmt_from_repr {
	($t:path) => {
		impl ::std::fmt::Display for $t {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				crate::core::repr::HasRepr::fmt_display(self, f)
			}
		}

		impl ::std::fmt::Debug for $t {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				crate::core::repr::HasRepr::fmt_debug(self, f)
			}
		}
	};
}

#[macro_export]
macro_rules! repr_from_fmt {
	($t:ty) => {
		impl crate::core::repr::HasRepr for $t {
			fn output_repr(&self, output: &Repr) {
				if output.is_debug() {
					output.write(format!("{self:?}"));
				} else {
					output.write(format!("{self}"));
				}
			}
		}
	};
}

pub use fmt_from_repr;
pub use repr_from_fmt;

#[derive(Clone)]
pub struct Repr {
	mode: ReprMode,
	format: ReprFormat,
	buffer: Arc<Mutex<String>>,
}

impl Repr {
	pub fn new(mode: ReprMode, format: ReprFormat) -> Repr {
		Repr {
			mode,
			format,
			buffer: Default::default(),
		}
	}

	pub fn with(&self, mode: ReprMode, format: ReprFormat) -> Repr {
		let mut repr = self.clone();
		repr.mode = mode;
		repr.format = format;
		repr
	}

	pub fn mode(&self) -> ReprMode {
		self.mode
	}

	pub fn format(&self) -> ReprFormat {
		self.format
	}

	pub fn is_debug(&self) -> bool {
		self.mode() == ReprMode::Debug
	}

	pub fn write<T: AsRef<str>>(&self, output: T) {
		let mut buffer = self.buffer.lock().unwrap();
		buffer.push_str(output.as_ref());
	}
}

impl Display for Repr {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		let buffer = self.buffer.lock().unwrap();
		write!(f, "{buffer}")
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReprMode {
	Debug,
	Display,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ReprFormat {
	Minimal,
	Compact,
	Full,
}
