package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/ax-lab/byte/bootstrap"
)

const BootstrapModuleDir = "bootstrap"
const ByteCargoDir = "byte-rs"

func main() {
	bootstrap.Boot()

	var cargoDir = filepath.Join(bootstrap.ProjectDir(), ByteCargoDir)

	var (
		clean   = os.Getenv("CLEAN") != ""
		release = os.Getenv("RELEASE") != ""
	)

	if clean {
		fmt.Printf("\nCleaning up source files...\n\n")
	}

	var success = bootstrap.ExecInDir("[cargo]", cargoDir, func() bool {
		if clean {
			return bootstrap.Run("[cargo]", "cargo", "clean")
		}
		args := []string{"build", "--quiet"}
		if release {
			args = append(args, "--release")
		}

		return bootstrap.Run("[cargo]", "cargo", args...)
	})

	if clean {
		return
	}

	if success {
		var outputDir = "debug"
		if release {
			outputDir = "release"
		}
		var exePath = filepath.Join(cargoDir, "target", outputDir, bootstrap.Exe("byte"))
		if err := bootstrap.Spawn(exePath, os.Args...); err != nil {
			fmt.Fprintf(os.Stderr, "\n[bootstrap] error: starting byte: %v\n\n", err)
			os.Exit(123)
		}
	}
}
