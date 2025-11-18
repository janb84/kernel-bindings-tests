package main

import (
	"flag"
	"fmt"
	"io/fs"
	"os"
	"strings"

	"github.com/stringintech/kernel-bindings-tests/runner"
	"github.com/stringintech/kernel-bindings-tests/testdata"
)

func main() {
	handlerPath := flag.String("handler", "", "Path to handler binary")
	flag.Parse()

	if *handlerPath == "" {
		fmt.Fprintf(os.Stderr, "Error: -handler flag is required\n")
		flag.Usage()
		os.Exit(1)
	}

	// Collect embedded test files
	testFiles, err := fs.Glob(testdata.FS, "*.json")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error finding test files: %v\n", err)
		os.Exit(1)
	}

	if len(testFiles) == 0 {
		fmt.Fprintf(os.Stderr, "No test files found\n")
		os.Exit(1)
	}

	// Run tests
	totalPassed := 0
	totalFailed := 0
	totalTests := 0

	for _, testFile := range testFiles {
		fmt.Printf("\n=== Running test suite: %s ===\n", testFile)

		// Load test suite from embedded FS
		suite, err := runner.LoadTestSuiteFromFS(testdata.FS, testFile)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error loading test suite: %v\n", err)
			continue
		}

		// Create test runner
		testRunner, err := runner.NewTestRunner(*handlerPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error creating test runner: %v\n", err)
			continue
		}

		// Run suite
		result := testRunner.RunTestSuite(*suite)
		testRunner.Close()

		printResults(result)

		totalPassed += result.PassedTests
		totalFailed += result.FailedTests
		totalTests += result.TotalTests
	}

	fmt.Printf("\n" + strings.Repeat("=", 60) + "\n")
	fmt.Printf("TOTAL SUMMARY\n")
	fmt.Printf(strings.Repeat("=", 60) + "\n")
	fmt.Printf("Total Tests: %d\n", totalTests)
	fmt.Printf("Passed:      %d\n", totalPassed)
	fmt.Printf("Failed:      %d\n", totalFailed)
	fmt.Printf(strings.Repeat("=", 60) + "\n")

	if totalFailed > 0 {
		os.Exit(1)
	}
}

func printResults(result runner.TestResult) {
	fmt.Printf("\nTest Suite: %s\n", result.SuiteName)
	fmt.Printf("Total: %d, Passed: %d, Failed: %d\n\n", result.TotalTests, result.PassedTests, result.FailedTests)

	for _, tr := range result.TestResults {
		status := "✓"
		if !tr.Passed {
			status = "✗"
		}
		fmt.Printf("  %s %s: %s\n", status, tr.TestID, tr.Message)
	}

	fmt.Printf("\n")
}
