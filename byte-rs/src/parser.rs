use std::io::Write;

use crate::core::repr::*;
use crate::lexer::*;
use crate::nodes::*;
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
	}

	resolver.wait();

	let errors = global_scope.errors();
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
		let expr = { read_line(&mut *lexer, scope.clone()) };
		let expr = Raw::new(expr);
		let expr = Node::new(expr, scope.clone());
		Some(expr)
	}
}

fn read_line(lexer: &mut Lexer, scope: Scope) -> Vec<Node> {
	let mut expr = Vec::new();
	while lexer.next().is_some() {
		let next = lexer.read();
		if next.token() == Token::Break {
			break;
		}
		let next = Node::new(Atom::from(next), scope.clone());
		expr.push(next);
	}
	expr
}
