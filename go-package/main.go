package sdl_parser

/*
#cgo CFLAGS: -I/usr/include/sdl-parser
#cgo LDFLAGS: -lsdl_parser
#include <stdlib.h>
#include <sdl_parser.h>
*/
import "C"

func ParseSDL(sdlString string) string {
	cSDLString := C.CString(sdlString)
	return "something"
}
