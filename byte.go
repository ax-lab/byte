package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"syscall"

	"github.com/ax-lab/byte/bootstrap"
)

const BootstrapModuleDir = "bootstrap"

func main() {
	err := rebuildCurrentExe()
	if err != nil {
		fmt.Fprintf(os.Stderr, "[bootstrap] warning: failed to rebuild bootstrap -- %v\n", err)
	}

	fmt.Println()
	bootstrap.About()
	fmt.Println("ARGS:", os.Args)
	fmt.Println()
}

// Quick and dirty solution to rebuild the main executable when the source
// file changes.
func rebuildCurrentExe() (err error) {
	_, rootFile, _, hasInfo := runtime.Caller(0)
	if !hasInfo {
		return nil
	} else if rootFile, err = filepath.Abs(rootFile); err != nil {
		return fmt.Errorf("filepath.Abs: %v", err)
	}

	exeFile, err := os.Executable()
	if err != nil {
		return fmt.Errorf("os.Executable: %v", err)
	}

	if exeFile, err = filepath.EvalSymlinks(exeFile); err != nil {
		return fmt.Errorf("filepath.EvalSymlinks: %v", err)
	}

	rootFile = filepath.Clean(rootFile)
	exeFile = filepath.Clean(exeFile)
	exePath := filepath.Dir(exeFile)

	if !strings.HasPrefix(rootFile, exePath) {
		// not running the bootstrap executable
		return nil
	}

	statFile, err := os.Stat(rootFile)
	if err != nil {
		return fmt.Errorf("os.Stat: %v", err)
	}

	statExe, err := os.Stat(exeFile)
	if err != nil {
		return fmt.Errorf("os.Stat: %v", err)
	}

	exeTime := statExe.ModTime()
	isNewer := statFile.ModTime().After(exeTime)
	if !isNewer {
		rootDir := filepath.Join(filepath.Dir(rootFile), BootstrapModuleDir)
		filepath.WalkDir(rootDir, func(path string, dir os.DirEntry, err error) error {
			if isNewer {
				return filepath.SkipDir
			} else if err != nil {
				return err
			} else if info, err := dir.Info(); err == nil {
				isNewer = isNewer || info.ModTime().After(exeTime)
			} else {
				return err
			}
			return nil
		})
	}

	if !isNewer {
		return nil
	}

	fmt.Printf("\n[bootstrap] rebuilding...\n")
	cmd := exec.Command("go", "build", "-o", exePath, rootFile)
	if err = cmd.Run(); err != nil {
		return fmt.Errorf("go build: %v", err)
	}

	files := make([]*os.File, 3)
	files[syscall.Stdin] = os.Stdin
	files[syscall.Stdout] = os.Stdout
	files[syscall.Stderr] = os.Stderr

	fmt.Printf("[bootstrap] restarting...\n")
	proc, err := os.StartProcess(exeFile, os.Args, &os.ProcAttr{
		Dir:   ".",
		Env:   os.Environ(),
		Files: files,
	})
	if err != nil {
		return err
	}

	state, err := proc.Wait()
	if err != nil {
		return err
	}

	os.Exit(state.ExitCode())
	return fmt.Errorf("unreachable")
}
