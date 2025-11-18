package runner

import (
	"encoding/json"
)

// TestCase represents a single test case
type TestCase struct {
	ID          string          `json:"id"`
	Description string          `json:"description,omitempty"`
	Method      string          `json:"method"`
	Params      json.RawMessage `json:"params"`
	Expected    TestExpectation `json:"expected"`
}

// TestExpectation defines what response is expected
type TestExpectation struct {
	Success *json.RawMessage `json:"success,omitempty"` // Expected successful result
	Error   *Error           `json:"error,omitempty"`   // Expected error
}

// TestSuite represents a collection of test cases
type TestSuite struct {
	Name        string     `json:"name"`
	Description string     `json:"description,omitempty"`
	Tests       []TestCase `json:"tests"`
}

// Request represents a request sent to the handler
type Request struct {
	ID     string          `json:"id"`
	Method string          `json:"method"`
	Params json.RawMessage `json:"params"`
}

// Response represents a response from the handler
type Response struct {
	ID      string           `json:"id"`
	Success *json.RawMessage `json:"success,omitempty"`
	Error   *Error           `json:"error,omitempty"`
}

// Error represents an error response
type Error struct {
	Type    string `json:"type"`
	Variant string `json:"variant,omitempty"`
}
