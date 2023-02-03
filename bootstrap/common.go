package bootstrap

import (
	"fmt"
	"os"
	"regexp"
	"runtime"
)

func ExeName(name string) string {
	if runtime.GOOS == "windows" {
		return name + ".exe"
	}
	return name
}

func NoError(err error, msg string) {
	if err != nil {
		fmt.Fprintf(os.Stderr, "\nfatal error: %s - %v\n\n", msg, err)
		os.Exit(3)
	}
}

func MatchesPattern(input, pattern string) bool {
	re := regexp.MustCompile(RegexpIgnoreCase + GlobRegex(pattern))
	return re.MatchString(input)
}
