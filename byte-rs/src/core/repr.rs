use std::{
	fmt::*,
	sync::{Arc, Mutex},
};

pub trait HasRepr {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()>;

	fn fmt_debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut output = Repr::new(ReprMode::Debug, ReprFormat::Full);
		let _ = self.output_repr(&mut output);
		write!(f, "{output}")
	}

	fn fmt_display(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut output = Repr::new(ReprMode::Display, ReprFormat::Full);
		let _ = self.output_repr(&mut output);
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
			fn output_repr(&self, output: &mut crate::core::repr::Repr) -> ::std::io::Result<()> {
				use ::std::io::Write;
				if output.is_debug() {
					write!(output, "{self:?}")
				} else {
					write!(output, "{self}")
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
	buffer: Arc<Mutex<ReprBuffer>>,
	indent: Arc<String>,
}

#[derive(Default)]
struct ReprBuffer {
	new_line: bool,
	data: String,
}

impl Repr {
	pub fn new(mode: ReprMode, format: ReprFormat) -> Repr {
		Repr {
			mode,
			format,
			buffer: Default::default(),
			indent: Default::default(),
		}
	}

	pub fn indented_by<T: AsRef<str>>(&self, new_indent: T) -> Repr {
		let mut repr = self.clone();
		let indent = Arc::make_mut(&mut repr.indent);
		indent.push_str(new_indent.as_ref());
		repr
	}

	pub fn indented(&self) -> Repr {
		self.indented_by("    ")
	}

	pub fn with(&self, mode: ReprMode, format: ReprFormat) -> Repr {
		let mut repr = self.clone();
		repr.mode = mode;
		repr.format = format;
		repr
	}

	pub fn display(&self) -> Repr {
		self.with(ReprMode::Display, self.format)
	}

	pub fn compact(&self) -> Repr {
		self.with(self.mode, ReprFormat::Compact)
	}

	pub fn minimal(&self) -> Repr {
		self.with(self.mode, ReprFormat::Minimal)
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
}

impl std::io::Write for Repr {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let mut str = unsafe { std::str::from_utf8_unchecked(buf) };
		let mut buffer = self.buffer.lock().unwrap();
		while let Some(index) = str.find(|c| c == '\r' || c == '\n') {
			let buf = str.as_bytes();
			let index = if buf[index] == '\r' as u8
				&& index < buf.len() - 1
				&& buf[index + 1] == '\n' as u8
			{
				index + 2
			} else {
				index + 1
			};

			let chunk = &str[..index];
			if buffer.new_line || buffer.data.len() == 0 {
				buffer.data.push_str(self.indent.as_str());
			}
			buffer.data.push_str(chunk);
			buffer.new_line = true;
			str = &str[index..];
		}

		if str.len() > 0 {
			if buffer.new_line || buffer.data.len() == 0 {
				buffer.data.push_str(self.indent.as_str());
			}
			buffer.data.push_str(str);
			buffer.new_line = false;
		}

		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

impl Display for Repr {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		let buffer = self.buffer.lock().unwrap();
		write!(f, "{}", buffer.data)
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
