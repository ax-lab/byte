package bootstrap

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"os/exec"
)

var lf = []byte("\n")

func Run(prefix, name string, args ...string) bool {
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
