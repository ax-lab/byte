package bootstrap

import (
	"fmt"
	"sort"
	"strings"
)

type Diff[T comparable] struct {
	src    []T
	dst    []T
	blocks []DiffBlock
}

func Compare[T comparable](input, output []T) Diff[T] {
	out := Diff[T]{
		src:    input,
		dst:    output,
		blocks: computeDiff(input, output),
	}
	return out
}

func (diff Diff[T]) Blocks() (out []DiffBlock) {
	return diff.blocks
}

func (diff Diff[T]) Text() (out []string) {
	for _, op := range diff.Blocks() {
		var sign rune
		if op.Kind < 0 {
			sign = '-'
		} else if op.Kind > 0 {
			sign = '+'
		} else {
			sign = ' '
		}

		for i := 0; i < op.Len; i++ {
			var txt T
			if op.Kind > 0 {
				txt = diff.dst[op.Dst+i]
			} else {
				txt = diff.src[op.Src+i]
			}
			out = append(out, fmt.Sprintf("%c%v", sign, txt))
		}
	}

	return out
}

func (diff Diff[T]) String() string {
	str := strings.Builder{}
	for _, it := range diff.Blocks() {
		if str.Len() > 0 {
			str.WriteRune('\n')
		}
		str.WriteString(it.String())
	}
	return str.String()
}

type DiffBlock struct {
	Kind int
	Src  int
	Dst  int
	Len  int
}

func (blk DiffBlock) String() string {
	var sign rune
	if blk.Kind < 0 {
		sign = '-'
	} else if blk.Kind > 0 {
		sign = '+'
	} else {
		sign = '='
	}
	return fmt.Sprintf("%c[%d] %d -> %d", sign, blk.Len, blk.Src, blk.Dst)
}

func computeDiff[T comparable](input, output []T) (out []DiffBlock) {
	lcs := computeLcs(input, output)
	lcs = append(lcs, [3]int{len(input), len(output), 0})

	src, dst := 0, 0
	for _, it := range lcs {
		s, d, l := it[0], it[1], it[2]
		if del := s - src; del > 0 {
			out = append(out, DiffBlock{
				Kind: -1,
				Src:  src,
				Dst:  d,
				Len:  del,
			})
		}
		if ins := d - dst; ins > 0 {
			out = append(out, DiffBlock{
				Kind: +1,
				Src:  s,
				Dst:  dst,
				Len:  ins,
			})
		}

		if l > 0 {
			if cnt := len(out); cnt > 0 && out[cnt-1].Kind == 0 {
				out[cnt-1].Len += l
			} else {
				out = append(out, DiffBlock{
					Kind: 0,
					Src:  s,
					Dst:  d,
					Len:  l,
				})
			}
		}

		src, dst = s+l, d+l
	}

	return out
}

func computeLcs[T comparable](a, b []T) (out [][3]int) {
	return LCS(a, 0, len(a), b, 0, len(b))
}

func ComputeLcsOld[T comparable](a, b []T) (out [][3]int) {
	m := make([]int, len(a)*len(b))

	get := func(xa, xb int) int {
		if xa >= len(a) || xb >= len(b) {
			return 0
		}
		return m[xb*len(a)+xa]
	}

	set := func(xa, xb, val int) {
		m[xb*len(a)+xa] = val
	}

	for xa := len(a) - 1; xa >= 0; xa-- {
		for xb := len(b) - 1; xb >= 0; xb-- {
			var val int
			if a[xa] == b[xb] {
				val = 1 + get(xa+1, xb+1)
			} else {
				v1 := get(xa+1, xb)
				v2 := get(xa, xb+1)
				if v1 > v2 {
					val = v1
				} else {
					val = v2
				}
			}
			set(xa, xb, val)
		}
	}

	xa, xb := 0, 0
	for xa < len(a) && xb < len(b) {
		if a[xa] == b[xb] {
			out = append(out, [3]int{xa, xb, 1})
			xa++
			xb++
		} else if get(xa+1, xb) >= get(xa, xb+1) {
			xa++
		} else {
			xb++
		}
	}

	return out
}

/*
Computes the D value for the optimal D-path between A and B for the Myers
diff-algorithm.

The D parameter is the number of vertical or horizontal edges for the
optimal D-path in the edit graph for A to B. This can also be seen as the
number of character edit operations necessary to transform A into B.

The edit graph for A[N], B[M] is defined by a grid from (0, 0) to (N, M),
with edges in this grid corresponding to edit operations from A to B:

- Horz edge (x, y) -> (x + 1, y): deletes character A[x]
- Vert edge (x, y) -> (x, y + 1): inserts character B[Y] at A[x]
- Diag edge (x, y) -> (x + 1, y + 1), iif A[x] == B[y]: no operation

Any path from (0, 0) to (N, M) defines a set of edit operations to transform
the string A into B. An optimal path will minimize the number of horizontal
and vertical edges, hence will minimize D.

Any D-path can be decomposed into a (D-1)-path, plus an horz/vert edge followed
by a diagonal (potentially empty) snake.

A furthest reaching D-path is a D-path ending at the furthest possible (x, y)
coordinate from the origin.
*/
func ComputeD(a, b string) int {
	// max possible value of D (i.e. when the LCS of A and B is empty)
	max := len(a) + len(b)
	if max == 0 {
		return 0 // A and B are empty
	}

	// Given a diagonal `k` in the edit graph, where `k = x - y`, the vector
	// V[K] will contain the endpoint of the furthest reaching D-path for
	// diagonal K and the current value of D.
	//
	// By definition, `y = x - k`, so V[k] only needs to store x.
	//
	// Note that K = 0 is the diagonal where A[i] == B[i] and will contain the
	// furthest reaching path with D = 0.
	//
	// By definition, a D-path must end on a diagonal K with:
	//
	//     K in { -D , -D+2 , ... , D-2 , D }
	//
	// The above be verified by induction from the trivial case for D = 0.
	vec := make([]int, 2*max+1)
	idx := func(i int) int {
		return i + max
	}
	get := func(i int) int {
		return vec[idx(i)]
	}
	set := func(i, val int) {
		vec[idx(i)] = val
	}

	// Finds the optimal D value by successively computing the furthest
	// reaching D-path for diagonal K until a path reaches (N, M).
	//
	// Note that since the parity of D and K is always the same, the D and D+1
	// values are disjoint. This allows us to build a D path from a D-1 path
	// in successive iterations.
	for d := 0; d <= max; d++ {
		for k := -d; k <= d; k += 2 {
			// Build the furthest reaching D-path for the current diagonal K
			// by extending the the furthest reaching (D-1) path computed in
			// the previous iteration
			var x int
			if k == -d || (k != d && get(k-1) < get(k+1)) {
				x = get(k + 1) // extend vertical edge from diag K+1
			} else {
				x = get(k-1) + 1 // extend horizontal edge from diag K-1
			}

			// extend the diagonal "snake" to find the furthest reaching point
			y := x - k
			for x < len(a) && y < len(b) && a[x] == b[y] {
				x++
				y++
			}

			// store the furthest reaching point and check the stop condition
			set(k, x)
			if x == len(a) && y == len(b) {
				return d
			}
		}
	}

	// The above loop is garanteed to find a D-path as long as `MAX = N + M`.
	panic("unreachable")
}

func LCS[T comparable](a []T, a0, a1 int, b []T, b0, b1 int) (out [][3]int) {
	if a1-a0 <= 0 || b1-b0 <= 0 {
		return nil
	}

	n := a1 - a0
	m := b1 - b0

	ls := diffFindMidSnakes(a[a0:a1], b[b0:b1])
	it := ls[0]
	d := it.Diff
	u := it.PosA
	v := it.PosB
	x := u + it.Len
	y := v + it.Len

	x += a0
	y += b0
	u += a0
	v += b0

	if d > 1 {
		out = LCS(a, a0, u, b, b0, v)
		if x-u > 0 {
			out = append(out, [3]int{u, v, x - u})
		}
		out = append(out, LCS(a, x, a1, b, y, b1)...)
	} else {
		if n < m {
			if a[0] == b[0] {
				out = append(out, [3]int{a0, b0, n})
			} else {
				out = append(out, [3]int{a0, b0 + 1, n})
			}
		} else {
			if a[0] == b[0] {
				out = append(out, [3]int{a0, b0, m})
			} else {
				out = append(out, [3]int{a0 + 1, b0, m})
			}
		}
	}

	return out
}

type diffSnake struct {
	PosA, PosB int // Coordinates for the diagonal (offset in A and B)
	Len        int // Length of the diagonal
	Diff       int // Edit count for the containing edit path (D for the optimal D-path)
	Segments   int // Number of segments
}

// Find the middle snakes for the optimal edit D-paths between A and B for
// the Myers diff algorithm.
//
// This will return a list of snakes for the optimal D-paths and the value
// for D. Each snake is a (possibly empty) diagonal in the edit path which
// corresponds to a common subsequence between A and B.
//
// More importantly, a snake divides the D-path into ⌈D/2⌉ and ⌊D/2⌋ paths
// which can be used to efficiently build an optimal edit script, that is,
// the LCS between A abd B, by splitting the search.
//
// Formally, the middle snake for a D-path from (0,0) to (N,M) is a diagonal
// in the edit grid with coordinates (x,y) -> (u,v) that splits the path as:
//
// - a ⌈D/2⌉ path from (0,0) to (x,y)
// - a ⌊D/2⌋ path from (u,v) to (N,M)
//
// Where `x-y = u-v` and `x >= u`. That is, the diagonal contains the
// overlap of both paths.
//
// Additionally, `u+v >= ⌈D/2⌉` and `x+y <= N+M - ⌊D/2⌋`.
//
// Also note that for the returned `diffSnake` values:
//
// > u = PosA, v = PosB, x = PosA + Len, y = PosB + Len, d = Diff
func diffFindMidSnakes[T comparable](a, b []T) (out []diffSnake) {

	//------------------------------------------------------------------------//
	//
	// Edit grids, D-paths, and snakes
	// ===============================
	//
	// Below is a sample edit grid. Any path in this grid describes a set of
	// edit operations that when applied transform A into B:
	//
	//   - horz edges from col X delete A[x]
	//   - vert edges from row Y insert B[y]
	//   - diagonal edges mean no edit, that is A[x] = B[y]
	//
	//         A[N=7] = abcabba
	//         B[M=6] = cbabac
	//
	//         a b c a b b a
	//       c ┌─┬─o─┬─┬─┬─┬─┐ 0
	//         │ │ │╲│ │ │ │ │
	//       b ┝─o─┼─┼─o─o─┼─┤ 1
	//         │ │╲│ │ │╲│╲│ │
	//       a o─┼─┼─o─┼─┼─o─┤ 2
	//         │╲│ │ │╲│ │ │╲│
	//       b ┝─o─┼─┼─o─o─┼─┤ 3
	//         │ │╲│ │ │╲│╲│ │
	//       a o─┼─┼─o─┼─┼─o─┤ 4
	//         │╲│ │ │╲│ │ │╲│
	//       c ┝─┼─o─┼─┼─┼─┼─┤ 5
	//         │ │ │╲│ │ │ │ │
	//         ┕─┴─┴─┴─┴─┴─┴─┙ 6
	//         0 1 2 3 4 5 6 7
	//
	// Diagonals in the grid correspond to common subsequences between A and B,
	// being referred here as "snakes".
	//
	// For any path in the above grid, we can define D as its number of non-
	// diagonal edges, which in turn corresponds to the number of individual
	// edit operations. Any path from (0,0) to (N,M) will edit A into B.
	//
	// Conversely, a D-path is any path in the grid with exactly D horizontal
	// or diagonal edges.
	//
	// The goal is to maximize the diagonal edges in a path, minimizing D and
	// the number of edit operations. This is the same problem as finding the
	// Longest Common Subsequence (LCS).
	//
	//
	// Diagonals and K
	// ===============
	//
	// We can also refer to a diagonal K in the grid, where `K = X-Y`.
	//
	//        0    1   2   3   4   5   6    7
	//         ┌───┬───┬───┬───┬───┬───┬───┐
	//         │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │
	//       -1┝───┼───┼───┼───┼───┼───┼───┤6
	//         │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │
	//       -2┝───┼───┼───┼───┼───┼───┼───┤5
	//         │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │
	//       -3┝───┼───┼───┼───┼───┼───┼───┤4
	//         │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │
	//       -4┝───┼───┼───┼───┼───┼───┼───┤3
	//         │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │
	//       -5┝───┼───┼───┼───┼───┼───┼───┤2
	//         │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │ ╲ │
	//         ┕───┴───┴───┴───┴───┴───┴───┙
	//       -6   -5  -4  -3  -2  -1   0    1
	//
	//
	// Additional definitions
	// ======================
	//
	// Following are important definitions that form the basis of this
	// algorithm.
	//
	// About D-paths and the diagonals K in the grid:
	//
	//   1) Any D-path will always end in a diagonal K where -D ≤ K ≤ D.
	//   2) The odd-parity of D and K is always the same.
	//
	// Also important to note that D ≤ len(A)+len(B).
	//
	// Additionally:
	//
	//   3) A D-path can always be decomposed into a (D-1)-path followed by
	//      a non-diagonal edge and a possibly empty diagonal "snake".
	//
	//
	// Furthest reaching D-paths
	// =========================
	//
	// A furthest reaching D-path is one that ends as far away as possible
	// from the (0,0) grid origin.
	//
	// The furthest reaching D-path for a diagonal K can be generated by
	// extending the furthest reaching (D-1) path from K+1 and K-1 and the
	// longest possible diagonal "snake".
	//
	//
	// Myers algorithm
	// ==============
	//
	// This algorithm works by finding the furthest reaching D-paths for
	// successive values of D for each of the K diagonals.
	//
	// The code works by searching for both forward and reverse paths until
	// paths with an overlapping diagonal "snake" are found. The snake is a
	// common subsequence of A and B and part of an optimal D-path.
	//
	// Moreover, the snake splits the optimal D-path into ⌈D/2⌉ and ⌊D/2⌋
	// paths. This can be used to recursively build the edit script, that is,
	// the LCS between A and B.
	//
	// Note that the overlapping snake can a diagonal of zero length.
	//
	// For each diagonal K, the furthest reaching point for the current D
	// is recorded in an array indexed by K. Since K = X-Y, only the value
	// of X needs to be stored.
	//
	// Since K and D have the same parity, for each D the algorithm computes
	// only the respective odd/even diagonals. This also means that the values
	// for K are disjoint from K+1 and K-1, allowing iterations to depend on
	// the previously computed D-1 values.
	//
	//------------------------------------------------------------------------//

	n, m := len(a), len(b)
	maxD := n + m

	// The maximum possible D is n+m, but since we work from both ends the loop
	// only needs to search until ⌈D/2⌉.
	halfD := (n + m + 1) / 2

	// The main diagonal for the reverse paths is delta. This also affects the
	// even/odd parity of the D-path.
	delta := n - m

	// The D for the edit path will always have the same parity as the size
	// difference between A and B (we need inserts or deletes to make up for
	// the size difference, while swapping elements preserves parity).
	//
	// This means that the odd/even parity of delta determines if a path
	// overlap is possible in the forward or reverse loops (see below).
	odd := delta%2 != 0

	// Furthest reaching D-path for diagonal K, where -maxD ≤ K ≤ maxD
	// for the forward and reverse paths.
	//
	// Note that since K=X-Y, we store only X with Y=X-K.
	fwd := make([]int, 2*maxD+1)
	rev := make([]int, len(fwd))

	fwdQuality := make([]diffQuality, len(fwd))
	revQuality := make([]diffQuality, len(rev))

	// For the reverse path, all diagonals start at the right of the grid.
	for i := 0; i < len(rev); i++ {
		rev[i] = n
	}

	// Offset to apply when indexing fwd and rev, since K can be negative.
	off := maxD

	// Search the furthest forward and reverse reaching paths for increasing
	// values of (half) D until we find an overlapping snake between them.
	for hd := 0; len(out) == 0 && hd <= halfD; hd++ {
		// Note that the following applies for these inner loops:
		//
		// - Forward path: D-path is HD + (HD - 1)
		// - Reverse path: D-path is HD + HD
		//
		// The forward path is always building towards an odd D-path, while
		// the reverse is always even.
		//
		// Since D will always have the same parity as `delta`, we only bother
		// checking for overlaps when the parity is possible.

		// forward path
		for k := -hd; k <= hd; k += 2 {
			// Extend the furthest path from either K-1 or K+1.
			var (
				nextX int
				fromK int
			)

			// We can either extend vertically or horizontally from a neighbour
			// diagonal...
			indexVert := k + 1 + off
			indexHorz := k - 1 + off
			switch k {
			// ...in edge cases there is no choice
			case -hd:
				fromK = indexVert
			case +hd:
				fromK = indexHorz
			// ...otherwise we take the diagonal with the furthest X value
			// while strongly favoring a horizontal extension (delete)
			default:
				if fwd[indexVert] == fwd[indexHorz]+1 {
					// in this case, both a vertical or horizontal extension
					// will reach the same position, so we check which one
					// provides a better quality
					if isDiffQualityWorse(fwdQuality[indexHorz], fwdQuality[indexVert]) {
						fromK = indexVert
					} else {
						fromK = indexHorz
					}
				} else if fwd[indexHorz] >= fwd[indexVert] {
					fromK = indexHorz
				} else {
					fromK = indexVert
				}
			}

			// Apply the non-diagonal edge extension.
			quality := fwdQuality[fromK]
			if fromK == indexHorz {
				nextX = fwd[indexHorz] + 1
				quality.moveHorz()
			} else {
				nextX = fwd[indexVert]
				quality.moveVert()
			}

			// Extend the diagonal snake as far as possible.
			nextY := nextX - k
			for nextX < len(a) && nextY < len(b) && a[nextX] == b[nextY] {
				quality.moveDiag()
				nextX++
				nextY++
			}

			fwd[k+off] = nextX
			fwdQuality[k+off] = quality

			// Check for overlap with the reverse (D-1)-paths. These diagonals
			// are centered around `delta`, so we check if our forward K is
			// also one of the previously calculated reverse Ks.
			if odd && k >= delta-(hd-1) && k <= delta+(hd-1) {
				if posA := rev[k+off]; posA <= nextX {
					snake := diffSnake{
						PosA: posA,
						PosB: posA - k,
						Len:  nextX - posA,
						Diff: 2*hd - 1,
					}
					snake.setQuality(quality, revQuality[k+off])
					out = append(out, snake)
				}
			}
		}

		// reverse path
		for k := hd; k >= -hd; k -= 2 {
			// the reverse K diagonal
			kr := k + delta

			// Extend the furthest path from either K-1 or K+1.
			//
			// Note the following differences from above since we are on the
			// reverse path and moving top left on the grid:
			//
			// - The center diagonal is delta instead of K=0;
			// - When K is on the edge (-D or +D) the behaviour is different;
			// - Horizontal edges extend from K+1 and vertical from K-1;
			// - We want to minimize the horizontal position.
			var (
				nextX int
				fromK int
			)

			// Here we have a similar logic as the forward case, but the
			// diagonal cases are totally different since we are moving
			// in reverse.
			indexVert := kr - 1 + off
			indexHorz := kr + 1 + off
			switch k {
			case +hd:
				fromK = indexVert // note this is the K=0 case as well
			case -hd:
				fromK = indexHorz
			default:
				// Note that the signs here are also swapped, since we are
				// minimizing the X in this case. We are still favoring the
				// horizontal extensions though.
				if rev[indexVert] == rev[indexHorz]-1 {
					if isDiffQualityWorse(revQuality[indexHorz], revQuality[indexVert]) {
						fromK = indexVert
					} else {
						fromK = indexHorz
					}
				} else if rev[indexHorz] <= rev[indexVert] {
					fromK = indexHorz
				} else {
					fromK = indexVert
				}
			}

			// Apply the non-diagonal edge extension.
			quality := revQuality[fromK]
			if fromK == indexHorz {
				nextX = rev[indexHorz] - 1 // note the sign
				quality.moveHorz()
			} else {
				nextX = rev[indexVert]
				quality.moveVert()
			}

			// Extend the diagonal snake as far as possible.
			nextY := nextX - kr
			for nextX > 0 && nextY > 0 && a[nextX-1] == b[nextY-1] {
				quality.moveDiag()
				nextX--
				nextY--
			}

			rev[kr+off] = nextX
			revQuality[kr+off] = quality

			// Check for overlap with the forward D-paths computed above.
			if !odd && kr >= -hd && kr <= hd {
				if endA := fwd[kr+off]; endA >= nextX {
					snake := diffSnake{
						PosA: nextX,
						PosB: nextY,
						Len:  endA - nextX,
						Diff: 2 * hd,
					}
					snake.setQuality(fwdQuality[kr+off], quality)
					out = append(out, snake)
				}
			}
		}
	}

	// Between the optimal D-paths, sort by the ones with less segments.
	sort.Slice(out, func(i, j int) bool {
		return out[i].Segments < out[j].Segments
	})

	return
}

// Qualitative information about a diff used to heuristically sort LCS options.
type diffQuality struct {
	init bool // init flag
	segs int  // number of segments so far
	last int  // last segment direction
}

// Set the resulting snake "quality" based on the prefix and suffix quality.
func (snake *diffSnake) setQuality(prefix, suffix diffQuality) {
	snake.Segments = prefix.segs + suffix.segs
	if prefix.last == suffix.last && prefix.segs > 0 && suffix.segs > 0 {
		snake.Segments--
	}
}

// Used to heuristically decide the best quality diff.
func isDiffQualityWorse(horz, vert diffQuality) bool {
	horzSegs, vertSegs := horz.segs, vert.segs
	if horz.init && horz.last != 1 {
		horzSegs++
	}
	if vert.init && vert.segs != -1 {
		vertSegs++
	}
	if horzSegs > vertSegs {
		return true
	}
	return false
}

func (info *diffQuality) moveVert() {
	info.move(-1)
}

func (info *diffQuality) moveHorz() {
	info.move(+1)
}

func (info *diffQuality) moveDiag() {
	info.move(0)
}

func (info *diffQuality) move(direction int) {
	if !info.init {
		// the first move from an uninitialized diagonal should be ignored
		info.init = true
	} else {
		if direction != info.last || info.segs == 0 {
			info.segs++
			info.last = direction
		}
	}
}
