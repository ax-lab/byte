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
	lcs := LCS(input, output)
	lcs = append(lcs, CommonSequence{PosA: len(input), PosB: len(output), Len: 0})

	srcLast, dstLast := 0, 0
	for _, it := range lcs {
		src, dst := it.PosA, it.PosB
		if del := src - srcLast; del > 0 {
			out = append(out, DiffBlock{
				Kind: -1,
				Src:  srcLast,
				Dst:  dst,
				Len:  del,
			})
		}
		if ins := dst - dstLast; ins > 0 {
			out = append(out, DiffBlock{
				Kind: +1,
				Src:  src,
				Dst:  dstLast,
				Len:  ins,
			})
		}

		if it.Len > 0 {
			if count := len(out); count > 0 && out[count-1].Kind == 0 {
				out[count-1].Len += it.Len
			} else {
				out = append(out, DiffBlock{
					Kind: 0,
					Src:  src,
					Dst:  dst,
					Len:  it.Len,
				})
			}
		}

		srcLast, dstLast = src+it.Len, dst+it.Len
	}

	return out
}

type CommonSequence struct {
	PosA int
	PosB int
	Len  int
}

// Compute the Longest Common Subsequence (LCS) of A and B.
func LCS[T comparable](a []T, b []T) (out []CommonSequence) {
	return computeLCS(a, 0, len(a), b, 0, len(b))
}

// Compute the LCS of a sub-sequence of A and B using Myers algorithm.
func computeLCS[T comparable](a []T, ax, ay int, b []T, bx, by int) (out []CommonSequence) {
	if ay-ax <= 0 || by-bx <= 0 {
		return nil
	}

	n := ay - ax
	m := by - bx

	// Find the middle "snake" diagonal in the edit path. See the function for
	// details.
	//
	// TL;DR: the snake is a (possibly empty) common sequence of A and B that
	// can be used to evenly split the edit path terms of D.
	ls := diffFindMidSnakes(a[ax:ay], b[bx:by])
	lastSegments := 0

	// Compute the LCS for each of the candidate snakes. They all will yield
	// the longest possible sequence, but some are better than others when
	// using for a diff.
	for i, mid := range ls {
		// Only consider a LCS producing the least diff segments
		if i > 0 && mid.Segments > lastSegments {
			break
		}
		lastSegments = mid.Segments

		midA := mid.PosA + ax
		midB := mid.PosB + bx

		var lcs []CommonSequence
		if mid.Diff > 1 {
			// prefix LCS
			lcs = computeLCS(a, ax, midA, b, bx, midB)

			if mid.Len > 0 {
				// add the mid-snake as a common sequence
				lcs = append(lcs, CommonSequence{PosA: midA, PosB: midB, Len: mid.Len})
			}

			// suffix LCS
			lcs = append(lcs, computeLCS(a, midA+mid.Len, ay, b, midB+mid.Len, by)...)
		} else {
			// If D is 1 then there is a single element added/removed from one of
			// the sequences. In that case the LCS is the shorter sequence.
			//
			// The same logic trivially works when D is zero (equal sequences).
			if n < m {
				if a[ax] == b[bx] {
					lcs = append(lcs, CommonSequence{PosA: ax, PosB: bx, Len: n})
				} else {
					lcs = append(lcs, CommonSequence{PosA: ax, PosB: bx + 1, Len: n})
				}
			} else {
				if a[ax] == b[bx] {
					lcs = append(lcs, CommonSequence{PosA: ax, PosB: bx, Len: m})
				} else {
					lcs = append(lcs, CommonSequence{PosA: ax + 1, PosB: bx, Len: m})
				}
			}
		}

		// Check if the LCS we generated is better than the one we got so far:
		use := i == 0
		use = use || diffCompare(lcs, out, n, m) < 0

		if use {
			out = lcs
		}
	}

	return out
}

func diffCompare(d1, d2 []CommonSequence, lenA, lenB int) int {
	return 0
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
			posX := nextX
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
						PosA: posX,
						PosB: posX - k,
						Len:  nextX - posX,
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
			posX := nextX
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
						Len:  posX - nextX,
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
