package runner

import (
	"bufio"
	"errors"
	"fmt"
	"os"
	"testing"
	"time"
)

const (
	// envTestAsSubprocess signals the binary to run as a subprocess helper.
	envTestAsSubprocess = "TEST_AS_SUBPROCESS"

	// envTestHelperName specifies which helper function to execute in subprocess mode.
	envTestHelperName = "TEST_HELPER_NAME"

	helperNameNormal       = "normal"
	helperNameUnresponsive = "unresponsive"
)

// testHelpers maps helper names to functions that simulate different handler behaviors.
var testHelpers = map[string]func(){
	helperNameNormal:       helperNormal,
	helperNameUnresponsive: helperUnresponsive,
}

// TestMain allows the test binary to serve two purposes:
// 1. Normal mode: runs tests when TEST_AS_SUBPROCESS != "1"
// 2. Subprocess mode: executes a helper function when TEST_AS_SUBPROCESS == "1"
//
// This enables tests to spawn the binary itself as a mock handler subprocess,
// avoiding the need for separate test fixture binaries.
func TestMain(m *testing.M) {
	if os.Getenv(envTestAsSubprocess) != "1" {
		// Run tests normally
		os.Exit(m.Run())
	}

	// Run as subprocess helper based on which helper is requested
	helperName := os.Getenv(envTestHelperName)
	if helper, ok := testHelpers[helperName]; ok {
		helper()
	} else {
		fmt.Fprintf(os.Stderr, "Unknown test helper: %s\n", helperName)
		os.Exit(1)
	}
}

// TestHandler_NormalOperation tests that a well-behaved handler works correctly
func TestHandler_NormalOperation(t *testing.T) {
	h, err := newHandlerForTest(t, helperNameNormal, 0)
	if err != nil {
		t.Fatalf("Failed to create handler: %v", err)
	}
	defer h.Close()

	// Send a request to the handler
	request := `{"id":1,"method":"test"}`
	if err := h.SendLine([]byte(request)); err != nil {
		t.Fatalf("Failed to send request: %v", err)
	}

	// Read the response
	line, err := h.ReadLine()
	if err != nil {
		t.Fatalf("Failed to read line: %v", err)
	}

	expected := `{"id":1,"result":true}`
	if string(line) != expected {
		t.Errorf("Expected %q, got %q", expected, string(line))
	}
}

// helperNormal simulates a normal well-behaved handler that reads a request,
// validates it, and sends a response.
func helperNormal() {
	// Read requests from stdin and respond with expected results
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		request := scanner.Text()
		expected := `{"id":1,"method":"test"}`
		if request != expected {
			fmt.Fprintf(os.Stderr, "Expected request %q, got %q\n", expected, request)
			os.Exit(1)
		}
		fmt.Println(`{"id":1,"result":true}`)
	}
}

// TestHandler_Unresponsive tests that the runner correctly handles an unresponsive handler
func TestHandler_Unresponsive(t *testing.T) {
	h, err := newHandlerForTest(t, helperNameUnresponsive, 100*time.Millisecond)
	if err != nil {
		t.Fatalf("Failed to create handler: %v", err)
	}
	defer h.Close()

	// Send a request to the handler
	request := `{"id":1,"method":"test"}`
	if err := h.SendLine([]byte(request)); err != nil {
		t.Fatalf("Failed to send request: %v", err)
	}

	// Try to read the response - should Timeout
	start := time.Now()
	_, err = h.ReadLine()
	elapsed := time.Since(start)

	if err == nil {
		t.Fatal("Expected error from unresponsive handler, got nil")
	}

	// Verify it's the Timeout error we expect
	if !errors.Is(err, ErrHandlerTimeout) {
		t.Errorf("Expected ErrHandlerTimeout, got: %v", err)
	}

	// Verify Timeout happened quickly (within reasonable margin)
	if elapsed > 200*time.Millisecond {
		t.Errorf("Timeout took too long: %v (expected ~100ms)", elapsed)
	}
}

// helperUnresponsive simulates a handler that receives requests but never responds,
// triggering the Timeout mechanism in the runner.
func helperUnresponsive() {
	// Read from stdin to prevent broken pipe, but never write responses
	scanner := bufio.NewScanner(os.Stdin)
	for scanner.Scan() {
		// Sleep indefinitely to simulate unresponsiveness
		time.Sleep(1 * time.Hour)
	}
}

// newHandlerForTest creates a Handler that runs a test helper as a subprocess.
// The helperName identifies which helper to run (e.g., "normal", "crash", "hang").
// The timeout parameter sets the per-request timeout (0 uses default).
func newHandlerForTest(t *testing.T, helperName string, timeout time.Duration) (*Handler, error) {
	t.Helper()

	return NewHandler(&HandlerConfig{
		Path:    os.Args[0],
		Env:     []string{"TEST_AS_SUBPROCESS=1", "TEST_HELPER_NAME=" + helperName},
		Timeout: timeout,
	})
}
