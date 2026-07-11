export function color_of_rgb(r, g, b) {
    return (Math.floor(r * 255) * 0x10000 +
        Math.floor(g * 255) * 0x100 +
        Math.floor(b * 255));
}
export function rgb_of_color(rgb) {
    rgb = Math.floor(rgb);
    return [
        ((rgb >> 16) & 255) / 255,
        ((rgb >> 8) & 255) / 255,
        (rgb & 255) / 255,
    ];
}
export function string_color(rgb) {
    return "#" + ("000000" + Math.floor(rgb).toString(16)).slice(-6);
}
export function hls_of_rgb(red, green, blue) {
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
export function rgb_of_hls(hue, saturation, lightness) {
    const chroma = saturation * (1 - Math.abs(2 * lightness - 1));
    if (chroma == 0) {
        return [lightness, lightness, lightness];
    }
    // lightness = (Max + min)/2
    // Chroma = Max - min
    let rgb_max = lightness + chroma / 2;
    let sector_hue = hue / 60 + 1;
    if (sector_hue < 0) {
        sector_hue += 6;
    }
    if (sector_hue > 6) {
        sector_hue -= 6;
    }
    // sector is 0 (red is max), 1 (green is max), 2 (blue is max)
    let sector = Math.floor(sector_hue / 2);
    // subhue is -0.5 to 0.5
    let subhue = sector_hue / 2 - sector - 0.5;
    // rgb0 is min -> max as hue increases
    let rgb0 = lightness + subhue * chroma;
    // rgb1 is max -> min as hue increases
    let rgb1 = lightness - subhue * chroma;
    let red = rgb_max;
    let green = rgb0;
    let blue = rgb1;
    if (sector == 1) {
        green = rgb_max;
        red = rgb1;
        blue = rgb0;
    }
    if (sector == 2) {
        blue = rgb_max;
        green = rgb1;
        red = rgb0;
    }
    return [red, green, blue];
}
export function color_choice_as_rgb(choice) {
    let red = choice.red ? choice.red : 0;
    let green = choice.green ? choice.green : 0;
    let blue = choice.blue ? choice.blue : 0;
    if (choice.rgb_string !== undefined) {
        let s = choice.rgb_string;
        if (s[0] == "#") {
            s = s.slice(1);
        }
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
