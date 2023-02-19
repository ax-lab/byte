use std::cmp::Ordering;

/// Predefined levels of precedence in order from the highest.
///
/// Operators within a same level of precedence can be further ordered by
/// the numeric field in ascending order, with highest precedence first
/// (i.e. lower numeric value).
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

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Operator {
	Unary(UnaryOp),
	Binary(BinaryOp),
	Ternary(TernaryOp),
	List(ListOp),
}

impl Operator {
	fn precedence(&self) -> (Precedence, Grouping) {
		match *self {
			Operator::Unary(op) => match op {
				UnaryOp::Not => (Precedence::Logical(1), Grouping::Right),
				UnaryOp::Plus => (Precedence::Unary(0), Grouping::Left),
				UnaryOp::Minus => (Precedence::Unary(0), Grouping::Left),
				UnaryOp::Negate => (Precedence::Unary(0), Grouping::Left),
				UnaryOp::Increment => (Precedence::Unary(0), Grouping::Left),
				UnaryOp::Decrement => (Precedence::Unary(0), Grouping::Left),
				UnaryOp::PosIncrement => (Precedence::Unary(0), Grouping::Left),
				UnaryOp::PosDecrement => (Precedence::Unary(0), Grouping::Left),
			},
			Operator::Binary(op) => match op {
				BinaryOp::Mul => (Precedence::Multiplicative(0), Grouping::Left),
				BinaryOp::Div => (Precedence::Multiplicative(0), Grouping::Left),
				BinaryOp::Mod => (Precedence::Multiplicative(0), Grouping::Left),
				BinaryOp::Add => (Precedence::Additive(0), Grouping::Left),
				BinaryOp::Sub => (Precedence::Additive(0), Grouping::Left),
				BinaryOp::Equal => (Precedence::Comparison(0), Grouping::Left),
				BinaryOp::Assign => (Precedence::Assignment(0), Grouping::Right),
			},
			Operator::Ternary(op) => match op {
				TernaryOp::Condition => (Precedence::Conditional(0), Grouping::Right),
			},
			Operator::List(op) => todo!(),
		}
	}
}

impl std::cmp::Ord for Operator {
	fn cmp(&self, other: &Self) -> Ordering {
		let (lp, lg) = self.precedence();
		let (rp, rg) = other.precedence();
		let cmp = lp.cmp(&rp);
		if cmp == Ordering::Equal {
			/*
			   Use associativity to decide precedence, as following:

			   +------+------+-------+-----------------------+
			   | L    | R    | Prec? | Example               |
			   +------+------+-------+-----------------------+
			   |  <-  |  <-  |   L   |    ((a <- b) <- c)    |
			   |  ->  |  ->  |   R   |    (a -> (b -> c))    |
			   |  <-  |  ->  |   R   |    (a <- (b -> c))    |
			   |  ->  |  <-  |   L   |    ((a -> b) <- c)    |
			   +------+------+-------+-----------------------+
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

impl std::cmp::PartialOrd for Operator {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnaryOp {
	Not,
	Plus,
	Minus,
	Negate,
	Increment,
	Decrement,
	PosIncrement,
	PosDecrement,
}

impl UnaryOp {
	pub fn get_prefix(token: &str) -> Option<UnaryOp> {
		let op = match token {
			"not" => UnaryOp::Not,
			"+" => UnaryOp::Plus,
			"-" => UnaryOp::Minus,
			"!" => UnaryOp::Negate,
			"++" => UnaryOp::Increment,
			"--" => UnaryOp::Decrement,
			_ => return None,
		};
		Some(op)
	}

	pub fn get_posfix(token: &str) -> Option<UnaryOp> {
		let op = match token {
			"++" => UnaryOp::PosIncrement,
			"--" => UnaryOp::PosDecrement,
			_ => return None,
		};
		Some(op)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SuffixOp {
	Inc,
	Dec,
}

impl SuffixOp {
	pub fn get(token: &str) -> Option<SuffixOp> {
		todo!()
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinaryOp {
	Mul,
	Div,
	Mod,
	Add,
	Sub,
	Equal,
	Assign,
}

impl BinaryOp {
	pub fn get(token: &str) -> Option<BinaryOp> {
		let op = match token {
			"*" => BinaryOp::Mul,
			"/" => BinaryOp::Div,
			"%" => BinaryOp::Mod,
			"+" => BinaryOp::Add,
			"-" => BinaryOp::Sub,
			"=" => BinaryOp::Assign,
			"==" => BinaryOp::Equal,
			_ => return None,
		};
		Some(op)
	}

	pub fn get_bracket(token: &str) -> Option<&'static str> {
		todo!()
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TernaryOp {
	Condition,
}

impl TernaryOp {
	pub fn get(token: &str) -> Option<(TernaryOp, &'static str)> {
		let op = match token {
			"?" => (TernaryOp::Condition, ":"),
			_ => return None,
		};
		Some(op)
	}
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ListOp {
	Comma,
}

impl ListOp {
	pub fn get(token: &str) -> Option<ListOp> {
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn precedence_should_be_ordered() {
		assert!(Precedence::Comma(0) > Precedence::Member(0));
		assert!(Precedence::Comma(0) > Precedence::Member(100));
		assert!(Precedence::Comma(1) > Precedence::Comma(-1));
	}
}
