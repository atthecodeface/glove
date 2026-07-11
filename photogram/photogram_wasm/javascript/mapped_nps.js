import * as utils from "./utils.js";
const focus_plus_symbol = "\u{271A}"; // ✚
const focus_circle_symbol = "\u{25ef}"; // ◯
const dustbin_symbol = "\u{1f5d1}"; // 🗑
const plus_minus_symbol = "\u{00b1}"; // 🗑
const up_arrow_symbol = "\u{2191}"; // 🗑
export class MappedNp {
    constructor(mapped_nps, wasm_np) {
        /** True if this has a PMS mapping */
        this.has_pms = false;
        /** Data from the PMS */
        this.pms_x = 0;
        this.pms_y = 0;
        this.pms_error = 0;
        this.pms_dsq = 0;
        this.mapped_nps = mapped_nps;
        this.wasm_np = wasm_np;
        this.name_upper = this.wasm_np.name.toUpperCase();
        this.expected_pxy = [0, 0];
        this.focus_dsq = 0;
    }
    /** Accessor */
    x() {
        return this.expected_pxy[0];
    }
    /** Accessor */
    y() {
        return this.expected_pxy[1];
    }
    /** Accessor */
    name() {
        return this.wasm_np.name;
    }
    /** Accessor */
    color() {
        return this.wasm_np.color;
    }
    color_select(parent) {
        const div = parent.add_ele("div");
        div.add_input_color({ rgb_string: this.wasm_np.color }, this.set_color.bind(this));
        div.add_span(this.wasm_np.color);
        return div;
    }
    set_color(color) {
        console.log(this.mapped_nps.project.nps_set_color(this.wasm_np.name, color));
    }
    uncertainty() {
        return 0;
    }
    map_with_camera(camera, focus) {
        const np_pxy = camera.map_model(this.wasm_np.model);
        this.expected_pxy = [np_pxy[0], np_pxy[1]];
        const dx = this.expected_pxy[0] - focus[0];
        const dy = this.expected_pxy[1] - focus[1];
        this.focus_dsq = dx * dx + dy * dy;
    }
    map_with_pm(pms, n) {
        const pxye = pms.get_xy_err(n);
        this.has_pms = true;
        this.pms_x = pxye[0];
        this.pms_y = pxye[1];
        this.pms_error = pxye[2];
        const dx = this.expected_pxy[0] - this.pms_x;
        const dy = this.expected_pxy[1] - this.pms_y;
        this.pms_dsq = dx * dx + dy * dy;
    }
    td_location(t) {
        let location = utils.point_to_dp(this.wasm_np.model, 3);
        if (this.wasm_np.at_infinity) {
            location = up_arrow_symbol + location;
        }
        return t.add_span(location);
    }
    td_uncertainty(t) {
        return t.add_span(this.wasm_np.error.toFixed(3));
    }
    td_expected_at(t) {
        return t.add_span(utils.point_to_dp([this.expected_pxy[0], this.expected_pxy[1]], 1));
    }
    td_pms(t) {
        return t.add_span(utils.point_to_dp([this.pms_x, this.pms_y], 1));
    }
    td_pms_error(t) {
        return t.add_span(plus_minus_symbol + this.pms_error.toString());
    }
    td_mapping_error(t) {
        return t.add_span(this.pms_dsq.toFixed(3));
    }
}
export class MappedNps {
    constructor(project) {
        this.center_pxy = [0, 0];
        this.focus_pxy = [0, 0];
        this.project = project;
        const nps = project.get_wasm_nps();
        this.named_points = [];
        for (const np_name of nps.pts()) {
            this.named_points.push(new MappedNp(this, nps.get_pt(np_name)));
        }
    }
    /** Set the 'focus'- the cursor, usually
     *
     * After this, 'map_with_cip' must be called to updated the points
     */
    set_focus(focus_pxy) {
        this.focus_pxy = focus_pxy;
    }
    map_with_cip(cip) {
        const wasm_cip = cip.wasm_cip;
        if (wasm_cip === null) {
            return;
        }
        const camera = wasm_cip.camera;
        const pms = wasm_cip.pms;
        this.center_pxy = [camera.sensor_cx, camera.sensor_cy];
        for (const np of this.named_points) {
            np.map_with_camera(camera, this.focus_pxy);
            const pm_n = pms.mapping_of_name(np.name());
            if (pm_n !== undefined) {
                np.map_with_pm(pms, pm_n);
            }
        }
    }
    fill_np_table(table) {
        table.add_headings(["Name", "Color", "Location", "Uncertainty"]);
        for (const np of this.named_points) {
            table.add_body([
                np.name(),
                np.color_select(table),
                np.td_location(table),
                np.td_uncertainty(table),
            ]);
        }
    }
    fill_table(table, client) {
        table.add_headings([
            "Name",
            "Color",
            "Location",
            "Uncertainty",
            "Expected at",
            "Focus",
            "Mapped to",
            "Mapping error",
            "Focus",
            "Delete",
        ]);
        for (const np of this.named_points) {
            const np_x = np.x();
            const np_y = np.y();
            const np_name = np.name();
            const focus_np = table.add_input_button(focus_plus_symbol, () => {
                client.mapped_np_select_xy(np_x, np_y);
            });
            const mapped_to = table.add_button("", "", () => {
                client.mapped_np_set_mapping_for(np_name);
            });
            mapped_to.add_content(np.td_pms(table));
            mapped_to.add_content(np.td_pms_error(table));
            const x = np.pms_x;
            const y = np.pms_y;
            const focus_pm = table.add_input_button(focus_circle_symbol, () => {
                client.mapped_np_select_xy(x, y);
            });
            const delete_pms = table.add_input_button(dustbin_symbol, () => {
                client.mapped_np_delete_mapping_for(np_name);
            });
            //let location = `<input type='button' value='&#x1F5D1;' onclick='window.image_canvas.derive_nps_location("${np.name}")'>&nbsp;${html.position(np.model)}`;
            table.add_body([
                np.name(),
                np.color_select(table),
                np.td_location(table),
                np.td_uncertainty(table),
                np.td_expected_at(table),
                focus_np,
                mapped_to,
                np.td_mapping_error(table),
                focus_pm,
                delete_pms,
            ]);
        }
    }
    fill_pms_table(table) {
        table.add_headings([
            "NP Name",
            "Color",
            "Placed at",
            "Given error",
            "Expected at",
            "Delete",
        ]);
        for (const np of this.named_points) {
            table.add_body([
                np.name(),
                np.color(),
                np.td_pms(table),
                np.td_pms_error(table),
                np.td_expected_at(table),
            ]);
        }
    }
    focus_on_src(_x, _y) { }
    delete_pms(_name) { }
    set_pms_to_cursor(_name) { }
    /** Sort by name
     *
     * Missing sort by error, pxy_err, cursor distance
     */
    sort_by_name() {
        this.named_points.sort((a, b) => {
            return utils.strcmp(a.name_upper, b.name_upper);
        });
    }
    sort_by_x() {
        this.named_points.sort((a, b) => {
            return a.expected_pxy[0] - b.expected_pxy[0];
        });
    }
    sort_by_y() {
        this.named_points.sort((a, b) => {
            return a.expected_pxy[1] - b.expected_pxy[1];
        });
    }
    sort_by_focus_dsq() {
        this.named_points.sort((a, b) => {
            return a.focus_dsq - b.focus_dsq;
        });
    }
    sort_by_pms_dsq() {
        this.named_points.sort((a, b) => {
            return a.pms_dsq - b.pms_dsq;
        });
    }
}
