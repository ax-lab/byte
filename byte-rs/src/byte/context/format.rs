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
}

impl Format {
	pub fn new(mode: Mode) -> Self {
		Self { mode, line_width: 0 }
	}

	pub fn with_line_width(mut self, line_width: usize) -> Self {
		self.line_width = line_width;
		self
	}

	pub fn mode(&self) -> Mode {
		self.mode
	}

	pub fn line_width(&self) -> usize {
		if self.line_width == 0 {
			DEFAULT_FORMAT_LINE_WIDTH
		} else {
			std::cmp::max(self.line_width, 16)
		}
	}
}

impl Context {
	pub fn get_format(&self) -> Format {
		self.read(|ctx| ctx.format.config.clone())
	}
	pub fn with_format<T, P: FnOnce() -> T>(&self, format: Format, run: P) -> T {
		self.clone().write(|ctx| ctx.set_format(format)).with(|_| run())
	}
}

impl<'a> ContextWriter<'a> {
	pub fn set_format(&mut self, format: Format) -> Format {
		self.write(|ctx| std::mem::replace(&mut ctx.format.config, format))
	}
}

#[derive(Default, Clone)]
pub(super) struct ContextDataFormat {
	config: Format,
}
