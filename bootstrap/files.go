package bootstrap

import (
	"fmt"
	"log"
	"path/filepath"
	"runtime"
)

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
