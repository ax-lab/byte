/// Global evaluation precedence for language nodes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Precedence {
	First,
	RawText,
	Comments,
	Parenthesis,
	Indentation,
	LineBreaks,
	Values,
	Last,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Sequence {
	Ordered,
	Reverse,
	SingleStep,
}