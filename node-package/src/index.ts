import { Library } from 'ffi-napi';

const sdlLibrary = Library('libsdl_parser_export', {
  parse_sdl_generate: ['char *', ['string']],
  parse_sdl_free: ['void', ['char *']],
});

export function parseSDL(sdl_string: string) {
  const responsePointer = sdlLibrary.parse_sdl_generate(sdl_string) as any;
  try {
    return responsePointer.readCString();
  } finally {
    sdlLibrary.parse_sdl_free(responsePointer);
  }
}
