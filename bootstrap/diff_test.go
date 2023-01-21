package bootstrap_test

import (
	"testing"

	"github.com/ax-lab/byte/bootstrap"
	"github.com/stretchr/testify/require"
)

func TestDiffEqual(t *testing.T) {
	test := require.New(t)

	var a, b []string

	// Diff of empty
	test.Equal("", bootstrap.Compare(a, b).String())

	// Diff for equal values should be empty
	a = []string{"a", "b", "c"}
	b = []string{"a", "b", "c"}
	test.Equal("=[3] 0 -> 0", bootstrap.Compare(a, b).String())
}

func TestDiffTrivial(t *testing.T) {
	test := require.New(t)

	var a, b []string

	// empty A or B

	a = []string{}
	b = []string{"a", "b", "c"}
	test.Equal(
		[]string{"+a", "+b", "+c"},
		bootstrap.Compare(a, b).Text(),
	)

	a = []string{"a", "b", "c"}
	b = []string{}
	test.Equal(
		[]string{"-a", "-b", "-c"},
		bootstrap.Compare(a, b).Text(),
	)

	// same prefix

	a = []string{"a"}
	b = []string{"a", "b", "c"}
	test.Equal(
		[]string{" a", "+b", "+c"},
		bootstrap.Compare(a, b).Text(),
	)

	a = []string{"a", "b", "c"}
	b = []string{"a"}
	test.Equal(
		[]string{" a", "-b", "-c"},
		bootstrap.Compare(a, b).Text(),
	)

	// same sufix

	a = []string{"c"}
	b = []string{"a", "b", "c"}
	test.Equal(
		[]string{"+a", "+b", " c"},
		bootstrap.Compare(a, b).Text(),
	)

	a = []string{"a", "b", "c"}
	b = []string{"c"}
	test.Equal(
		[]string{"-a", "-b", " c"},
		bootstrap.Compare(a, b).Text(),
	)

	// same infix

	a = []string{"b"}
	b = []string{"a", "b", "c"}
	test.Equal(
		[]string{"+a", " b", "+c"},
		bootstrap.Compare(a, b).Text(),
	)

	a = []string{"a", "b", "c"}
	b = []string{"b"}
	test.Equal(
		[]string{"-a", " b", "-c"},
		bootstrap.Compare(a, b).Text(),
	)
}

func TestDiffLCS(t *testing.T) {
	test := require.New(t)

	var a, b []string

	a = []string{"a", "b", "c", "a", "b", "b", "a"}
	b = []string{"c", "b", "a", "b", "a", "c"}
	diff := bootstrap.Compare(a, b)
	test.Equal(
		[]string{"-a", "-b", " c", "+b", " a", " b", "-b", " a", "+c"},
		diff.Text(),
	)
}
