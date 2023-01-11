package bootstrap

import (
	"fmt"
	"os"
	"path/filepath"
)

type Runner struct {
	exe string
}

func NewRunner(release bool) Runner {
	var cargoDir = CargoDir()
	var outputDir = "debug"
	if release {
		outputDir = "release"
	}

	var exePath = filepath.Join(cargoDir, "target", outputDir, ExeName("byte"))
	return Runner{
		exe: exePath,
	}
}

func (runner Runner) Spawn(args ...string) {
	if err := Spawn(runner.exe, args...); err != nil {
		fmt.Fprintf(os.Stderr, "\n[bootstrap] error: starting byte: %v\n\n", err)
		os.Exit(123)
	}
}
