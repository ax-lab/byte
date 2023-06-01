use std::{fmt::*, sync::Arc};

use super::*;

//====================================================================================================================//
// Debug & Format
//====================================================================================================================//

pub trait WithDebug {
	fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<T: IsValue + std::fmt::Debug> WithDebug for T {
	fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.fmt(f)
	}
}

pub trait WithDisplay {
	fn fmt_display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<T: IsValue + std::fmt::Display> WithDisplay for T {
	fn fmt_display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.fmt(f)
	}
}

impl Value {
	pub fn with_debug(&self) -> Option<&dyn WithDebug> {
		get_trait!(self, WithDebug)
	}

	pub fn with_display(&self) -> Option<&dyn WithDisplay> {
		get_trait!(self, WithDisplay)
	}
}

impl std::fmt::Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if let Some(value) = self.with_repr() {
			value.fmt_debug(f)
		} else if let Some(value) = self.with_debug() {
			value.fmt_debug(f)
		} else {
			let ptr = Arc::as_ptr(self.inner());
			write!(f, "Value({}: {ptr:?})", self.type_name())
		}
	}
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if let Some(value) = self.with_repr() {
			value.fmt_display(f)
		} else if let Some(value) = self.with_display() {
			value.fmt_display(f)
		} else {
			write!(f, "Value({})", self.type_name())
		}
	}
}

//====================================================================================================================//
// Repr
//====================================================================================================================//

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReprMode {
	Debug,
	Display,
}

impl ReprMode {
	pub fn is_debug(&self) -> bool {
		self == &ReprMode::Debug
	}

	pub fn is_display(&self) -> bool {
		self == &ReprMode::Display
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ReprFormat {
	Word,
	Line,
	Full,
}

pub trait WithRepr {
	fn output(&self, mode: ReprMode, format: ReprFormat, output: &mut dyn Write) -> Result;

	fn fmt_debug(&self, output: &mut dyn Write) -> Result {
		self.output(ReprMode::Debug, ReprFormat::Line, output)
	}

	fn fmt_display(&self, output: &mut dyn Write) -> Result {
		self.output(ReprMode::Display, ReprFormat::Line, output)
	}
}

impl Value {
	pub fn with_repr(&self) -> Option<&dyn WithRepr> {
		get_trait!(self, WithRepr)
	}

	pub fn output(&self, mode: ReprMode, format: ReprFormat, output: &mut dyn Write) -> Result {
		if let Some(value) = self.with_repr() {
			value.output(mode, format, output)
		} else if mode == ReprMode::Debug {
			write!(output, "{self:?}")
		} else {
			write!(output, "{self}")
		}
	}
}

//====================================================================================================================//
// Indented output
//====================================================================================================================//

pub trait WithIndent {
	fn indented(&mut self) -> IndentedFormatter;

	fn indented_with(&mut self, indent: &'static str) -> IndentedFormatter {
		let mut indented = self.indented();
		indented.indent = indent;
		indented
	}
}

impl<'a, T: Write> WithIndent for T {
	fn indented(&mut self) -> IndentedFormatter {
		IndentedFormatter::new(self)
	}
}

pub struct IndentedFormatter<'a> {
	indent: &'static str,
	prefix: &'static str,
	inner: &'a mut dyn Write,
}

impl<'a> IndentedFormatter<'a> {
	fn new(f: &'a mut dyn Write) -> Self {
		Self {
			indent: "    ",
			prefix: "",
			inner: f,
		}
	}
}

impl<'a> std::fmt::Write for IndentedFormatter<'a> {
	fn write_str(&mut self, s: &str) -> Result {
		let mut str = s;
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
			self.inner.write_str(self.prefix)?;
			self.prefix = self.indent;

			self.inner.write_str(chunk)?;
			str = &str[index..];
		}

		if str.len() > 0 {
			self.inner.write_str(self.prefix)?;
			self.inner.write_str(str)?;
			self.prefix = "";
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use std::fmt::*;

	use super::*;

	#[test]
	fn test_indent() {
		let expected = vec![
			"Object {",
			"    A = [",
			"        <item 1>",
			"        <item 2>",
			"        <item 3 but",
			"        …with multiple lines>",
			"    ]",
			"    B = [",
			"    ]",
			"    C = [",
			"        <C1>",
			"        <C2>",
			"    ]",
			"}",
		];

		let value = Obj(vec![
			Key(
				"A",
				List(vec!["item 1", "item 2", "item 3 but\nwith multiple lines"]),
			),
			Key("B", List(vec![])),
			Key("C", List(vec!["C1", "C2"])),
		]);

		let expected = expected.join("\n");
		assert_eq!(format!("{value}"), expected);
	}

	pub struct List(Vec<&'static str>);

	pub struct Key(&'static str, List);

	pub struct Obj(Vec<Key>);

	impl Display for List {
		fn fmt(&self, f: &mut Formatter) -> Result {
			{
				let mut f = f.indented();
				write!(f, "[")?;
				for it in self.0.iter() {
					write!(f, "\n")?;
					write!(f.indented_with("…"), "<{it}>")?;
				}
			}
			write!(f, "\n]")
		}
	}

	impl Display for Key {
		fn fmt(&self, f: &mut Formatter) -> Result {
			write!(f, "{} = {}", self.0, self.1)
		}
	}

	impl Display for Obj {
		fn fmt(&self, f: &mut Formatter) -> Result {
			{
				let mut f = f.indented();
				write!(f, "Object {{")?;
				for it in self.0.iter() {
					write!(f, "\n{it}")?;
				}
			}
			write!(f, "\n}}")
		}
	}
}
