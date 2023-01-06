package main

import (
	"os"
	"path/filepath"

	"github.com/ax-lab/byte/bootstrap"
)

const BootstrapModuleDir = "bootstrap"
const ByteCargoDir = "byte-rs"

func main() {
	bootstrap.Boot()

	var cargoDir = filepath.Join(bootstrap.ProjectDir(), ByteCargoDir)
	var success = bootstrap.ExecInDir("[cargo]", cargoDir, func() bool {
		return bootstrap.Run("[cargo]", "cargo", "build", "--quiet")
	})

	if success {
		var exePath = filepath.Join(cargoDir, "target", "debug", bootstrap.Exe("byte"))
		bootstrap.Spawn(exePath, os.Args...)
	}
}
