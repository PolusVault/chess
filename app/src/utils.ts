export function clamp(val: number, min: number, max: number) {
  return Math.min(Math.max(val, min), max);
}

export function splitStringOnce(s: string, delimiter: string) {
  var i = s.indexOf(delimiter);
  var splits = [s.slice(0, i), s.slice(i + 1)];

  return splits;
}
