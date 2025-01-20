package tests

import (
	"testing"
	. "json-serializer/src"
)

func TestSerialization(t *testing.T) {

	// json object
	input := Json{
		Type: Object,
		ObjectVal: map[string]Json{
			"name": {Type: String, StringVal: "Ricardo"},
			"age":  {Type: Number, NumberVal: 22},
		},
	}

	// serialize
	jsonStream := make(chan JC)
	done := make(chan string)
	go func() {
		SerializeJson(input, jsonStream)
		close(jsonStream)
		done <- "done"
	}()

	// deserialize
	var output Json
	go func() {
		output = DeserializeJson(jsonStream)
		done <- "done"
	}()

	// wait for goroutines to finish
	for i := 0; i < 2; i++ {
		<-done
	}

	// assertions
	if output.Type != input.Type {
		t.Errorf("Expected type %d, got %d", input.Type, output.Type)
	}
	if output.ObjectVal["name"].StringVal != input.ObjectVal["name"].StringVal {
		t.Errorf("Expected name %s, got %s", input.ObjectVal["name"].StringVal, output.ObjectVal["name"].StringVal)
	}
	if output.ObjectVal["age"].NumberVal != input.ObjectVal["age"].NumberVal {
		t.Errorf("Expected age %f, got %f", input.ObjectVal["age"].NumberVal, output.ObjectVal["age"].NumberVal)
	}
}
