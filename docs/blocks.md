# Blocks

TODO: a bunch of this has changed since the original idea, update

Blocks make for the fundamental structure of a Byte file. They provide a
well-defined structure for the language and provide the building blocks
for the language extensible and dynamic syntax.

The block syntax is defined from indentation, line breaks, parenthesis and
brackets, plus a few basic symbols such as colons (:), semi-colons (;), and
commas (,). The actual tokens can be customized at the lexer level.

Blocks are absolute. They are parsed and delimited before any other parsing
stage. Higher-level constructs are parsed within and from these basic blocks.

# Indentation

Blocks are absolute, and within the block syntax indentation is absolute.

When parsing blocks, the indentation level of each non-blank line is considered:

- Any indented line is ALWAYS nested under the previous line with a lower
  indentation level.
- Consecutive lines at the same indentation level are ALWAYS parsed separately.
- Parenthesis and brackets NEVER override indentation, but MUST ALWAYS balance.
- How nested and consecutive lines are parsed will depend on the parent block
  context and syntax.

Here are the basic indentation rules:

- Indentation is only considered for non-blank lines.
- The line's indentation level is the column of its first non-blank character.
- Comments ARE considered for indentation purposes.

Additionally, to prevent confusing issues with indentation, the following
rules must also be considered.

- The only valid whitespace for a line is U+0020 (space) and U+0009 (TAB).
  - Any other space characters (Zs) in indentation are an error.
- Tabs and spaces can be mixed, but tabs MUST always be first.
- Between any two sequential lines, the leading indentation can only differ
  by its suffix.
  - This means that indentation changes can only add or remove characters. For
    example, you cannot have a line indented by tabs and then switch to spaces,
	or vice-versa.
  - Blank lines are ignored, that is, lines are considered sequential even
    if separated by blank lines.

# Blocks and the Lexer

Before blocks are parsed, the input stream is split into tokens by using
the Lexer. 

As with the parser, the Lexer for the language can also be extended with new
symbols and literals. Those extended symbols can also affect the block parsing.

Blocks are parsed sequentially in incremental steps. As such, any lexer changes
from a given file are only visible to subsequent blocks inside the scope where
these changes happen.

To allow lexer extensions within files to work with the block parser, the lexer
configuration uses an explicit and limited syntax that is processed inline with
the block parsing.

# Types of blocks

## Lines

Any sequence of text lines with the same indentation level will be split into
individual line blocks, unless overridden by parenthesis.

## Parenthesized

Any opening token (e.g. parenthesis or bracket) within a line will start a
parenthesized block that will continue until the matching close token.

> Any opening or closing token MUST have a matching pair within
> the same block.

Parenthesized blocks can span across lines, but any line within it MUST be
indented.

A parenthesized block MUST NOT have a DEDENT without a matching INDENT.

> When the closing token is found, an implicit DEDENT token is generated
> for each IDENT token within the parenthesized block. The parent block
> context is then resumed as if at the same indentation level.


Examples:

	if (1 +            # starting level
		2 +            # indent within the parenthesis
		3 +            # same level (separate line, but merged by the syntax)
		4) <= 10:      # implicit dedent back to the if level
		true           # indented under the if

	# also valid (same overall syntax):
	if (
		1 +            # indent
		2              # same level
	) <= 10:           # explicit dedent
		true


	# despite the extra INDENT and DEDENT, those end up as equivalent
	call 1, (2, 3), 4
	call 1, (
		2, 3           # generates an INDENT and later a DEDENT
	), 4

	# this is completely invalid, though
	if A:
		(
			B,
			C,
	)                  # error: additional DEDENT inside the parenthesis

## Indented continuation

Any sequence of indented lines at the same level will be nested under the
previous line with a lesser level:

	parent A
		child '1A'
		child '2B'

		child '3B'
	
	parent B
		child '1B'

The semantics of the indented lines will depend on the context. For example,
plain expressions will just merge the sequential lines:

	# assign A + B + C to X
	let X =
		A +
		B +
		C

	# this will also work
	let X
		= A
		+ B
		+ C

## Indented block

Some tokens will start a block when used at the end of a line. They MUST be
followed by an INDENT:

	if X < Y:
		something()
	else:              # this comment is ignored
		or_other()

This is just an extension of the continuation syntax, but it is enforced by
the compiler. The resulting grammar for extended blocks is also different.
