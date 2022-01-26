import { Library } from 'ffi-napi';

const sdlLibrary = Library('libsdl_parser_export', {
  parse_sdl_generate: ['char *', ['string']],
  parse_sdl_free: ['void', ['char *']],
});

export default function parseSDL(sdlString: string) {
  const responsePointer = sdlLibrary.parse_sdl_generate(sdlString) as any;
  try {
    return responsePointer.readCString();
  } finally {
    sdlLibrary.parse_sdl_free(responsePointer);
  }
}
