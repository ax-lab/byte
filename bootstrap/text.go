package bootstrap

import (
	"regexp"
	"strings"
	"unicode"
)

func Lines(input string) []string {
	re := regexp.MustCompile(`\r\n?|\n`)
	return re.Split(input, -1)
}

func TrimLines(lines []string) []string {
	for i, it := range lines {
		lines[i] = strings.TrimRightFunc(it, unicode.IsSpace)
	}

	for len(lines) > 0 && lines[len(lines)-1] == "" {
		lines = lines[:len(lines)-1]
	}

	return lines
}
