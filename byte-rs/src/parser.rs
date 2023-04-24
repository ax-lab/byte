use std::io::Write;

use crate::core::repr::*;
use crate::lexer::*;
use crate::nodes::*;
use crate::old::stream::Stream;
use crate::vm::Op;

pub fn parse(input: crate::core::input::Input) {
	let mut lexer = open(input);
	let mut global_scope = Scope::new();
	let mut scope = global_scope.new_child();
	let mut list = Vec::new();
	let mut resolver = NodeResolver::new();
	while let Some(next) = parse_next(&mut lexer, &mut scope) {
		list.push(next.clone());
		scope = next.scope().inherit();
		resolver.resolve(next);
		if lexer.has_errors() || global_scope.has_errors() {
			break;
		}
	}

	resolver.wait();

	let mut errors = lexer.errors();
	errors.append(global_scope.errors());
	if !errors.empty() {
		super::print_error_list(errors);
		std::process::exit(1);
	}

	let mut repr = Repr::new(ReprMode::Debug, ReprFormat::Full);
	let repr = &mut repr;
	for (i, it) in list.into_iter().enumerate() {
		let _ = write!(repr, "\n>>> Node {}", i + 1);
		if let Some(span) = it.span() {
			let _ = write!(repr, " from {span}");
		}
		let _ = write!(repr, "\n\n");
		let _ = it.output_repr(&mut repr.indented().compact().display());
		let _ = write!(repr, "\n\n-- DEBUG REPR --\n\n");

		let repr = &mut repr.indented();
		let _ = it.output_repr(repr);
		let _ = write!(repr, "\n");
	}

	println!("{repr}");
	std::process::exit(0);
}

pub fn open(input: crate::core::input::Input) -> Lexer {
	use crate::lang::*;

	let mut lexer = Lexer::new(input.start(), Scanner::new());
	lexer.config(|scanner| {
		scanner.add_matcher(Comment);
		scanner.add_matcher(Identifier);
		scanner.add_matcher(Literal);
		scanner.add_matcher(Integer);

		scanner.add_symbol("(", Token::Symbol("("));
		scanner.add_symbol(")", Token::Symbol(")"));
		scanner.add_symbol(",", Token::Symbol(","));
		scanner.add_symbol(";", Token::Symbol(";"));
		scanner.add_symbol(":", Token::Symbol(":"));

		Op::add_symbols(scanner);
	});
	lexer
}

fn parse_next(lexer: &mut Lexer, scope: &mut Scope) -> Option<Node> {
	if lexer.next().is_none() {
		None
	} else {
		let next = parse_line(lexer, scope, Stop::None, true);
		if next.len() == 0 {}

		match next.len() {
			0 => {
				let next = lexer.next();
				scope.error_if(Some(next.span()), format!("expected statement, got {next}"));
				None
			}
			1 => next.into_iter().next(),
			_ => {
				let scope = next[0].scope();
				Some(Node::new(Block::new(next), scope))
			}
		}
	}
}

fn parse_expr_group(lexer: &mut Lexer, mut scope: Scope, limit: Stop) -> Node {
	let next = lexer.next();
	if next.token() == Token::Break {
		lexer.read();
		let next = lexer.next();
		let indented = if lexer.next().token() == Token::Indent {
			lexer.read();
			true
		} else {
			scope
				.errors_mut()
				.at(Some(next.span()), "parenthesized block must be indented");
			false
		};

		let block = if let Some(block) = parse_block(lexer, scope.clone(), limit) {
			block
		} else {
			Node::new(Block::new(Vec::new()), scope.clone())
		};

		if indented {
			let next = lexer.next();
			if next.token() == Token::Dedent {
				lexer.read();
			} else {
				scope
					.errors_mut()
					.at(Some(next.span()), format!("dedent expected, got {next}"));
			}
		}
		block
	} else {
		if let Some(expr) = parse_expr_with_block(lexer, scope.clone(), limit) {
			expr
		} else {
			Node::new(Raw::new(Vec::new()), scope)
		}
	}
}

fn parse_block(lexer: &mut Lexer, mut scope: Scope, limit: Stop) -> Option<Node> {
	let mut block = Vec::new();
	while !limit.should_stop(lexer) {
		let mut next = parse_line(lexer, &mut scope, limit, false);
		if next.len() == 0 || lexer.has_errors() || scope.has_errors() {
			break;
		}

		block.append(&mut next);
	}

	match block.len() {
		0 => None,
		1 => Some(block.into_iter().next().unwrap()),
		_ => Some(Node::new(Block::new(block), scope)),
	}
}

fn parse_line(lexer: &mut Lexer, scope: &mut Scope, limit: Stop, top_level: bool) -> Vec<Node> {
	let mut block = Vec::new();
	let mut is_block = false;
	while !limit.should_stop(lexer) {
		if lexer.has_errors() || scope.has_errors() {
			break;
		}

		if let Some(next) = parse_expr_with_block(lexer, scope.clone(), limit) {
			*scope = next.scope().inherit();
			is_block = next.is::<BlockExpr>();
			block.push(next);
		} else {
			break;
		}

		let next = lexer.next();
		match next.token() {
			Token::None => {}
			Token::Break => {
				break;
			}
			Token::Symbol(";") => {
				lexer.read();
				if lexer.next().token() == Token::Break {
					lexer.read();
				}
			}
			_ => {
				if is_block {
					break;
				}
			}
		}
	}

	// Check if the block stopped at a valid point
	if !limit.should_stop(lexer) && !lexer.has_errors() && !scope.has_errors() && !is_block {
		let next = lexer.next();
		let valid = match next.token() {
			Token::None => true,
			Token::Break => {
				lexer.read();
				true
			}
			Token::Dedent if !top_level => true,
			_ => false,
		};
		if !valid {
			scope
				.errors_mut()
				.at(Some(next.span()), format!("unexpected {next} after block"))
		}
	}

	block
}

/// Parse a single expression with an optional indented block.
fn parse_expr_with_block(lexer: &mut Lexer, scope: Scope, limit: Stop) -> Option<Node> {
	// parse the basic expression
	let expr = parse_expr(lexer, scope.clone(), limit);
	if expr.len() == 0 {
		return None;
	}

	let expr = Node::new(Raw::new(expr), scope);
	let mut scope = expr.scope();
	let mut has_block = false;

	// parse indented block, if any
	let next = lexer.next();
	let node = if next.token() == Token::Symbol(":") && lexer.lookahead(1).token() == Token::Break {
		let colon = next.clone();
		if lexer.lookahead(2).token() != Token::Indent {
			scope.errors_mut().at(
				Some(colon.span()),
				format!("a block start must be followed by an indented block"),
			);

			// skip just the colon and return the expression
			lexer.read();
			return Some(expr);
		}

		// skip to the start of the block
		lexer.skip(3);
		has_block = true;

		let mut block_scope = scope.new_child();
		let block = if let Some(block) = parse_block(lexer, block_scope.clone(), limit) {
			block
		} else {
			block_scope.error_if(Some(colon.span()), "empty block is not allowed");
			return Some(expr);
		};

		let next = lexer.next();
		if next.token() == Token::Dedent {
			lexer.read();
		} else {
			block_scope.error_if(
				Some(next.span()),
				format!(
					"`:` block at {}: expected dedent, got {next}",
					colon.span().sta
				),
			);
		}

		let node = Node::new(BlockExpr::new(expr, block), scope);
		node
	} else {
		expr
	};

	// Check if the expression stopped at a valid point
	let mut scope = node.scope();
	if !limit.should_stop(lexer) && !lexer.has_errors() && !scope.has_errors() && !has_block {
		let next = lexer.next();
		let valid = match next.token() {
			Token::None => true,
			Token::Break => true,
			Token::Dedent => true,
			_ => false,
		};
		if !valid {
			scope.errors_mut().at(
				Some(next.span()),
				format!("unexpected {next} after expression"),
			)
		}
	}

	Some(node)
}

/// Parse a plain expression, stopping at the first unsupported token.
fn parse_expr(lexer: &mut Lexer, mut scope: Scope, limit: Stop) -> Vec<Node> {
	let mut expr = Vec::new();
	let mut level = 0;
	let mut done = false;
	while !done {
		// parse a sequence of atoms
		while let Some(atom) = parse_atom(lexer, scope.clone(), limit) {
			expr.push(atom);
		}

		// check for a stop condition
		done = limit.should_stop(lexer) || {
			// at the end of the line we must handle indented continuations
			let next = lexer.next();
			if next.token() == Token::Break {
				let next = lexer.lookahead(1);
				let continues = if next.token() == Token::Indent {
					// consume the indent and increase the expression level
					level += 1;
					lexer.skip(2);
					true
				} else if next.token() == Token::Dedent && level > 0 {
					// consume the dedent and decrease the expression level
					level -= 1;
					lexer.skip(2);
					true
				} else if level > 0 {
					// consume the line break if we are indented
					lexer.read();
					true
				} else {
					// stop at the line break otherwise
					false
				};
				!continues
			} else {
				true
			}
		};
	}

	// restore indentation to the level at the start of the expression
	if level > 0 {
		if lexer.pop_indent_levels(level).is_err() {
			scope
				.errors_mut()
				.at(Some(lexer.next().span()), "invalid indentation");
		}
	}

	expr
}

/// Parse an expression atom.
fn parse_atom(lexer: &mut Lexer, mut scope: Scope, limit: Stop) -> Option<Node> {
	if limit.should_stop(lexer) {
		None // never cross a stop boundary
	} else {
		let next = lexer.next();
		let valid = match next.token() {
			// layout tokens
			Token::None => false,
			Token::Break => false,
			Token::Indent | Token::Dedent => false,
			// don't consume invalid tokens
			Token::Invalid => false,
			// statement separator
			Token::Symbol(";") => false,
			// block start
			Token::Symbol(":") => lexer.lookahead(1).token() != Token::Break,
			_ => true,
		};
		if !valid {
			return None;
		} else {
			lexer.read();
		}

		// Check for a parenthesized block
		let node = if let Some(end_symbol) = lexer.is_parenthesis(&next) {
			let mut scope = scope.new_child();
			let sta = next;
			let node = parse_expr_group(lexer, scope.clone(), Stop::Symbol(end_symbol));
			let end = lexer.next();
			if end.symbol() != Some(end_symbol) {
				scope.errors_mut().at(
					Some(end.span()),
					format!(
						"{sta} parenthesized expression at {}: expected ending `{end_symbol}`, got {end}",
						sta.span().short()
					),
				);
			} else {
				lexer.read();
			}
			Node::new(Group::new(sta, end, node), scope)
		} else {
			Node::new(Atom::from(next), scope)
		};
		Some(node)
	}
}

#[derive(Copy, Clone)]
enum Stop {
	None,
	Symbol(&'static str),
}

impl Stop {
	pub fn should_stop(&self, lexer: &Lexer) -> bool {
		let next = lexer.next();
		if next.is_none() {
			true
		} else {
			match self {
				Stop::None => false,
				Stop::Symbol(symbol) => next.symbol() == Some(symbol),
			}
		}
	}
}
