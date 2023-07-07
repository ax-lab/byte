use super::*;

//====================================================================================================================//
// Debug & Format
//====================================================================================================================//

/// Dynamic trait implemented automatically for any value with [`Debug`].
pub trait WithDebug {
	fn fmt_debug(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result;
}

impl<T: std::fmt::Debug> WithDebug for T {
	fn fmt_debug(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}

/// Dynamic trait implemented automatically for any value with [`Display`].
pub trait WithDisplay {
	fn fmt_display(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result;
}

impl<T: std::fmt::Display> WithDisplay for T {
	fn fmt_display(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
		write!(f, "{self}")
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

//====================================================================================================================//
// Format mixin
//====================================================================================================================//

trait MixinFormattedOutput {
	fn output(&self, mode: ReprMode, output: &mut dyn std::fmt::Write) -> std::fmt::Result;

	fn fmt_debug(&self, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		self.output(ReprMode::Debug, output)
	}

	fn fmt_display(&self, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		self.output(ReprMode::Display, output)
	}
}

impl<T: ?Sized + HasTraits> MixinFormattedOutput for T {
	fn output(&self, mode: ReprMode, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		if mode == ReprMode::Debug {
			if let Some(value) = get_trait!(self, WithDebug) {
				value.fmt_debug(output)
			} else {
				let name = std::any::type_name::<Self>();
				let ptr = self as *const T;
				write!(output, "<{name}: {ptr:?}>")
			}
		} else {
			if let Some(value) = get_trait!(self, WithDisplay) {
				value.fmt_display(output)
			} else {
				let name = std::any::type_name::<Self>();
				write!(output, "({name})")
			}
		}
	}
}

//====================================================================================================================//
// Indented output
//====================================================================================================================//

/// Supports indented output for a [`Formatter`] or [`IndentedFormatter`].
pub trait WithIndent {
	fn indented(&mut self) -> IndentedFormatter;

	fn indented_with(&mut self, indent: &'static str) -> IndentedFormatter {
		let mut indented = self.indented();
		indented.indent = indent;
		indented
	}
}

impl<'a, T: std::fmt::Write> WithIndent for T {
	fn indented(&mut self) -> IndentedFormatter {
		IndentedFormatter::new(self)
	}
}

/// Support for indented output for a [`Formatter`].
pub struct IndentedFormatter<'a> {
	indent: &'a str,
	prefix: &'a str,
	inner: &'a mut dyn std::fmt::Write,
}

impl<'a> IndentedFormatter<'a> {
	pub(crate) fn new(f: &'a mut dyn std::fmt::Write) -> Self {
		Self {
			indent: "    ",
			prefix: "",
			inner: f,
		}
	}
}

pub fn fmt_indented<T: Display>(value: &T, prefix: &str, indent: &str) -> String {
	let mut output = String::from(prefix);
	{
		let mut output = IndentedFormatter {
			indent,
			prefix: prefix,
			inner: &mut output,
		};
		let _ = write!(output, "{value}");
	}
	output
}

pub fn fmt_indented_debug<T: Debug>(value: &T, prefix: &str, indent: &str) -> String {
	let mut output = String::from(prefix);
	{
		let mut output = IndentedFormatter {
			indent,
			prefix: prefix,
			inner: &mut output,
		};
		let _ = write!(output, "{value:?}");
	}
	output
}

impl<'a> std::fmt::Write for IndentedFormatter<'a> {
	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		let mut str = s;
		while let Some(index) = str.find(|c| c == '\r' || c == '\n') {
			let buf = str.as_bytes();
			let index = if buf[index] == '\r' as u8 && index < buf.len() - 1 && buf[index + 1] == '\n' as u8 {
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
	use std::fmt::{Display, Result};

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
			Key("A", List(vec!["item 1", "item 2", "item 3 but\nwith multiple lines"])),
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
