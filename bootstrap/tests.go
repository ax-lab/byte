package bootstrap

import (
	"fmt"
	"path/filepath"
)

func RunScriptTest(scriptFileName string) (out ScriptTest) {
	out.File = scriptFileName
	out.Name = Relative(".", out.File)
	out.outputStartBanner()

	outFile := WithExtension(out.File, ".out")
	outJson := outFile + ".json"

	outputText := ReadText(outFile)
	outputJson := ReadJson(outJson, nil)

	if outputText == "" && outputJson == nil {
		fmt.Printf("... SKIP!\t(no test output file)\n")
		out.Success = true
		out.Skipped = true
		return
	}

	if outputText != "" && outputJson != nil {
		out.Error = fmt.Errorf("found both a text and JSON output")
		return
	}

	out.Expected = outputJson
	if outputText != "" {
		out.Expected = outputText
	}

	out.Directory = filepath.Dir(out.File)

	execOK := ExecInDir("test script", out.Directory, func() bool {
		runner := NewRunner(false)
		out.ExitCode, out.Error = runner.ExecScript(out.File,
			func(output string, isError bool) {
				if isError {
					out.StdErr += output
				} else {
					out.StdOut += output
				}
			},
		)
		return true
	})

	if !execOK && out.Error == nil {
		out.Error = fmt.Errorf("execute in script dir failed")
	}

	out.CheckResult()
	return out
}

type ScriptTest struct {
	Name    string
	File    string
	Error   error
	Success bool
	Skipped bool

	Directory string

	Expected any
	StdOut   string
	StdErr   string
	ExitCode int

	ExpectOutput []string
	ActualOutput []string
}

func (test *ScriptTest) CheckResult() {
	if test.StdErr != "" {
		test.Error = fmt.Errorf("test generated error output")
	}
	if test.ExitCode != 0 {
		test.Error = fmt.Errorf("test exited with code %d", test.ExitCode)
	}

	actualLines := Lines(test.StdOut)
	expectLines := []string{}
	switch expected := test.Expected.(type) {
	case string:
		expectLines = TrimLines(Lines(expected))
		actualLines = TrimLines(actualLines)
	case []any:
		for _, it := range expected {
			expectLines = append(expectLines, fmt.Sprint(it))
		}
	case []string:
		expectLines = expected
	default:
		test.Error = fmt.Errorf("invalid JSON output configuration: %T = %v", expected, expected)
	}

	success := test.Error == nil && len(actualLines) == len(expectLines)
	for i := 0; success && i < len(actualLines); i++ {
		success = actualLines[i] == expectLines[i]
	}

	test.ActualOutput = actualLines
	test.ExpectOutput = expectLines

	test.Success = success
	if test.Success {
		test.output("PASS!\n")
	} else if test.Error != nil {
		test.output("\n... ERROR: %v\n", test.Error)
	} else {
		test.output("FAIL!\n")
	}
}

func (test ScriptTest) OutputDetails() {
	if test.Success || test.Skipped || (test.Error != nil && test.StdErr == "") {
		return // nothing to output or we already output
	}

	test.output("\n==============================================\n")
	test.output("%s", test.Name)
	test.output("\n==============================================\n\n")

	if test.StdErr != "" && len(test.ActualOutput) == 0 {
		fmt.Println("  - No output")
	} else {
		diff := Compare(test.ActualOutput, test.ExpectOutput)
		test.output("  - Expected output differences:\n\n")
		for _, it := range diff.Blocks() {
			num := it.Dst
			sign, text, pos := " ", test.ExpectOutput, it.Dst
			if it.Kind > 0 {
				sign = "+"
			} else if it.Kind < 0 {
				num = it.Src
				sign, text, pos = "-", test.ActualOutput, it.Src
			}
			for i := 0; i < it.Len; i++ {
				line := text[i+pos]
				if line == "" {
					line = "⏎"
				}
				test.output("      %03d %s %s\n", num+i+1, sign, line)
			}
		}
	}

	if test.StdErr != "" {
		test.output("\n  - Error output:\n\n")
		for _, it := range TrimLines(Lines(test.StdErr)) {
			test.output("      %s\n", it)
		}
	}

	if test.ExitCode != 0 {
		test.output("\n  - Exited with code %d\n", test.ExitCode)
	}

	test.output("\n")
}

func (test ScriptTest) outputStartBanner() {
	test.output("\n>>> [TEST] %s...", test.Name)
}

func (test ScriptTest) output(msg string, args ...any) {
	fmt.Printf(msg, args...)
}
