mod block;
mod scope;

pub use block::*;
pub use scope::*;

mod root;
mod statement;

pub use root::*;
pub use statement::*;

pub fn exec(input: crate::lexer::TokenStream) {
	let scope = RootScope::new(input);
	let (_, statements) = Root::parse(scope);
	for it in statements {
		it.print();
	}
	std::process::exit(0);
}
