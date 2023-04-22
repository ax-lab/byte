use std::cmp::Ordering;

use crate::vm::operators::*;

impl OpUnary {
	pub fn get_prefix(symbol: &str) -> Option<OpUnary> {
		let op = match symbol {
			"not" => OpUnary::Not,
			"+" => OpUnary::Plus,
			"-" => OpUnary::Minus,
			"!" => OpUnary::Negate,
			"++" => OpUnary::PreIncrement,
			"--" => OpUnary::PreDecrement,
			_ => return None,
		};
		Some(op)
	}

	pub fn get_posfix(symbol: &str) -> Option<OpUnary> {
		let op = match symbol {
			"++" => OpUnary::PosIncrement,
			"--" => OpUnary::PosDecrement,
			_ => return None,
		};
		Some(op)
	}

	pub fn is_posfix(&self) -> bool {
		match self {
			OpUnary::PosIncrement | OpUnary::PosDecrement => true,
			_ => false,
		}
	}
}

impl std::fmt::Display for OpUnary {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let symbol = match self {
			OpUnary::Not => "not",
			OpUnary::Plus => "+",
			OpUnary::Minus => "-",
			OpUnary::Negate => "!",
			OpUnary::PreIncrement => "++",
			OpUnary::PreDecrement => "--",
			OpUnary::PosIncrement => "++",
			OpUnary::PosDecrement => "--",
		};
		write!(f, "{symbol}")
	}
}

impl OpBinary {
	pub fn get(token: &str) -> Option<OpBinary> {
		let op = match token {
			"*" => OpBinary::Mul,
			"/" => OpBinary::Div,
			"%" => OpBinary::Mod,
			"+" => OpBinary::Add,
			"-" => OpBinary::Sub,
			"=" => OpBinary::Assign,
			"==" => OpBinary::Equal,
			"and" => OpBinary::And,
			"or" => OpBinary::Or,
			_ => return None,
		};
		Some(op)
	}
}

impl std::fmt::Display for OpBinary {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let symbol = match self {
			OpBinary::Add => "+",
			OpBinary::Sub => "-",
			OpBinary::Mul => "*",
			OpBinary::Div => "/",
			OpBinary::Mod => "%",
			OpBinary::Assign => "=",
			OpBinary::Equal => "==",
			OpBinary::And => "and",
			OpBinary::Or => "or",
		};
		write!(f, "{symbol}")
	}
}

impl OpTernary {
	pub fn get(token: &str) -> Option<(OpTernary, &'static str)> {
		let op = match token {
			"?" => (OpTernary::Conditional, ":"),
			_ => return None,
		};
		Some(op)
	}

	pub fn get_symbol(&self) -> (&'static str, &'static str) {
		match self {
			OpTernary::Conditional => ("?", ":"),
		}
	}
}

/// Predefined levels of precedence in order from the highest.
///
/// Operators within a same level of precedence can be further ordered by
/// the numeric field in ascending order, with highest precedence first
/// (i.e. lower numeric value).
#[allow(unused)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum Precedence {
	Member(i8),
	Unary(i8),
	Power(i8),
	Multiplicative(i8),
	Additive(i8),
	Comparison(i8),
	Logical(i8),
	Conditional(i8),
	Assignment(i8),
	Comma(i8),
}

pub enum Grouping {
	Left,
	Right,
}

impl Op {
	fn precedence(&self) -> (Precedence, Grouping) {
		match *self {
			Op::Unary(op) => match op {
				OpUnary::Not => (Precedence::Logical(1), Grouping::Right),
				OpUnary::Plus => (Precedence::Unary(0), Grouping::Left),
				OpUnary::Minus => (Precedence::Unary(0), Grouping::Left),
				OpUnary::Negate => (Precedence::Unary(0), Grouping::Left),
				OpUnary::PreIncrement => (Precedence::Unary(0), Grouping::Left),
				OpUnary::PreDecrement => (Precedence::Unary(0), Grouping::Left),
				OpUnary::PosIncrement => (Precedence::Unary(0), Grouping::Left),
				OpUnary::PosDecrement => (Precedence::Unary(0), Grouping::Left),
			},
			Op::Binary(op) => match op {
				OpBinary::Mul => (Precedence::Multiplicative(0), Grouping::Left),
				OpBinary::Div => (Precedence::Multiplicative(0), Grouping::Left),
				OpBinary::Mod => (Precedence::Multiplicative(0), Grouping::Left),
				OpBinary::Add => (Precedence::Additive(0), Grouping::Left),
				OpBinary::Sub => (Precedence::Additive(0), Grouping::Left),
				OpBinary::Equal => (Precedence::Comparison(0), Grouping::Left),
				OpBinary::Assign => (Precedence::Assignment(0), Grouping::Right),
				OpBinary::And => (Precedence::Logical(0), Grouping::Right),
				OpBinary::Or => (Precedence::Logical(1), Grouping::Right),
			},
			Op::Ternary(op) => match op {
				OpTernary::Conditional => (Precedence::Conditional(0), Grouping::Right),
			},
		}
	}
}

impl std::cmp::Ord for Op {
	fn cmp(&self, other: &Self) -> Ordering {
		let (lp, lg) = self.precedence();
		let (rp, rg) = other.precedence();
		let cmp = lp.cmp(&rp);
		if cmp == Ordering::Equal {
			/*
			   Use associativity to decide precedence, as following:

			   +------+------+------------+-----------------------+
			   | L    | R    | Precedence | Example               |
			   +------+------+------------+-----------------------+
			   |  <-  |  <-  |      L     |    ((a <- b) <- c)    |
			   |  ->  |  ->  |      R     |    (a -> (b -> c))    |
			   |  <-  |  ->  |      R     |    (a <- (b -> c))    |
			   |  ->  |  <-  |      L     |    ((a -> b) <- c)    |
			   +------+------+------------+-----------------------+
			*/
			match lg {
				Grouping::Left => match rg {
					Grouping::Left => Ordering::Less,
					Grouping::Right => Ordering::Greater,
				},
				Grouping::Right => match rg {
					Grouping::Right => Ordering::Greater,
					Grouping::Left => Ordering::Less,
				},
			}
		} else {
			cmp
		}
	}
}

impl std::cmp::PartialOrd for Op {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}
