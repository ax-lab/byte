package bootstrap

import (
	"encoding/json"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
)

const RegexpIgnoreCase = "(?i)"

var projectDir = (func() string {
	// assume we are in a directory under the project root
	dir := filepath.Dir(FileName())
	dir = filepath.Dir(dir)
	return dir
})()

// Returns the absolute root directory for the project.
func ProjectDir() string {
	return projectDir
}

// Returns the root path where to run cargo.
func CargoDir() string {
	return filepath.Join(ProjectDir(), CargoWorkspace)
}

// Returns the Go filename of the caller function.
func FileName() string {
	_, callerFile, _, hasInfo := runtime.Caller(1)
	if !hasInfo {
		log.Fatal("could not retrieve caller file name")
	}
	if !filepath.IsAbs(callerFile) {
		log.Fatal("caller file name is not an absolute path")
	}
	return filepath.Clean(callerFile)
}

func Caller(skip int) string {
	_, file, line, hasInfo := runtime.Caller(1 + skip)
	if hasInfo {
		return fmt.Sprintf("%s:%d: ", file, line)
	}
	return ""
}

func Glob(root, pattern string) (out []string) {
	re := regexp.MustCompile(RegexpIgnoreCase + GlobRegex(pattern) + "$")
	filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if !d.IsDir() && re.MatchString(path) {
			out = append(out, path)
		}
		return nil
	})
	return out
}

func GlobRegex(pattern string) string {
	var output []string

	next, runes := ' ', []rune(pattern)
	for len(runes) > 0 {
		next, runes = runes[0], runes[1:]
		switch next {
		case '/', '\\':
			output = append(output, `[/\\]`)
		case '?':
			output = append(output, `[^/\\]`)
		case '*':
			output = append(output, `[^/\\]*`)
		default:
			output = append(output, regexp.QuoteMeta(string(next)))
		}
	}
	return strings.Join(output, "")
}

func Relative(base, path string) string {
	fullbase, err := filepath.Abs(base)
	NoError(err, "getting absolute base path for relative")

	fullpath, err := filepath.Abs(path)
	NoError(err, "getting absolute path for relative")

	rel, err := filepath.Rel(fullbase, fullpath)
	NoError(err, "getting relative path")
	return rel
}

func WithExtension(filename string, ext string) string {
	out := strings.TrimSuffix(filename, filepath.Ext(filename))
	return out + ext
}

func ReadText(filename string) string {
	out, err := os.ReadFile(filename)
	if err != nil && !os.IsNotExist(err) {
		NoError(err, "reading file text")
	}
	return string(out)
}

func ReadJson(filename string, output any) any {
	data, err := os.ReadFile(filename)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		NoError(err, "reading JSON file")
	}

	if output == nil {
		output = &output
	}

	err = json.Unmarshal(data, output)
	NoError(err, "decoding JSON file")
	return output
}
