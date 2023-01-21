package bootstrap

import (
	"fmt"
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
