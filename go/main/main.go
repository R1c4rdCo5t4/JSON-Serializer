package main

import (
	"fmt"
	. "json-serializer/src"
)

func main() {

	// json object
	jsonObject := Json{
		Type: Object,
		ObjectVal: map[string]Json{
			"socialProfiles": {
				Type: Array,
				ArrayVal: []Json{{
                    Type: Object,
                    ObjectVal: map[string]Json{
                        "name": {Type: String, StringVal: "Twitter"},
                    },
                },
                {
                    Type: Object,
                    ObjectVal: map[string]Json{
                        "name": {Type: String, StringVal: "Facebook"},
                    },
                }},
			},
		},
	}

	// ."socialProfiles" map ."name" end
	accessor := &Accessor{
		Field:"socialProfiles",
		Sub: &Accessor{
			Map: true,
			Sub: &Accessor{
				Field: "name",
				Sub: &Accessor{
					End: true,
				},
			},
		},
	}

	// create channels
	jsonStream := make(chan JC)
	transformedStream := make(chan JC) 
	done := make(chan string)

	// launch goroutines
	go func() {
		SerializeJson(jsonObject, jsonStream)
		close(jsonStream)
	}()

	go func() {
		Eval(accessor, jsonStream, transformedStream)
		close(transformedStream)
	}()

	go func() {
		result := DeserializeJson(transformedStream)
		fmt.Printf("Result: %+v\n", result)
		printJson(result)
		fmt.Println()
		done <- "done"
	}()

	<-done // wait for goroutines to finish
}

// function to print json object
func printJson(json Json) {
	switch json.Type {
        case Null:
            fmt.Print("null")
        case Bool:
            fmt.Printf("%v", json.BoolVal)
        case Number:
            fmt.Printf("%v", json.NumberVal)
        case String:
            fmt.Printf("%q", json.StringVal)
        case Array:
            fmt.Print("[")
            for i, v := range json.ArrayVal {
                if i > 0 {
                    fmt.Print(", ")
                }
                printJson(v)
            }
            fmt.Print("]")
        case Object:
            fmt.Print("{")
            first := true
            for k, v := range json.ObjectVal {
                if !first {
                    fmt.Print(", ")
                }
                first = false
                fmt.Printf("%q: ", k)
                printJson(v)
            }
            fmt.Print("}")
	}
}
