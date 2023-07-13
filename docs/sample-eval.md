# Sample code

```
# this is implicit
import core
use!(core)

# Enable to output debug information
const DEBUG = true

if DEBUG:
	let time = now()
	print!("{time} - current version is {VERSION}")

macro! print(args: FnArgs):
	format!($args)


const MAJOR = 1
const MINOR = 23
const VERSION = $"{MAJOR}.{MINOR}"
```

## Lexer

```
0: COMMENT BR
0: W(import) W(core) BR
0: W(use) S(!) `(` W(core) `)` BR
0: COMMENT BR
0: W(const) W(DEBUG) S(=) W(true) BR
0: W(if) W(DEBUG) S(:) BR
4: W(let) W(time) S(=) W(now) `(` `)` BR
4: W(print) S(!) `(` VALUE `)` BR
0: W(macro) S(!) W(print) `(` W(args) S(:) W(FnArgs) `)` S(:) BR
4: W(format) S(!) `(` S($) W(args) `)` BR
0: W(const) W(MAJOR) S(=) VALUE BR
0: W(const) W(MINOR) S(=) VALUE BR
0: W(const) W(VERSION) S(=) S($) VALUE BR
```

```
[00] COMMENT [01] BR         [02] W(import) [03] W(core)     [04] BR       [05] W(use)      [06] S(!)     [07] `(`         [08] W(core)   [09] `)`
[10] BR      [11] COMMENT    [12] BR        [13] W(const)    [14] W(DEBUG) [15] S(=)        [16] W(true)  [17] BR          [18] W(if)     [19] W(DEBUG)
[20] S(:)    [21] BR         [22] W(let)    [23] W(time)     [24] S(=)     [25] W(now)      [26] `(`      [27] `)`         [28] BR        [29] W(print)
[30] S(!)    [31] `(`        [32] VALUE     [33] `)`         [34] BR       [35] W(macro)    [36] S(!)     [37] W(print)    [38] `(`       [39] W(args)
[40] S(:)    [41] W(FnArgs)  [42] `)`       [43] S(:)        [44] BR       [45] W(format)   [46] S(!)     [47] `(`         [48] S($)      [49] W(args)
[50] `)`     [51] BR         [52] W(const)  [53] W(MAJOR)    [54] S(=)     [55] VALUE       [56] BR       [57] W(const)    [58] W(MINOR)  [59] S(=)
[60] VALUE   [61] BR         [62] W(const)  [63] W(VERSION)  [64] S(=)     [65] S($)        [66] VALUE    [67] BR

INDENT - [00..21] = 0  /  [22..34] = 4  /  [35..44] = 0  /  [45..51] = 4  /  [52..67] = 0
BREAKS - 01 04 10 12 17 21 28 34 44 51 56 61 67
PARENS = (07..09)  (26..27)  (31..33)  (38..42)  (47..50)

S(!) - 06 30 36 46
S(=) - 15 24 54 59 64
S(:) - 20 40 43
S($) - 48 65

W(args)    - 39 49
W(const)   - 13 52 57 62
W(core)    - 03 08
W(DEBUG)   - 14 19
W(FnArgs)  - 41
W(format)  - 45
W(if)      - 18
W(import)  - 02
W(let)     - 22
W(macro)   - 35
W(MAJOR)   - 53
W(MINOR)   - 58
W(now)     - 25
W(print)   - 29
W(time)    - 37
W(true)    - 16
W(use)     - 05
W(VERSION) - 63
```

## Parsing

Operators: PARENS, SPLIT-INDENT, BLOCK, COMMENTS, LINES

```
- START

	R(00:67)

- PARENS

	R(00:06) P(07:09) R(10:25) P(26:27) R(28:30) P(31:33) R(34:37) P(38:42) R(43:46) P(47:50) R(51:67)

	R(00:06) P(R(08:08)) R(10:25) P(EMPTY) R(28:30) P(R(32:32)) R(34:37) P(R(39:41)) R(43:46) P(R(48:49)) R(51:67)

- SPLIT-INDENT

	R(00) v R(02:03) v R(05:06) P(R(08:08)) v R(11) v R(13:16) v R(18:20) v R(22:25) P(EMPTY) v R(29:30) P(R(32:32)) v R(35:37) P(R(39:41)) R(43) v R(45:46) P(R(48:49)) v R(52:55) v R(57:60) v R(62:66) v
	
	R(00) v R(02:03) v R(05:06) P(R(08:08)) v R(11) v R(13:16) v R(18:20) v R(22:25) P(EMPTY) v R(29:30) P(R(32:32)) v R(35:37) P(R(39:41)) R(43) v R(45:46) P(R(48:49)) v R(52:55) v R(57:60) v R(62:66)
	
	R(00) v R(02:03) v R(05:06) P(R(08:08)) v R(11) v R(13:16) v R(18:20) < R(22:25) P(EMPTY) v R(29:30) P(R(32:32)) > R(35:37) P(R(39:41)) R(43) < R(45:46) P(R(48:49)) > R(52:55) v R(57:60) v R(62:66)

- BLOCK

	R(00) v R(02:03) v R(05:06) P(R(08:08)) v R(11) v R(13:16) v R(18:20)
		BLOCK(
			R(22:25) P(EMPTY) v R(29:30) P(R(32:32))
		)
		R(35:37) P(R(39:41)) R(43) 
		BLOCK(
			R(45:46) P(R(48:49))
		)
		R(52:55) v R(57:60) v R(62:66)

- COMMENTS

	# DOC(00->02)
	# DOC(11->13)

	R(02:03) v R(05:06) P(R(08:08)) v R(13:16) v R(18:20)
		BLOCK(
			R(22:25) P(EMPTY) v R(29:30) P(R(32:32))
		)
		R(35:37) P(R(39:41)) R(43) 
		BLOCK(
			R(45:46) P(R(48:49))
		)
		R(52:55) v R(57:60) v R(62:66)

- LINES

	R(02:03)
	R(05:06) P(R(08:08))
	R(13:16)
	R(18:20)
	BLOCK(
		R(22:25) P(EMPTY)
		R(29:30) P(R(32:32))
	)
	R(35:37) P(R(39:41)) R(43) 
	BLOCK(
		R(45:46) P(R(48:49))
	)
	R(52:55)
	R(57:60)
	R(62:66)

- STR-EVAL

- MACRO-ID

- MACROS

- IMPORT

- CONST

- LET

- PRINT

- FORMAT

```
