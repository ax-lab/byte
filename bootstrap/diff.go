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
	lcs := computeLCS(input, output)
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

func computeLCS[T comparable](a, b []T) (out [][3]int) {
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
