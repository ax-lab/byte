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
		let expr = parse_expr_list(lexer, scope.clone(), Stop::None);
		Some(expr)
	}
}

fn parse_expr_block(lexer: &mut Lexer, mut scope: Scope, limit: Stop) -> Node {
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

		let mut list = Vec::new();
		loop {
			let next = parse_expr_list(lexer, scope.clone(), limit);
			scope = next.scope().inherit();
			list.push(next);

			let next = lexer.next();
			match next.token() {
				Token::Break => {
					lexer.read();
				}
				_ => break,
			}
		}

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

		if !limit.should_stop(lexer) {
			let next = lexer.next();
			match next.token() {
				Token::None | Token::Break => {}
				_ => scope.errors_mut().at(
					Some(next.span()),
					format!("unexpected {next} at the end of expression"),
				),
			}
		}
		if list.len() == 1 {
			list[0].clone()
		} else {
			Node::new(Block::new(list), scope)
		}
	} else {
		let expr = parse_expr(lexer, scope.clone(), limit);
		Node::new(Raw::new(expr), scope)
	}
}

fn parse_expr_list(lexer: &mut Lexer, root_scope: Scope, limit: Stop) -> Node {
	let mut list: Vec<Node> = Vec::new();
	let mut scope = root_scope.clone();
	loop {
		let expr = parse_expr(lexer, scope.clone(), limit);
		if expr.len() > 0 {
			let expr = Node::new(Raw::new(expr), scope.clone());
			scope = expr.scope().inherit();
			list.push(expr);
		}

		let next = lexer.next();
		if next.token() == Token::Symbol(":") && lexer.lookahead(1).token() == Token::Break {
			lexer.skip(2);
			if list.len() != 1 {
				scope.errors_mut().at(
					Some(next.span()),
					format!("blocks are not allowed in multi-expression lines"),
				);
			} else {
				let next = lexer.next();
				if next.token() == Token::Indent {
					lexer.read();
					let expr = list[0].clone();
					let block = parse_expr_block(lexer, scope.new_child(), limit);
					return Node::new(BlockExpr::new(expr, block), scope.clone());
				} else {
					scope
						.errors_mut()
						.at(Some(next.span()), format!("expected indented block"));
				}
			}
		}

		if !limit.should_stop(lexer) {
			let next = lexer.next();
			match next.token() {
				Token::None | Token::Break => {
					lexer.read();
					break;
				}
				Token::Symbol(";") => {
					lexer.read();
				}
				_ => {
					if list.len() != 1 {
						// expression lists don't allow extended syntaxes and such
						scope.errors_mut().at(
							Some(next.span()),
							format!("unexpected {next} in expression line"),
						)
					}
					break;
				}
			}
		} else {
			break;
		}
	}
	if list.len() == 0 {
		Node::new(Raw::new(Vec::new()), scope)
	} else if list.len() == 1 {
		list[0].clone()
	} else {
		Node::new(Block::new(list), scope)
	}
}

/// Parse an expression and stops at the next significant token.
fn parse_expr(lexer: &mut Lexer, scope: Scope, limit: Stop) -> Vec<Node> {
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

	if level > 0 {
		// restore indentation to the level at the start of the expression
		lexer.pop_indent_levels(level);
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
			let node = parse_expr_block(lexer, scope.clone(), Stop::Symbol(end_symbol));
			let end = lexer.next();
			if end.symbol() != Some(end_symbol) {
				scope.errors_mut().at(
					Some(end.span()),
					format!(
						"expected end `{end_symbol}` for {sta} from {} in grouped expression",
						sta.span().sta
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
