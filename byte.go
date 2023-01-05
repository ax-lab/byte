package main

import (
	"fmt"
	"os"

	"github.com/ax-lab/byte/bootstrap"
)

const BootstrapModuleDir = "bootstrap"
const ByteCargoDir = "byte-rs"

func main() {
	bootstrap.Boot()

	fmt.Println()
	fmt.Println("ROOT:", bootstrap.ProjectDir())
	fmt.Println()
	fmt.Println("ARGS:", os.Args)
	fmt.Println()
}

func buildAndRun() {

}
