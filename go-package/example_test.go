package sdl_parser

import (
	"fmt"
)

func ExampleParseSDL() {
	
	fmt.Println(ParseSDL(`
 name: test-scenario
 start: 2022-01-20T13:00:00Z
 end: 2022-01-20T23:00:00Z`))
 
	// Output:
	// map[map[description:<nil> end:2022-01-20T23:00:00Z infrastructure:<nil> name:test-scenario start:2022-01-20T13:00:00Z]] <nil>

}
