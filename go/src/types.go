package src

type JsonType int
type JCType int

// represent the possible types of a json
const (
	Null JsonType = iota
	Bool
	Number
	String
	Array
	Object
)

// represent the tokens of a json that are sent over the channel
const (
	JCNull JCType = iota
	JCBool
	JCNumber
	JCString
	JCBeginArray
	JCEndArray
	JCBeginObject
	JCEndObject
	JCElement
)

// json object
type Json struct {
	Type      JsonType
	BoolVal   bool
	NumberVal float64
	StringVal string
	ArrayVal  []Json 
	ObjectVal map[string]Json
}

// json channel
type JC struct {
	Type      JCType
	BoolVal   bool
	NumberVal float64
	StringVal string
	Sub       chan JC // subchannel for nested elements
}

// accessor of a json
type Accessor struct {
	Field string // s. a
	Index int    // [n] a
	Map   bool   // map a
	Sub   *Accessor
	End   bool
}
