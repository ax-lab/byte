use std::{
	fmt::Formatter,
	io::Write,
	sync::{Arc, Mutex},
};

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

pub trait HasRepr {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()>;

	fn fmt_debug(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut output = Vec::<u8>::new();
		let mut writer = Repr::new(&mut output, ReprMode::Debug, ReprFormat::Full);
		let _ = self.output_repr(&mut writer);
		f.write_str(unsafe { std::str::from_utf8_unchecked(&output) })
	}

	fn fmt_display(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut output = Vec::<u8>::new();
		let mut writer = Repr::new(&mut output, ReprMode::Display, ReprFormat::Compact);
		let _ = self.output_repr(&mut writer);
		f.write_str(unsafe { std::str::from_utf8_unchecked(&output) })
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

//====================================================================================================================//
// HasRepr for default types
//====================================================================================================================//

repr_from_fmt!(String);
repr_from_fmt!(bool);
repr_from_fmt!(i8);
repr_from_fmt!(i16);
repr_from_fmt!(i32);
repr_from_fmt!(i64);
repr_from_fmt!(i128);
repr_from_fmt!(isize);
repr_from_fmt!(u8);
repr_from_fmt!(u16);
repr_from_fmt!(u32);
repr_from_fmt!(u64);
repr_from_fmt!(u128);
repr_from_fmt!(usize);
repr_from_fmt!(f32);
repr_from_fmt!(f64);
repr_from_fmt!(&str);

impl HasRepr for () {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "{self:?}")
		} else if output.is_full() {
			write!(output, "(none)")
		} else {
			write!(output, "")
		}
	}
}

impl<T: HasRepr> HasRepr for [T] {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if output.is_compact() {
			write!(output, "[")?;
			for (i, it) in self.iter().enumerate() {
				if i > 0 {
					write!(output, ", ")?;
				} else {
					write!(output, " ")?;
				}
				it.output_repr(output)?;
			}
			write!(output, " ]")?;
		} else {
			let mut empty = true;
			write!(output, "[")?;
			{
				let mut output = output.indented().compact();
				for it in self.iter() {
					empty = false;
					write!(output, "\n")?;
					it.output_repr(&mut output)?;
				}
			}
			if !empty {
				write!(output, "\n")?;
			}
			write!(output, "]")?;
		}
		Ok(())
	}
}

impl<T: HasRepr> HasRepr for Vec<T> {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		self.as_slice().output_repr(output)
	}
}

//====================================================================================================================//
// Repr implementation
//====================================================================================================================//

pub use fmt_from_repr;
pub use repr_from_fmt;

#[derive(Clone)]
pub struct Repr<'a> {
	mode: ReprMode,
	format: ReprFormat,
	buffer: Arc<Mutex<ReprBuffer<'a>>>,
	indent: Arc<String>,
}

struct ReprBuffer<'a> {
	new_line: bool,
	output: &'a mut dyn Write,
}

impl<'a> Repr<'a> {
	pub fn new(output: &'a mut dyn Write, mode: ReprMode, format: ReprFormat) -> Repr<'a> {
		Repr {
			mode,
			format,
			buffer: Arc::new(Mutex::new(ReprBuffer {
				new_line: true,
				output,
			})),
			indent: Default::default(),
		}
	}

	pub fn dump_list<T: HasRepr, I: IntoIterator<Item = T>>(output: &'a mut dyn Write, items: I) {
		let mut repr = Repr::new(output, ReprMode::Debug, ReprFormat::Full).indented();
		for (i, it) in items.into_iter().enumerate() {
			if i > 0 {
				let _ = write!(repr, "\n");
			}
			let _ = write!(repr, "[{i}] ");
			let output = &mut repr.indented();
			let _ = it.output_repr(output);
		}
	}

	pub fn string<T: HasRepr>(value: &T, mode: ReprMode, format: ReprFormat) -> String {
		let mut output = Vec::<u8>::new();
		let mut writer = Repr::new(&mut output, mode, format);
		let _ = value.output_repr(&mut writer);
		unsafe { String::from_utf8_unchecked(output) }
	}

	pub fn indented_by<T: AsRef<str>>(&self, new_indent: T) -> Self {
		let mut repr = self.clone();
		let indent = Arc::make_mut(&mut repr.indent);
		indent.push_str(new_indent.as_ref());
		repr
	}

	pub fn indented(&self) -> Self {
		self.indented_by("    ")
	}

	pub fn with(&self, mode: ReprMode, format: ReprFormat) -> Self {
		let mut repr = self.clone();
		repr.mode = mode;
		repr.format = format;
		repr
	}

	pub fn display(&self) -> Self {
		self.with(ReprMode::Display, self.format)
	}

	pub fn debug(&self) -> Self {
		self.with(ReprMode::Debug, self.format)
	}

	pub fn compact(&self) -> Self {
		self.with(self.mode, ReprFormat::Compact)
	}

	pub fn minimal(&self) -> Self {
		self.with(self.mode, ReprFormat::Minimal)
	}

	pub fn full(&self) -> Self {
		self.with(self.mode, ReprFormat::Full)
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

	pub fn is_display(&self) -> bool {
		self.mode() == ReprMode::Display
	}

	pub fn is_compact(&self) -> bool {
		!self.is_full()
	}

	pub fn is_minimal(&self) -> bool {
		self.format == ReprFormat::Minimal
	}

	pub fn is_full(&self) -> bool {
		self.format() == ReprFormat::Full
	}
}

impl<'a> Write for Repr<'a> {
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
			if buffer.new_line {
				write!(buffer.output, "{}", self.indent.as_str())?;
			}
			write!(buffer.output, "{}", chunk)?;
			buffer.new_line = true;
			str = &str[index..];
		}

		if str.len() > 0 {
			if buffer.new_line {
				write!(buffer.output, "{}", self.indent.as_str())?;
			}
			write!(buffer.output, "{}", str)?;
			buffer.new_line = false;
		}

		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn compact_repr() {
		let value = vec![vec![1], vec![1, 2], vec![1, 2, 3]];
		let value = Repr::string(&value, ReprMode::Display, ReprFormat::Compact);
		assert_eq!(value, "[ [ 1 ], [ 1, 2 ], [ 1, 2, 3 ] ]")
	}

	#[test]
	fn full_repr() {
		let value = vec![vec![1], vec![1, 2], vec![1, 2, 3]];
		let value = Repr::string(&value, ReprMode::Display, ReprFormat::Full);
		assert_eq!(value, "[\n    [ 1 ]\n    [ 1, 2 ]\n    [ 1, 2, 3 ]\n]")
	}
}
