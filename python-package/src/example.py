from sdl_parser import parse_sdl

print(parse_sdl("""
  name: test-scenario
  start: 2022-01-20T13:00:00Z
  end: 2022-01-20T23:00:00Z
"""))
