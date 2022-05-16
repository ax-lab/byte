use byte_core as byte;

use byte::lexer;
use byte::tokens::*;

macro_rules! span {
	($name:literal at $line:literal : $column:literal offset $offset:literal) => {{
		let pos = byte::tokens::Pos {
			line: $line,
			column: $column,
			offset: $offset,
		};
		byte::tokens::Span {
			filename: $name.to_string(),
			start: pos,
			end: pos,
		}
	}};
}

#[test]
fn should_parse_empty_input() {
	let list = lexer::parse_string("empty.by", "");
	let list = list.collect::<Vec<_>>();
	assert_eq!(list, vec![Token::End(span!("empty.by" at 0:0 offset 0))])
}
