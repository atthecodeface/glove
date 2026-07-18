//fp is_array
export function is_array(obj) {
    return Object.prototype.toString.call(obj) === "[object Array]";
}
//fp is_string
export function is_string(obj) {
    return typeof obj === "string";
}
//fp is_float
export function is_float(obj) {
    return typeof obj === "number";
}
//fp parse_json
export function parse_json(data) {
    const regex = new RegExp("//[^\n]*", "g");
    // Use replace for older browser compatibility compared to replaceAll; the /g flag makes them do the same thing
    data = data.replace(regex, "");
    try {
        const obj = JSON.parse(data);
        return obj;
    }
    catch (e) {
        return null;
    }
}
//fp strcmp
export function strcmp(a, b) {
    if (a < b) {
        return -1;
    }
    else if (a > b) {
        return 1;
    }
    else {
        return 0;
    }
}
//mp round_to_multiple
export function round_to_multiple(x, m, to = 0) {
    if (to == 0) {
        return m * Math.round(x / m);
    }
    else if (to < 0) {
        return m * Math.floor(x / m);
    }
    else {
        return m * Math.ceil(x / m);
    }
}
export function point_div_to_dp_vertical(parent, prefix, coords, dp) {
    const div = parent.add_ele("div");
    const s = [];
    for (const c of coords) {
        s.push(c.toFixed(dp));
    }
    let n = s.length;
    for (let i = 0; i < n; i++) {
        if (i == 0) {
            div.add_span(prefix + "(" + s[i]);
        }
        else if (i == n - 1) {
            div.add_ele("br");
            div.add_span(s[i] + ")");
        }
        else {
            div.add_ele("br");
            div.add_span(s[i] + ",");
        }
    }
    return div;
}
export function point_to_dp(coords, dp) {
    let result = "";
    let sep = "(";
    for (const c of coords) {
        result += sep;
        result += c.toFixed(dp);
        sep = ", ";
    }
    return result + ")";
}
