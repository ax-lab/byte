package bootstrap

import (
	"fmt"
	"os"
	"path/filepath"
	"syscall"
)

const (
	MainSourceFile  = "byte.go"
	BootstrapModule = "bootstrap"
)

// Performs the boot routine for the bootstrap process. This will check if
// the bootstrapper needs to be update and rebuild itself, restarting the
// process if that is the case.
func Boot() {
	root := ProjectDir()

	exeFile := getBootstrapExe()
	if exeFile == "" {
		return
	}

	rootFile := filepath.Join(root, MainSourceFile)
	rootStat := logBoot(os.Stat(rootFile))
	if rootStat == nil {
		return
	}

	exeStat := logBoot(os.Stat(exeFile))
	if exeStat == nil {
		return
	}

	exeTime := exeStat.ModTime()
	isNewer := rootStat.ModTime().After(exeTime)
	if !isNewer {
		rootDir := filepath.Join(filepath.Dir(rootFile), BootstrapModule)
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
		return
	}

	fmt.Printf("\n[bootstrap] rebuilding...\n")
	if !Run("[build] ", "go", "build", "-o", exeFile, rootFile) {
		logBootErr(fmt.Errorf("build failed"))
		return
	}

	files := make([]*os.File, 3)
	files[syscall.Stdin] = os.Stdin
	files[syscall.Stdout] = os.Stdout
	files[syscall.Stderr] = os.Stderr

	fmt.Printf("[bootstrap] restarting...\n")
	proc := logBoot(os.StartProcess(exeFile, os.Args, &os.ProcAttr{
		Dir:   ".",
		Env:   os.Environ(),
		Files: files,
	}))
	if proc == nil {
		return
	}

	state := logBoot(proc.Wait())
	if state == nil {
		return
	}

	os.Exit(state.ExitCode())
}

func getBootstrapExe() string {
	exeFile := logBoot(os.Executable())

	if exeFile != "" {
		exeFile = logBoot(filepath.EvalSymlinks(exeFile))
	}

	if exeFile != "" {
		exeFile = filepath.Clean(exeFile)
	}

	// check if we are running the bootstrap executable (e.g. not `go run`)
	if filepath.Dir(exeFile) != ProjectDir() {
		return ""
	}

	return exeFile
}

func logBoot[T any](val T, err error) T {
	if err != nil {
		location := Caller(1)
		logBootErr(fmt.Errorf("%s%v", location, err))
	}
	return val
}

func logBootErr(err any) {
	fmt.Fprintf(os.Stderr, "[bootstrap] error: %v\n", err)
}
