use crate::lexer;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(unused)]
pub enum Token {
	None,
	LineBreak,
	Ident,
	Dedent,
	Identifier,
	Integer,
	String,
	Symbol,
	Comma,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Pos {
	pub line: usize,
	pub column: usize,
	pub offset: usize,
}

#[derive(Copy, Clone, Debug)]
pub struct Span {
	pub pos: Pos,
	pub end: Pos,
}

impl std::fmt::Display for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.pos)
	}
}

impl std::fmt::Display for Pos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:03},{:02}", self.line + 1, self.column + 1)
	}
}

pub trait Reader: lexer::Input {
	fn read_text(&mut self, span: Span) -> &str;

	fn pos(&mut self) -> Pos;
}

pub struct TokenStream<'a, T: Reader> {
	input: &'a mut T,
	next: Option<(Token, Span)>,
}

#[allow(unused)]
impl<'a, T: Reader> TokenStream<'a, T> {
	pub fn new(input: &'a mut T) -> TokenStream<'a, T> {
		TokenStream { input, next: None }
	}

	/// Returns the next available token in this stream or None at the end
	/// of the stream.
	///
	/// This does not consume the token. Multiple calls to this method will
	/// return the same token until [`TokenStream::shift`] is called.
	pub fn get(&mut self) -> Token {
		self.read().0
	}

	pub fn span(&mut self) -> Span {
		self.read().1
	}

	pub fn text(&mut self) -> &str {
		let span = self.span();
		self.input.read_text(span)
	}

	/// Returns a token to the stream, making it available for reading again.
	pub fn unget(&mut self, token: Token) {
		todo!()
	}

	/// Consumes the next token in the stream.
	pub fn shift(&mut self) {
		self.next = None;
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at a [`Token::Ident`].
	pub fn indented(&mut self) -> TokenStream<T> {
		todo!()
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at tokens for which `f` returns true.
	pub fn until(&mut self, f: fn(Token) -> bool) -> TokenStream<T> {
		todo!()
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at the given right parenthesis.
	///
	/// Note that this overrides an [`TokenStream::until`] limitation.
	pub fn parenthesized(&mut self, right: Token) -> TokenStream<T> {
		todo!()
	}

	fn read(&mut self) -> (Token, Span) {
		let next = self.next.get_or_insert_with(|| {
			let (token, pos) = loop {
				let pos = self.input.pos();
				let (token, ok) = lexer::read_token(self.input);
				if token != Token::None {
					break (token, pos);
				} else if !ok {
					break (Token::None, pos);
				}
			};
			let end = self.input.pos();
			(token, Span { pos, end })
		});
		*next
	}
}
