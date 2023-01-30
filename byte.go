package main

import (
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"

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

		var verbArgs []string
		if len(args) > 1 {
			verb, verbArgs = args[1], args[2:]
		}

		var byte = bootstrap.NewRunner(release)
		switch verb {
		case "test":
			tests := filepath.Join(bootstrap.ProjectDir(), "tests")
			files := bootstrap.Glob(tests, "*.by")
			for _, it := range files {
				matches := len(verbArgs) == 0
				for _, pattern := range verbArgs {
					matches = matches || bootstrap.MatchesPattern(it, pattern)
				}

				if !matches {
					continue
				}

				fmt.Printf("\n>>> %s\n", bootstrap.Relative(".", it))
				input, err := ioutil.ReadFile(it)
				bootstrap.NoError(err, "reading input")

				text := string(input)
				if len(text) == 0 {
					text = "(empty file)"
				} else if last := len(text) - 1; text[last] == '\n' {
					text = text[:last]
				} else {
					text = text + "¶"
				}
				fmt.Printf("\n%s\n", text)

				fmt.Printf("\n---- RUNNING ----\n\n")
				runner := bootstrap.NewRunner(false)
				code, err := runner.ExecScript(it, func(output string, isError bool) {
					if isError {
						os.Stderr.WriteString(output)
					} else {
						os.Stdout.WriteString(output)
					}
				})
				fmt.Printf("\n---- RESULTS ----\n")
				if err != nil {
					fmt.Printf("\nERROR: %v\n", err)
				} else {
					fmt.Printf("\nEXIT: %d\n", code)
				}
			}
			fmt.Println()
		default:
			byte.Spawn(args...)
		}
	}
}
