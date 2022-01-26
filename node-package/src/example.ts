import { parseSDL } from '.';

console.log(parseSDL(`
scenario:
  name: test-scenario
  start: 2022-01-20T13:00:00Z
  end: 2022-01-20T23:00:00Z
`));
