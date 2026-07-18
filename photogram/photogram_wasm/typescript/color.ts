/** Choice of color with values from 0 to 1, except hue which is 0-360 */
export interface ColorChoice {
  rgb_string?: string;
  red?: number;
  green?: number;
  blue?: number;
  hue?: number;
  saturation?: number;
  lightness?: number;
}

export function color_of_rgb(r: number, g: number, b: number): number {
  return (
    Math.floor(r * 255) * 0x10000 +
    Math.floor(g * 255) * 0x100 +
    Math.floor(b * 255)
  );
}
export function rgb_of_color(rgb: number): [number, number, number] {
  rgb = Math.floor(rgb);
  return [
    ((rgb>>16) & 255) / 255,
    ((rgb>>8) & 255) / 255,
    (rgb & 255) / 255,
];
}
export function string_color(rgb: number): string {
  return "#" + ("000000" + Math.floor(rgb).toString(16)).slice(-6);
}

export function hls_of_rgb(
  red: number,
  green: number,
  blue: number,
): [number, number, number] {
  let rgb_min = Math.min(red, green, blue);
  let rgb_max = Math.max(red, green, blue);
  let hue = 0;
  let sat = 0;
  let lightness = (rgb_max + rgb_min) / 2;
  if (rgb_min != rgb_max) {
    const chroma = rgb_max - rgb_min;
    sat = chroma / (1 - Math.abs(2 * lightness - 1));
    hue = (green - blue) / chroma;
    if (rgb_max == green) {
      hue = (blue - red) / chroma + 2;
    }
    if (rgb_max == blue) {
      hue = (red - green) / chroma + 4;
    }
    hue = (hue + 6) * 60;
    if (hue > 360) {
      hue -= 360;
    }
  }
  return [hue, sat, lightness];
}

export function rgb_of_hls(
  hue: number,
  saturation: number,
  lightness: number,
): [number, number, number] {
  // RGB of 0,1,5 should be rgb (1,0,0)
  const chroma = saturation * (1 - Math.abs(2 * lightness - 1));
  if (chroma == 0) {
    return [lightness, lightness, lightness];
  }
  // lightness = (Max + min)/2
  // Chroma = Max - min
  let sector = Math.floor(hue / 60);
  let in_sector = hue / 60 - sector;
  if ((sector % 2)== 1) { in_sector = 1 - in_sector; }
  let rgb_min = lightness - chroma / 2;
  let rgb_max = lightness + chroma / 2;
  let rgb_other = rgb_min + chroma * in_sector;

  let red = 0;
  let green = 0;
  let blue = 0;
  switch (sector) {
    case 0: { red = rgb_max; green = rgb_other; blue = rgb_min; break; }
    case 1: { red = rgb_other; green = rgb_max; blue = rgb_min; break; }
    case 2: { red = rgb_min; green = rgb_max; blue = rgb_other; break; }
    case 3: { red = rgb_min; green = rgb_other; blue = rgb_max; break; }
    case 4: { red = rgb_other; green = rgb_min; blue = rgb_max; break; }
    default: { red = rgb_max; green = rgb_min; blue = rgb_other; break; }
}
  return [red, green, blue];
}

export function color_choice_as_rgb(choice: ColorChoice): number {
  let red = choice.red ? choice.red : 0;
  let green = choice.green ? choice.green : 0;
  let blue = choice.blue ? choice.blue : 0;
  if (choice.rgb_string !== undefined) {
    let s = choice.rgb_string;
    if (s[0] == "#") { s = s.slice(1); }
    let color = parseInt(s, 16);
    if (!isNaN(color)) {
      [red, green, blue] = rgb_of_color(color);
    }
  }

  let [hue, saturation, lightness] = hls_of_rgb(red, green, blue);
  if (choice.hue !== undefined) {
    hue = choice.hue;
  }
  if (choice.saturation !== undefined) {
    saturation = choice.saturation;
  }
  if (choice.lightness !== undefined) {
    lightness = choice.lightness;
  }
  let [r, g, b] = rgb_of_hls(hue, saturation, lightness);
  return color_of_rgb(r, g, b);
}
