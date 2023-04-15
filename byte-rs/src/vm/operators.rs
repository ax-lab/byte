#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Op {
	Unary(OpUnary),
	Binary(OpBinary),
	Ternary(OpTernary),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OpUnary {
	Not,
	Plus,
	Minus,
	Negate,
	PreIncrement,
	PreDecrement,
	PosIncrement,
	PosDecrement,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OpBinary {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	Assign,
	Equal,
	And,
	Or,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OpTernary {
	Conditional,
}
