package bootstrap_test

import (
	"encoding/json"
	"regexp"
	"testing"

	"github.com/ax-lab/byte/bootstrap"
	"github.com/stretchr/testify/require"
)

func TestGlobRegex(t *testing.T) {
	re := reMatcher{t: t}

	// literal matching
	re.SetPattern("abc")
	re.Match("abc")
	re.False("123")
	re.False("")

	// basic regexp escaping
	re.SetPattern(".")
	re.Match(".")
	re.False("!")

	// directory separator matching
	re.SetPattern("/")
	re.Match("/")
	re.Match("\\")

	re.SetPattern("a/b")
	re.Match("a/b")
	re.Match("a\\b")

	// windows directory separator matching
	re.SetPattern("\\")
	re.Match("/")
	re.Match("\\")

	re.SetPattern("a\\b")
	re.Match("a/b")
	re.Match("a\\b")

	// any char matching
	re.SetPattern("?")
	re.Match("?")
	re.Match("a")
	re.Match("b")
	re.SetPattern("a?c")
	re.Match("abc")
	re.Match("a c")
	re.Match("a-c")
	re.Match("a\tc")
	re.Match("a\nc")
	re.False("ac")

	re.False("a/c") // does not match directory separator
	re.False("a\\c")

	// glob match
	re.SetPattern("*")
	re.Match("")
	re.Match("a")
	re.Match("ab")
	re.Match("abc")

	re.SetPattern("a*c")
	re.Match("ac")
	re.Match("abc")
	re.Match("abbc")
	re.Match("a\nc")

	re.False("a/c") // does not match directory separator
	re.False("a\\c")
	re.False("ab/bc")
	re.False("ab\\bc")

	// unicode support
	re.SetPattern("[?]")
	re.Match("[a]")
	re.Match("[滅]")
	re.False("[滅多]")

	re.SetPattern("[??]")
	re.Match("[日本]")
}

type reMatcher struct {
	t   *testing.T
	re  *regexp.Regexp
	pat string
}

func (m reMatcher) Match(input string) {
	require.True(m.t, m.re.MatchString(input),
		"expected \"%s\" to match input %s", m.pat, m.debugString(input))
}

func (m reMatcher) False(input string) {
	require.False(m.t, m.re.MatchString(input),
		"expected \"%s\" NOT to match input %s", m.pat, m.debugString(input))
}

func (m reMatcher) debugString(input string) string {
	v, _ := json.Marshal(input)
	return string(v)
}

func (m *reMatcher) SetPattern(pattern string) {
	m.pat = pattern
	m.re = regexp.MustCompile("^" + bootstrap.GlobRegex(pattern) + "$")
}
