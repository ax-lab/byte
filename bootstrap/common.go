package bootstrap

import "runtime"

func ExeName(name string) string {
	if runtime.GOOS == "windows" {
		return name + ".exe"
	}
	return name
}
