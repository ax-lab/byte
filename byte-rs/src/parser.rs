mod blocks;
mod context;
mod error;
mod statement;

pub use context::*;
pub use error::*;
pub use statement::*;

pub fn parse(input: crate::core::input::Input) {
	let mut ctx = open(input);
	let mut list = Vec::new();
	loop {
		let next = parse_next(&mut ctx);
		if let Statement::End(..) = next {
			break;
		}

		let next = next.resolve(&mut ctx);
		if let Some(next) = next {
			list.push(next);
		}
	}

	if ctx.has_errors() {
		super::print_error_list(ctx.errors());
		std::process::exit(1);
	}

	for it in list.into_iter() {
		println!("\n>>> {:?}\n{it:#?}", it.span());
	}

	std::process::exit(0);
}

pub fn open(input: crate::core::input::Input) -> Context {
	use crate::core::input::*;
	use crate::lang::*;
	use crate::lexer::*;

	let lexer = create_lexer(input);
	let context = Context::new(lexer);
	return context;

	fn create_lexer(input: Input) -> Lexer {
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

			scanner.add_symbol("=", Token::Symbol("="));
			scanner.add_symbol("+", Token::Symbol("+"));
			scanner.add_symbol("-", Token::Symbol("-"));
			scanner.add_symbol("%", Token::Symbol("%"));
			scanner.add_symbol("==", Token::Symbol("=="));
			scanner.add_symbol("..", Token::Symbol(".."));
		});
		lexer
	}
}

pub fn parse_next(ctx: &mut Context) -> Statement {
	use crate::core::error::*;
	use crate::lexer::*;

	let next = ctx.next();
	if next.is_none() {
		Statement::End(ctx.pos())
	} else {
		let expr = blocks::parse_line_expr(ctx);
		while ctx.next().is_some() {
			let next = ctx.read();
			if next.token() == Token::Break {
				break;
			} else if !ctx.has_errors() {
				ctx.add_error(Error::new(next.span(), ParserError::ExpectedEnd(next)));
			}
		}
		Statement::Expr(expr)
	}
}
