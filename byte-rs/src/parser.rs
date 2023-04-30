use std::io::Write;

use crate::core::repr::*;
use crate::lexer::*;
use crate::nodes::*;
use crate::vm::Op;

pub fn parse(input: crate::core::input::Input) {
	let mut lexer = open(input);
	let mut list = Vec::new();
	let mut resolver = NodeResolver::new();
	while let Some(next) = parse_next(&mut lexer) {
		list.push(next.clone());
		resolver.resolve(next.clone());
		if lexer.has_errors() || next.has_errors() {
			break;
		}
	}

	resolver.wait();

	let mut errors = lexer.errors();
	for it in list.iter() {
		errors.append(it.errors())
	}
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

pub fn parse_next(lexer: &mut Lexer) -> Option<Node> {
	if lexer.next().is_none() {
		None
	} else {
		let next = parse_line(lexer, Stop::None, true);
		match next.len() {
			0 => {
				let next = lexer.next();
				lexer.error_at(Some(next.span()), format!("expected statement, got {next}"));
				None
			}
			1 => next.into_iter().next(),
			_ => Some(Block::new(next)),
		}
	}
}

fn parse_expr_group(lexer: &mut Lexer, limit: Stop) -> Node {
	let next = lexer.next();
	if next.token() == Token::Break {
		lexer.read();
		let next = lexer.next();
		let indented = if lexer.next().token() == Token::Indent {
			lexer.read();
			true
		} else {
			lexer.error_at(Some(next.span()), "parenthesized block must be indented");
			false
		};

		let block = if let Some(block) = parse_block(lexer, limit) {
			block
		} else {
			Block::new(Vec::new())
		};

		if indented {
			let next = lexer.next();
			if next.token() == Token::Dedent {
				lexer.read();
			} else {
				lexer.error_at(Some(next.span()), format!("dedent expected, got {next}"));
			}
		}
		block
	} else {
		let expr = parse_line(lexer, limit, false);
		match expr.len() {
			0 => Node::new(Raw::empty()),
			1 => expr[0].clone(),
			_ => Block::new(expr),
		}
	}
}

fn parse_block(lexer: &mut Lexer, limit: Stop) -> Option<Node> {
	let mut block = Vec::new();
	while !limit.should_stop(lexer) {
		let mut next = parse_line(lexer, limit, false);
		if next.len() == 0 || lexer.has_errors() || next.iter().any(|x| x.has_errors()) {
			break;
		}

		block.append(&mut next);
	}

	match block.len() {
		0 => None,
		1 => Some(block.into_iter().next().unwrap()),
		_ => Some(Block::new(block.clone())),
	}
}

fn parse_line(lexer: &mut Lexer, limit: Stop, top_level: bool) -> Vec<Node> {
	let mut block = Vec::new();
	let mut is_complete = false;
	while !limit.should_stop(lexer) {
		if lexer.has_errors() {
			break;
		}

		let (next, complete) = parse_expr_with_block(lexer, limit);
		is_complete = complete;
		if let Some(next) = next {
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
				if is_complete {
					break;
				}
			}
		}
	}

	// Check if the block stopped at a valid point
	if !limit.should_stop(lexer) && !lexer.has_errors() && !is_complete {
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
			lexer.error_at(Some(next.span()), format!("unexpected {next} after block"))
		}
	}

	block
}

/// Parse a single expression with an optional indented block.
fn parse_expr_with_block(lexer: &mut Lexer, limit: Stop) -> (Option<Node>, bool) {
	// parse the basic expression
	let (expr, is_complete) = parse_expr(lexer, limit);
	let expr = if let Some(expr) = expr {
		expr
	} else {
		return (None, false);
	};

	let node = Raw::new(expr.clone());
	// parse indented block, if any
	let next = lexer.next();
	let (node, is_complete) =
		if next.token() == Token::Symbol(":") && lexer.lookahead(1).token() == Token::Break {
			let colon = next.clone();
			if lexer.lookahead(2).token() != Token::Indent {
				lexer.error_at(
					Some(colon.span()),
					format!("a block start must be followed by an indented block"),
				);

				// skip just the colon and return the expression
				lexer.read();
				return (Some(node), is_complete);
			}

			// skip to the start of the block
			lexer.skip(3);

			let block = if let Some(block) = parse_block(lexer, limit) {
				block
			} else {
				if !lexer.has_errors() {
					lexer.error_at(Some(colon.span()), "empty block is not allowed");
				}
				return (Some(node), true);
			};

			let next = lexer.next();
			if next.token() == Token::Dedent {
				lexer.read();
			} else if !lexer.has_errors() {
				lexer.error_at(
					Some(next.span()),
					format!(
						"`:` block at {}: expected dedent, got {next}",
						colon.span().sta
					),
				);
			}

			let block_expr = BlockExpr::new(node.clone(), block.clone());
			(block_expr, true)
		} else {
			(node, is_complete)
		};

	// Check if the expression stopped at a valid point
	if !limit.should_stop(lexer) && !is_complete && !lexer.has_errors() {
		let next = lexer.next();
		let valid = next.first_of_line()
			|| match next.token() {
				Token::None => true,
				Token::Break => true,
				Token::Dedent => true,
				Token::Symbol(";") => true,
				_ => false,
			};
		if !valid {
			lexer.error_at(
				Some(next.span()),
				format!("unexpected {next} after expression"),
			)
		}
	}

	(Some(node), is_complete)
}

/// Parse a plain expression, stopping at the first unsupported token.
fn parse_expr(lexer: &mut Lexer, limit: Stop) -> (Option<Node>, bool) {
	let mut expr: Option<Node> = None;
	let mut level = 0;
	let mut done = false;
	let mut is_complete = false;
	while !done {
		// parse a sequence of atoms
		while let Some(atom) = parse_atom(lexer, limit) {
			if let Some(ref mut expr) = expr {
				expr.push(atom);
			} else {
				expr = Some(atom);
			}
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
					if level == 0 {
						is_complete = true
					}
					level > 0
				} else if level > 0 {
					// consume the line break if we are indented
					lexer.read();
					true
				} else {
					is_complete = true;
					lexer.read();
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
			lexer.error_at(Some(lexer.next().span()), "invalid indentation");
		}
	}

	(expr, is_complete)
}

/// Parse an expression atom.
fn parse_atom(lexer: &mut Lexer, limit: Stop) -> Option<Node> {
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
			let sta = next;
			let node = parse_expr_group(lexer, Stop::Symbol(end_symbol));
			let end = lexer.next();
			if end.symbol() != Some(end_symbol) {
				lexer.error_at(
					Some(end.span()),
					format!(
						"{sta} parenthesized expression at {}: expected closing `{end_symbol}`, got {end}",
						sta.span().short()
					),
				);
			} else {
				lexer.read();
			}

			Group::new(sta, end, node.clone())
		} else {
			Atom::from(next)
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
