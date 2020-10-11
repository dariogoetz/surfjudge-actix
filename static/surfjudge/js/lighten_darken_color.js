function lighten_darken_color(col, amt) {

    var usePound = false;

    if (col[0] == "#") {
        col = col.slice(1);
        usePound = true;
    }

    var num = parseInt(col,16);

    var r = (num >> 16) + amt;

    if (r > 255) r = 255;
    else if  (r < 0) r = 0;

    var b = ((num >> 8) & 0x00FF) + amt;

    if (b > 255) b = 255;
    else if  (b < 0) b = 0;

    var g = (num & 0x0000FF) + amt;

    if (g > 255) g = 255;
    else if (g < 0) g = 0;

    return (usePound?"#":"") + (g | (b << 8) | (r << 16)).toString(16);
}

function lighten(col) {
    return lighten_darken_color(col, 175);
}

function get_rgb_components(color) {

    var r = color.substring(1, 3);
    var g = color.substring(3, 5);
    var b = color.substring(5, 7);

    return [parseInt(r, 16), parseInt(g, 16), parseInt(b, 16)];
}

function hex_to_luminance(color) {
    var rgb = get_rgb_components(color);

    for (var i = 1; i < 3; i ++) {
        rgb[i] = rgb[i] / 255.0;
        if (rgb[i] <= 0.03928) {
            rgb[i] = rgb[i] / 12.92;
        } else {
            rgb[i] = ((rgb[i] + 0.055) / 1.055) ^ 2.4;
        }
    }
    var L = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    return L
}

function text_color_for_background(color) {
    return hex_to_luminance(color) < 0.179 ? 'white' : 'black'
}