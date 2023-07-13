# Parser scopes

TODO: update to reflect the actual scope implementation

On a basic level, a Byte file is parsed as follows:

A configurable lexer reads the input text and incrementally generates a stream
of lexemes. As the input is read, the lexer can be reconfigured so new tokens
can be supported mid-stream.

This lexeme stream, or lex stream for short, is then parsed in terms of scopes.

A scope limits the parsing to a particular range of lexemes. It can also filter
the input stream to apply certain rules (e.g. certain scopes can skip line
breaks or filter out indentation tokens).

Scopes can be:

- Nested: the child scope will operate within the tokens and limits of the
  parent scope.
- Overridden: a child scope can override the parent scope (e.g. parenthesis).
- Boxed: the scope limits can't be overridden.

Note that Byte balances INDENT and DEDENT inside parenthesis at the lexer level
such that any INDENT after an open parenthesis will generate a DEDENT before the
matching closing parenthesis.


## Statement scope

This is a top-level boxed scope used to parse a file at the top-level, but
can also be triggered inside blocks.

- Extends to the end of the line unless followed by an INDENT.
- If followed by an IDENT, continues until the matching DEDENT.
- Note that parenthesis do not force continuation without an INDENT.

Statement scope can also consider other breaks such as `;`.

## Parenthesis scope

This scopes overrides the parent scope and balances parenthesis and indentation
until the matching close parenthesis is found.

## Line scope

Nested scope that goes only to the end of the line. If followed by an INDENT
the line continues with any INDENT, DEDENT, and line BREAK filtered out until
the matching DEDENT is found.
