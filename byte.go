package main

import (
	"fmt"
	"os"

	"github.com/ax-lab/byte/bootstrap"
)

const ()

func main() {
	bootstrap.Boot("byte.go")

	var cargoDir = bootstrap.CargoDir()
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
		var (
			args = os.Args
			verb = ""
		)

		if len(args) > 1 {
			verb = args[1]
		}

		var byte = bootstrap.NewRunner(release)
		switch verb {
		default:
			byte.Spawn(args...)
		}
	}
}
