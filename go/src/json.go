package src

import "fmt"

func SerializeJson(value Json, out chan<- JC) {
	switch value.Type {
	case Null:
		out <- JC{Type: JCNull}
	case Bool:
		out <- JC{Type: JCBool, BoolVal: value.BoolVal}
	case Number:
		out <- JC{Type: JCNumber, NumberVal: value.NumberVal}
	case String:
		out <- JC{Type: JCString, StringVal: value.StringVal}
	case Array:
		out <- JC{Type: JCBeginArray}
		for _, elem := range value.ArrayVal {
			subChan := make(chan JC)
			out <- JC{Type: JCElement, Sub: subChan}

			// serialize element in separate goroutine
			go func(e Json, ch chan JC) {
				SerializeJson(e, ch)
				close(ch)
			}(elem, subChan)
		}
		out <- JC{Type: JCEndArray}
	case Object:
		out <- JC{Type: JCBeginObject}
		for key, val := range value.ObjectVal {
			subChan := make(chan JC)
			out <- JC{Type: JCElement, Sub: subChan}

			// serialize key value pair in separate goroutine
			go func(k string, v Json, ch chan JC) {
				ch <- JC{Type: JCString, StringVal: k} // key
				SerializeJson(v, ch)                   // value
				close(ch)
			}(key, val, subChan)
		}
		out <- JC{Type: JCEndObject}
	default:
		panic(fmt.Sprintf("Unexpected json type: %v", value.Type))
	}
}

func DeserializeJson(in <-chan JC) Json {
	token, ok := <-in
	if !ok {
		panic("Unexpected end of input while deserializing json")
	}
	switch token.Type {
	case JCNull:
		return Json{Type: Null}
	case JCBool:
		return Json{Type: Bool, BoolVal: token.BoolVal}
	case JCNumber:
		return Json{Type: Number, NumberVal: token.NumberVal}
	case JCString:
		return Json{Type: String, StringVal: token.StringVal}
	case JCBeginArray:
		array := []Json{}
		for { // loop until end of array
			token := <-in
			if token.Type == JCEndArray {
				break
			}
			if token.Type == JCElement {
				array = append(array, DeserializeJson(token.Sub)) // deserialize element and add it to array
			}
		}
		return Json{Type: Array, ArrayVal: array}
	case JCBeginObject:
		obj := make(map[string]Json)
		for { // loop until end of object
			token := <-in
			if token.Type == JCEndObject {
				break
			}
			if token.Type == JCElement {
				key := (<-token.Sub).StringVal      // key is always a string
				value := DeserializeJson(token.Sub) // deserialize value
				obj[key] = value                    // add key value pair to object
			}
		}
		return Json{Type: Object, ObjectVal: obj}
	case JCElement:
		// deserialize element in subchannel
		return DeserializeJson(token.Sub)
	default:
		panic(fmt.Sprintf("Unexpected token type: %v", token.Type))
	}
}

func Eval(accessor *Accessor, in <-chan JC, out chan<- JC) {
	if accessor.End {
		token := <-in
		switch token.Type {
		case JCBeginArray:
			out <- token
			for { // loop until end of array
				token := <-in
				if token.Type == JCEndArray {
					out <- token
					return
				}
				out <- token
				if token.Type == JCElement {
					for t := range token.Sub {
						out <- t
					}
				}
			}
		case JCBeginObject:
			out <- token
			for { // loop until end of object
				token := <-in
				if token.Type == JCEndObject {
					out <- token
					return
				}
				out <- token
				if token.Type == JCElement {
					for t := range token.Sub {
						out <- t
					}
				}
			}
		default: // single value
			out <- token
		}
		return // we are done here
	}

	token := <-in
	switch {
	case accessor.Field != "": // field access
		if token.Type != JCBeginObject {
			panic("Expected object for field accessor")
		}
		for {
			token := <-in
			if token.Type == JCEndObject {
				break
			}
			if token.Type == JCElement {
				key := (<-token.Sub).StringVal
				if key == accessor.Field {
					Eval(accessor.Sub, token.Sub, out) // evaluate sub accessor
					break
				} else {
					// consume unused channel
					for range token.Sub {
					}
				}
			}
		}
		// consume remaining tokens
		for range in {
		}

	case accessor.Map: // map operation
		if token.Type != JCBeginArray {
			panic("Expected array for map accessor")
		}
		out <- JC{Type: JCBeginArray}
		for {
			token := <-in
			if token.Type == JCEndArray {
				out <- JC{Type: JCEndArray}
				break
			}
			if token.Type == JCElement {
				subChan := make(chan JC)
				out <- JC{Type: JCElement, Sub: subChan}

				// evaluate sub accessor in separate goroutine
				go func() {
					Eval(accessor.Sub, token.Sub, subChan)
					close(subChan)
				}()
			}
		}

	case accessor.Index >= 0: // array index access
		if token.Type != JCBeginArray {
			panic("Expected array for index accessor")
		}
		currentIndex := 0
		for {
			token := <-in
			if token.Type == JCEndArray {
				break
			}
			if token.Type == JCElement {
				if currentIndex == accessor.Index {
					Eval(accessor.Sub, token.Sub, out)
					break
				}
				// consume unused channel
				for range token.Sub {
				}
				currentIndex++
			}
		}
		// consume remaining tokens
		for range in {
		}
	default:
		panic(fmt.Sprintf("Unexpected accessor: %v", accessor))
	}
}
