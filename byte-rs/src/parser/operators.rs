use std::collections::HashMap;

use crate::lexer;

/// Enumerate the type of supported operators with its symbols.
#[derive(Copy, Clone)]
pub enum Op {
	UnaryPrefix(&'static str),
	UnarySuffix(&'static str),
	Binary(&'static str, OpGroup),
	Ternary(&'static str, &'static str),
	List(&'static str),
	BracketSuffix(&'static str, &'static str),
}

#[derive(Copy, Clone)]
pub enum OpGroup {
	Left,
	Right,
}

/// Predefined precedence levels.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum OpPrecedence {
	Scope,
	Member,
	Prefix,
	Suffix,
	Pow,
	Mul,
	Add,
	Shift,
	Compare,
	Bitwise,
	Logical,
	Ternary,
	Assignment,
	Comma,
}

impl OpPrecedence {
	pub fn least() -> Self {
		OpPrecedence::Comma
	}

	pub fn next(self) -> Option<Self> {
		let level = match self {
			OpPrecedence::Scope => return None,
			OpPrecedence::Member => OpPrecedence::Scope,
			OpPrecedence::Prefix => OpPrecedence::Member,
			OpPrecedence::Suffix => OpPrecedence::Prefix,
			OpPrecedence::Pow => OpPrecedence::Suffix,
			OpPrecedence::Mul => OpPrecedence::Pow,
			OpPrecedence::Add => OpPrecedence::Mul,
			OpPrecedence::Shift => OpPrecedence::Add,
			OpPrecedence::Compare => OpPrecedence::Shift,
			OpPrecedence::Bitwise => OpPrecedence::Compare,
			OpPrecedence::Logical => OpPrecedence::Bitwise,
			OpPrecedence::Ternary => OpPrecedence::Logical,
			OpPrecedence::Assignment => OpPrecedence::Ternary,
			OpPrecedence::Comma => OpPrecedence::Assignment,
		};
		Some(level)
	}
}

#[derive(Default)]
pub struct OpTable {
	empty: Vec<(i64, Op)>,
	prefix: Vec<Op>,
	suffix: Vec<Op>,
	levels: HashMap<OpPrecedence, Vec<(i64, Op)>>,
}

impl OpTable {
	pub fn get(&self, level: OpPrecedence) -> impl Iterator<Item = Op> + '_ {
		self.levels
			.get(&level)
			.unwrap_or(&self.empty)
			.iter()
			.map(|(_, it)| it)
			.cloned()
	}

	pub fn get_all_prefix(&self) -> &Vec<Op> {
		&self.prefix
	}

	pub fn get_all_suffix(&self) -> &Vec<Op> {
		&self.suffix
	}

	pub fn with(self, level: OpPrecedence) -> OpTableBuilder {
		OpTableBuilder {
			level,
			order: 0,
			table: self,
		}
	}

	pub fn export_symbols(&self, symbols: &mut lexer::SymbolTable) {
		let mut add = |sym: &'static str| {
			if let Some('a'..='z' | 'A'..='Z' | '_') = sym.chars().next() {
				// keyword operator, don't add
			} else if sym != "" {
				symbols.add_symbol(sym);
			}
		};

		for it in self.levels.values() {
			for (_, op) in it {
				match op {
					Op::UnaryPrefix(sym) => add(sym),
					Op::UnarySuffix(sym) => add(sym),
					Op::Binary(sym, _) => add(sym),
					Op::Ternary(a, b) => {
						add(a);
						add(b);
					}
					Op::List(sym) => add(sym),
					Op::BracketSuffix(a, b) => {
						add(a);
						add(b);
					}
				}
			}
		}
	}
}

pub struct OpTableBuilder {
	level: OpPrecedence,
	order: i64,
	table: OpTable,
}

impl OpTableBuilder {
	pub fn add_operator(mut self, op: Op) -> Self {
		let mut entry = self.table.levels.entry(self.level);
		let list = entry.or_insert(Default::default());
		list.push((self.order, op));
		list.sort_by_key(|x| x.0);
		self
	}

	pub fn after(self) -> Self {
		OpTableBuilder {
			level: self.level,
			order: self.order + 1,
			table: self.table,
		}
	}

	pub fn before(self) -> Self {
		OpTableBuilder {
			level: self.level,
			order: self.order - 1,
			table: self.table,
		}
	}

	pub fn with(self, level: OpPrecedence) -> Self {
		self.table.with(level)
	}

	pub fn table(self) -> OpTable {
		self.table
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn precedence_next_returns_higher_precedence() {
		let mut prec = OpPrecedence::least();
		assert_eq!(prec, OpPrecedence::Comma);
		while let Some(next) = prec.next() {
			prec = next;
		}
		assert_eq!(prec, OpPrecedence::Scope);
	}
}
