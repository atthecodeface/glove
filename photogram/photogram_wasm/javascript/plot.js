import * as utils from "./utils.js";
export class DataRange {
    constructor(data = []) {
        this.data = data;
    }
    push(x, y) {
        this.data.push([x, y]);
    }
    get_range(min, max, properties = {}) {
        if (properties.expand_factor !== undefined) {
            const mid = (min + max) / 2;
            const diff = max - min;
            min = mid - (diff / 2) * properties.expand_factor;
            max = mid + (diff / 2) * properties.expand_factor;
        }
        if (properties.include_zero !== undefined && properties.include_zero) {
            if (min * max > 0) {
                min = Math.min(0, min);
                max = Math.max(0, max);
            }
        }
        return [min, max];
    }
    get_xrange(properties = {}) {
        let min = this.data[0][0];
        let max = this.data[0][0];
        for (const xy of this.data) {
            min = Math.min(min, xy[0]);
            max = Math.max(max, xy[0]);
        }
        return this.get_range(min, max, properties);
    }
    get_yrange(properties = {}) {
        let min = this.data[0][1];
        let max = this.data[0][1];
        for (const xy of this.data) {
            min = Math.min(min, xy[1]);
            max = Math.max(max, xy[1]);
        }
        return this.get_range(min, max, properties);
    }
}
export class Tics {
    constructor(properties) {
        this.spacing = 1;
        this.length = 1;
        this.show_grid = false;
        this.label = false;
        if (properties.spacing !== undefined) {
            this.spacing = properties.spacing;
        }
        if (properties.length !== undefined) {
            this.length = properties.length;
        }
        if (properties.show_grid !== undefined) {
            this.show_grid = properties.show_grid;
        }
        if (properties.label !== undefined) {
            this.label = properties.label;
        }
    }
    /** Set the spacing to be a power of ten such that the range will have at least min_tics */
    set_spacing_of_range(range, min_tics) {
        const delta = range[1] - range[0];
        const delta_spacing = delta / min_tics;
        let spacing = Math.pow(10, Math.floor(Math.log10(delta_spacing)));
        const min_rounded_up = Math.ceil(range[0] / spacing);
        const max_rounded_down = Math.floor(range[1] / spacing);
        if (max_rounded_down - min_rounded_up < min_tics - 1) {
            spacing = spacing / 10;
        }
        this.spacing = spacing;
    }
}
class LinearMap {
    constructor(ofs, scale) {
        this.ofs = 0;
        this.scale = 1;
        this.ofs = ofs;
        this.scale = scale;
    }
    set_mapping(x0, map_x0, x1, map_x1) {
        this.scale = (map_x1 - map_x0) / (x1 - x0);
        this.ofs = map_x0 - x0 * this.scale;
    }
    map(x) {
        return this.ofs + x * this.scale;
    }
    inv(mapped_x) {
        return (mapped_x - this.ofs) / this.scale;
    }
}
export class Plot {
    constructor(wh) {
        /** Origin in the drawing space at which to put the bottom left of the graph */
        this.graph_origin = [0, 0];
        this.xtics = [];
        this.ytics = [];
        /** Derived graph area inside the drawing space */
        this.graph_area = [0, 0, 0, 0];
        /** The area of the data space that is shown in the graph_area on the drawing */
        this.data_area = [0, 0, 0, 0];
        this.graph_wh = wh;
        this.map_x_graph_rel_to_abs = new LinearMap(0, 1);
        this.map_y_graph_rel_to_abs = new LinearMap(0, 1);
        this.map_x_data_to_graph_rel = new LinearMap(0, 1);
        this.map_y_data_to_graph_rel = new LinearMap(0, 1);
    }
    /** Set the position of the bottom-left of the graph rectangle
     *
     */
    set_graph_origin(origin) {
        this.graph_origin = origin;
        this.derive_data();
    }
    set_graph_wh(grapH_wh) {
        this.graph_wh = grapH_wh;
        this.derive_data();
    }
    set_data_range(x0, y0, x1, y1) {
        this.map_x_data_to_graph_rel.set_mapping(x0, 0, x1, 1);
        this.map_y_data_to_graph_rel.set_mapping(y0, 0, y1, 1);
        this.derive_data();
    }
    derive_data() {
        this.graph_area = [
            this.graph_origin[0],
            this.graph_origin[1],
            this.graph_origin[0] + this.graph_wh[0],
            this.graph_origin[1] - this.graph_wh[1],
        ];
        this.map_x_graph_rel_to_abs.set_mapping(0, this.graph_area[0], 1, this.graph_area[2]);
        this.map_y_graph_rel_to_abs.set_mapping(0, this.graph_area[1], 1, this.graph_area[3]);
        this.data_area = [
            this.map_x_data_to_graph_rel.inv(0),
            this.map_y_data_to_graph_rel.inv(0),
            this.map_x_data_to_graph_rel.inv(1),
            this.map_y_data_to_graph_rel.inv(1),
        ];
    }
    generate_box(draw) {
        draw.extend([
            ["W", 4.0],
            ["S", "#fff"],
            ["b"],
            ["m", this.graph_area[0], this.graph_area[1]],
            ["l", this.graph_area[2], this.graph_area[1]],
            ["l", this.graph_area[2], this.graph_area[3]],
            ["l", this.graph_area[0], this.graph_area[3]],
            ["l", this.graph_area[0], this.graph_area[1]],
            ["s"],
        ]);
    }
    generate_plot(draw, data_range) {
        draw.extend([
            ["push"],
            ["W", 4.0],
            ["S", "#ff3"],
            [
                "C",
                this.graph_area[0],
                this.graph_area[3],
                this.graph_area[2],
                this.graph_area[1],
            ],
            ["b"],
        ]);
        for (const [d0, d1] of data_range.data) {
            const x = this.map_x_graph_rel_to_abs.map(this.map_x_data_to_graph_rel.map(d0));
            const y = this.map_y_graph_rel_to_abs.map(this.map_y_data_to_graph_rel.map(d1));
            draw.extend([["l", x, y]]);
        }
        draw.extend([["s"], ["pop"]]);
    }
    generate_tics(draw) {
        for (let i = 0; i < this.xtics.length; i++) {
            this.generate_x_tics(draw, i);
        }
        for (let i = 0; i < this.ytics.length; i++) {
            this.generate_y_tics(draw, i);
        }
    }
    generate_grid(draw) {
        for (let i = 0; i < this.xtics.length; i++) {
            this.generate_x_grid(draw, i);
        }
        for (let i = 0; i < this.ytics.length; i++) {
            this.generate_y_grid(draw, i);
        }
    }
    generate_labels(draw) {
        for (let i = 0; i < this.xtics.length; i++) {
            this.generate_x_labels(draw, i);
        }
        for (let i = 0; i < this.ytics.length; i++) {
            this.generate_y_labels(draw, i);
        }
    }
    iter_spacing(dmin, dmax, spacing, callback) {
        let drange_min = Math.floor(dmin / spacing);
        let drange_max = Math.ceil(dmax / spacing);
        for (let i = drange_min; i <= drange_max; i++) {
            let d = i * spacing;
            let rel = (d - dmin) / (dmax - dmin);
            if (rel < 0 || rel > 1) {
                continue;
            }
            callback(d, rel);
        }
    }
    x_iter_spacing(spacing, callback) {
        this.iter_spacing(this.data_area[0], this.data_area[2], spacing, callback);
    }
    y_iter_spacing(spacing, callback) {
        this.iter_spacing(this.data_area[1], this.data_area[3], spacing, callback);
    }
    generate_x_tics(draw, level) {
        if (this.xtics.length <= level) {
            return;
        }
        const tics = this.xtics[level];
        draw.extend([["W", 4.0], ["S", "#ff3"], ["b"]]);
        this.x_iter_spacing(tics.spacing, (_d, graph_rel) => {
            let x = this.map_x_graph_rel_to_abs.map(graph_rel);
            draw.extend([
                ["m", x, this.graph_area[1]],
                ["L", 0, -tics.length],
            ]);
            draw.extend([
                ["m", x, this.graph_area[3]],
                ["L", 0, tics.length],
            ]);
        });
        draw.extend([["s"]]);
    }
    generate_x_labels(draw, level) {
        if (this.xtics.length <= level) {
            return;
        }
        const tics = this.xtics[level];
        if (!tics.label) {
            return;
        }
        // draw.extend([["W", 4.0], ["S", "#ff3"], ["b"]]);
        this.x_iter_spacing(tics.spacing, (d, graph_rel) => {
            let x = this.map_x_graph_rel_to_abs.map(graph_rel);
            draw.extend([
                ["txt", x, this.graph_area[1] + 30, utils.decimal_to_sig_fig(d, 3)],
            ]);
        });
    }
    generate_y_labels(draw, level) {
        if (this.ytics.length <= level) {
            return;
        }
        const tics = this.ytics[level];
        if (!tics.label) {
            return;
        }
        this.y_iter_spacing(tics.spacing, (d, graph_rel) => {
            let y = this.map_y_graph_rel_to_abs.map(graph_rel);
            draw.extend([
                ["txt", this.graph_area[0] - 30, y, utils.decimal_to_sig_fig(d, 3)],
            ]);
        });
    }
    generate_x_grid(draw, level) {
        if (this.xtics.length <= level) {
            return;
        }
        const tics = this.xtics[level];
        if (!tics.show_grid) {
            return;
        }
        draw.extend([["W", 1.0], ["S", "#ff3"], ["b"]]);
        this.x_iter_spacing(tics.spacing, (d, graph_rel) => {
            let x = this.map_x_graph_rel_to_abs.map(graph_rel);
            if (d == 0) {
                draw.extend([["s"], ["W", 2.0], ["b"]]);
            }
            draw.extend([
                ["m", x, this.graph_area[1]],
                ["l", x, this.graph_area[3]],
            ]);
            if (d == 0) {
                draw.extend([["s"], ["W", 1.0], ["b"]]);
            }
        });
        draw.extend([["s"]]);
    }
    generate_y_tics(draw, level) {
        if (this.ytics.length <= level) {
            return;
        }
        const tics = this.ytics[level];
        draw.extend([["W", 4.0], ["S", "#ff3"], ["b"]]);
        this.y_iter_spacing(tics.spacing, (_d, graph_rel) => {
            let y = this.map_y_graph_rel_to_abs.map(graph_rel);
            draw.extend([
                ["m", this.graph_area[0], y],
                ["L", tics.length, 0],
            ]);
            draw.extend([
                ["m", this.graph_area[2], y],
                ["L", -tics.length, 0],
            ]);
        });
        draw.extend([["s"]]);
    }
    generate_y_grid(draw, level) {
        if (this.ytics.length <= level) {
            return;
        }
        const tics = this.ytics[level];
        if (!tics.show_grid) {
            return;
        }
        draw.extend([["W", 1.0], ["S", "#ff3"], ["b"]]);
        this.y_iter_spacing(tics.spacing, (d, graph_rel) => {
            let y = this.map_y_graph_rel_to_abs.map(graph_rel);
            if (d == 0) {
                draw.extend([["s"], ["W", 2.0], ["b"]]);
            }
            draw.extend([
                ["m", this.graph_area[0], y],
                ["l", this.graph_area[2], y],
            ]);
            if (d == 0) {
                draw.extend([["s"], ["W", 1.0], ["b"]]);
            }
        });
        draw.extend([["s"]]);
    }
}
