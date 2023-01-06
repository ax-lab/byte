package bootstrap

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"os/exec"
	"syscall"
)

// Spawn a new process "replaces" the current process by the given one.
//
// The new process shares the same environment and standard output streams
// as the current process.
//
// After the spawned process exits, the current process will exit with the
// same exit code.
func Spawn(name string, args ...string) error {
	files := make([]*os.File, 3)
	files[syscall.Stdin] = os.Stdin
	files[syscall.Stdout] = os.Stdout
	files[syscall.Stderr] = os.Stderr

	proc, err := os.StartProcess(name, args, &os.ProcAttr{
		Dir:   ".",
		Env:   os.Environ(),
		Files: files,
	})

	if err != nil {
		return fmt.Errorf("spawn: %v", err)
	}

	state, err := proc.Wait()
	if err != nil {
		return fmt.Errorf("spawn: wait failed: %v", err)
	}

	os.Exit(state.ExitCode())
	panic("unreachable")
}

// Run a new process handling errors and stderr output.
func Run(prefix, name string, args ...string) bool {
	var lf = []byte("\n")
	cmd := exec.Command(name, args...)

	stderr, err := cmd.StderrPipe()
	if err != nil {
		fmt.Fprintf(os.Stderr, "\n%sio error: %v\n\n", prefix, err)
		return false
	}

	if err = cmd.Start(); err != nil {
		fmt.Fprintf(os.Stderr, "\n%sstart error: %v\n\n", prefix, err)
		return false
	}

	errors, eol := false, true
	buffer := make([]byte, 4096)
	for {
		n, err := stderr.Read(buffer)
		if n > 0 {
			if !errors {
				os.Stderr.Write(lf)
			}

			errors = true
			for i, line := range bytes.Split(buffer[:n], lf) {
				if i > 0 {
					os.Stderr.Write(lf)
					eol = true
				}
				if eol && len(line) > 0 {
					os.Stderr.Write([]byte(prefix))
					os.Stderr.Write([]byte("err: "))
					eol = false
				}

				os.Stderr.Write(line)
			}
		}
		if err == io.EOF {
			break
		}
	}

	if err = cmd.Wait(); err != nil {
		fmt.Fprintf(os.Stderr, "\n%scommand error: %v\n\n", prefix, err)
		return false
	}

	if errors {
		fmt.Fprintf(os.Stderr, "\n%scommand run with errors\n\n", prefix)
		return false
	}

	return true
}
