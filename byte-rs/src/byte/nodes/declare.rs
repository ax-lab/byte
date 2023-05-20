use super::*;

#[derive(Eq, PartialEq)]
pub struct Declare {}

has_traits!(Declare: IsNode);

impl IsNode for Declare {}

impl Declare {
	pub fn parse(stream: &mut NodeStream, errors: &mut Errors) -> Option<Self> {
		let parsed = stream
			.read_map_symbol(|symbol| {
				Some(match symbol {
					"const" => DeclareKind::Const,
					"static" => DeclareKind::Static,
					_ => return None,
				})
			})
			.and_then(|kind| stream.read_id().map(|id| (kind, id)));
		if let Some((kind, id)) = parsed {
			let _ = (kind, id);
			if !stream.read_symbol("=") {
				errors.add("expected `=`".maybe_at_node(stream.peek()));
				return None;
			}

			let _body = stream.to_list();
			todo!()
		} else {
			None
		}
	}
}

impl HasRepr for Declare {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		write!(output, "<Declare>")
	}
}

pub enum DeclareKind {
	Const,
	Static,
}

pub struct DeclareMacro {}

impl SyntaxMacro for DeclareMacro {
	fn parse(&self, stream: &mut NodeStream, errors: &mut Errors) -> Option<Node> {
		Declare::parse(stream, errors).map(|x| x.into())
	}

	fn valid_symbols(&self) -> Option<&[&'static str]> {
		Some(&["const", "static"])
	}
}
