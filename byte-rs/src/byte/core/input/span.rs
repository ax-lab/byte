use std::io::Write;

use crate::core::repr::*;

use super::*;

/// Span indexes a range of text from an [`Input`].
#[derive(Clone, Default, Eq, PartialEq)]
pub struct Span {
	sta: Cursor,
	end: Cursor,
}

impl Span {
	pub fn from_range(a: Option<Span>, b: Option<Span>) -> Option<Span> {
		if a.is_none() && b.is_none() {
			None
		} else if let Some(a) = a {
			if let Some(b) = b {
				let (a, b) = if a.sta.offset() > b.sta.offset() {
					(b, a)
				} else {
					(a, b)
				};
				Some(Span::new(&a.sta, &b.sta))
			} else {
				Some(Span::new(&a.sta, &a.sta))
			}
		} else {
			Span::from_range(b, None)
		}
	}

	pub fn new(sta: &Cursor, end: &Cursor) -> Self {
		assert!(sta.input() == end.input());
		if sta.offset() > end.offset() {
			Span::new(end, sta)
		} else {
			Span {
				sta: sta.clone(),
				end: end.clone(),
			}
		}
	}

	pub fn input(&self) -> &Input {
		self.sta.input()
	}

	pub fn text(&self) -> &str {
		let sta = self.sta.offset();
		let end = self.end.offset();
		self.sta.input().text(sta..end)
	}

	pub fn start(&self) -> &Cursor {
		&self.sta
	}

	pub fn end(&self) -> &Cursor {
		&self.end
	}
}

fmt_from_repr!(Span);

impl HasRepr for Span {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let debug = output.is_debug();
		if debug {
			write!(output, "<span ")?;
		}

		if output.format() > ReprFormat::Compact {
			let input = self.input().name();
			if input != "" {
				write!(output, "{input}:")?;
			}
		}
		write!(output, "{}", self.sta)?;
		if self.end != self.sta && (!output.is_minimal() || output.is_debug()) {
			let _ = write!(output, "…");
			write!(output, "{}", self.end)?;
		}
		if debug {
			write!(output, ">")?;
		}
		Ok(())
	}
}
