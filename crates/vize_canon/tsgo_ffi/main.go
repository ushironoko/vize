package main

/*
#include <stdlib.h>

typedef struct {
    char* message;
    int line;
    int column;
    char* code;
    int severity; // 1 = error, 2 = warning
} TsgoDiagnostic;

typedef struct {
    TsgoDiagnostic* diagnostics;
    int count;
    char* error;
} TsgoCheckResult;
*/
import "C"

import (
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"unsafe"
)

// Diagnostic represents a TypeScript diagnostic
type Diagnostic struct {
	Message  string `json:"message"`
	Line     int    `json:"line"`
	Column   int    `json:"column"`
	Code     string `json:"code"`
	Severity int    `json:"severity"`
}

// CheckResult represents the result of type checking
type CheckResult struct {
	Diagnostics []Diagnostic `json:"diagnostics"`
	Error       string       `json:"error,omitempty"`
}

var diagnosticPattern = regexp.MustCompile(`^(.+)\((\d+),(\d+)\): (error|warning) (TS\d+): (.+)$`)

//export tsgo_check
func tsgo_check(content *C.char, filename *C.char) *C.char {
	goContent := C.GoString(content)
	goFilename := C.GoString(filename)

	result := checkTypeScript(goContent, goFilename)

	jsonBytes, err := json.Marshal(result)
	if err != nil {
		return C.CString(`{"error":"failed to marshal result"}`)
	}

	return C.CString(string(jsonBytes))
}

//export tsgo_free
func tsgo_free(ptr *C.char) {
	C.free(unsafe.Pointer(ptr))
}

func checkTypeScript(content, filename string) CheckResult {
	// Create temp file
	tmpDir := os.TempDir()
	tmpFile := filepath.Join(tmpDir, "vize-tsgo-"+strconv.Itoa(os.Getpid())+".ts")

	if err := os.WriteFile(tmpFile, []byte(content), 0644); err != nil {
		return CheckResult{Error: "failed to write temp file: " + err.Error()}
	}
	defer os.Remove(tmpFile)

	// Run tsgo
	cmd := exec.Command("tsgo",
		"--noEmit",
		"--skipLibCheck",
		"--strict",
		"--target", "ESNext",
		"--module", "ESNext",
		"--moduleResolution", "bundler",
		tmpFile,
	)

	output, _ := cmd.CombinedOutput()

	// Parse diagnostics
	diagnostics := parseDiagnostics(string(output), tmpFile, filename)

	return CheckResult{Diagnostics: diagnostics}
}

func parseDiagnostics(output, tmpFile, originalFile string) []Diagnostic {
	var diagnostics []Diagnostic

	lines := strings.Split(output, "\n")
	for _, line := range lines {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		matches := diagnosticPattern.FindStringSubmatch(line)
		if len(matches) == 7 {
			lineNum, _ := strconv.Atoi(matches[2])
			colNum, _ := strconv.Atoi(matches[3])
			severity := 1
			if matches[4] == "warning" {
				severity = 2
			}

			diagnostics = append(diagnostics, Diagnostic{
				Message:  matches[6],
				Line:     lineNum,
				Column:   colNum,
				Code:     matches[5],
				Severity: severity,
			})
		}
	}

	return diagnostics
}

func main() {
	// This is required for building as a C shared library
	// but will never be called
}
