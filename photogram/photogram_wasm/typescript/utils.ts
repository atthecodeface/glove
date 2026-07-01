//fp is_array
export function is_array(obj: Object): boolean {
  return Object.prototype.toString.call(obj) === "[object Array]";
}

//fp is_string
export function is_string(obj: Object): boolean {
  return typeof obj === "string";
}

//fp is_float
export function is_float(obj: Object): boolean {
  return typeof obj === "number";
}

//fp parse_json
export function parse_json(data: string): Object | null {
  const regex = new RegExp("//[^\n]*", "g");
  // Use replace for older browser compatibility compared to replaceAll; the /g flag makes them do the same thing
  data = data.replace(regex, "");
  try {
    const obj = JSON.parse(data);
    return obj;
  } catch (e) {
    return null;
  }
}

//fp strcmp
export function strcmp(a: string, b: string): number {
  if (a < b) {
    return -1;
  } else if (a > b) {
    return 1;
  } else {
    return 0;
  }
}

//mp round_to_multiple
export function round_to_multiple(x: number, m: number, to = 0): number {
  if (to == 0) {
    return m * Math.round(x / m);
  } else if (to < 0) {
    return m * Math.floor(x / m);
  } else {
    return m * Math.ceil(x / m);
  }
}

export function decimal_to_sig_fig(x: number, _max_dp: number): string {
  return x.toString();
}
