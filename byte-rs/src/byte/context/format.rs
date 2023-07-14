use super::*;

const DEFAULT_FORMAT_LINE_WIDTH: usize = 120;

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Mode {
	Minimal,
	#[default]
	Normal,
	Detail,
}

#[derive(Default, Clone)]
pub struct Format {
	mode: Mode,
	line_width: usize,
	separator: String,
	nested: bool,
	disable_span: bool,
}

impl Format {
	pub fn mode(&self) -> Mode {
		self.mode
	}

	pub fn with_mode(mut self, mode: Mode) -> Self {
		self.mode = mode;
		self
	}

	pub fn nested(&self) -> bool {
		self.nested
	}

	pub fn as_nested(mut self) -> Format {
		self.nested = true;
		self
	}

	pub fn line_width(&self) -> usize {
		if self.line_width == 0 {
			DEFAULT_FORMAT_LINE_WIDTH
		} else {
			std::cmp::max(self.line_width, 16)
		}
	}

	pub fn separator(&self) -> &str {
		self.separator.as_str()
	}

	pub fn show_span(&self) -> bool {
		!self.disable_span
	}

	pub fn with_separator<T: Into<String>>(&self, separator: T) -> Self {
		let mut output = self.clone();
		output.separator = separator.into();
		output
	}

	pub fn with_line_width(&self, line_width: usize) -> Self {
		let mut output = self.clone();
		output.line_width = line_width;
		output
	}

	pub fn with_span(&self) -> Self {
		let mut output = self.clone();
		output.disable_span = false;
		output
	}

	pub fn without_span(&self) -> Self {
		let mut output = self.clone();
		output.disable_span = true;
		output
	}
}

impl Context {
	pub fn format(&self) -> Format {
		self.read(|ctx| ctx.format.config.clone())
	}

	pub fn with_format<T, P: FnOnce() -> T>(&self, format: Format, run: P) -> T {
		self.clone().write(|ctx| ctx.set_format(format)).with(|_| run())
	}

	pub fn and_format(self, format: Format) -> Self {
		self.write(|ctx| ctx.set_format(format))
	}

	pub fn format_without_span(self) -> Self {
		let fmt = self.format();
		self.and_format(fmt.without_span())
	}

	pub fn format_with_span(self) -> Self {
		let fmt = self.format();
		self.and_format(fmt.with_span())
	}
}

impl<'a> ContextWriter<'a> {
	pub fn set_format(&mut self, format: Format) -> Format {
		self.write(|ctx| std::mem::replace(&mut ctx.format.config, format))
	}
}

#[derive(Default, Clone)]
pub(super) struct ContextFormat {
	config: Format,
}
