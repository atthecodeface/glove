import { WasmPointMapping, WasmQuatf64, WasmVec2f64, WasmVec3f64, } from "../pkg/photogram_wasm.js";
import * as utils from "./utils.js";
import { color_of_rgb, rgb_of_hls, string_color } from "./color.js";
const plus_symbol = "\u{271A}"; // ✚
const circle_symbol = "\u{25ef}"; // ◯
const dustbin_symbol = "\u{1f5d1}"; // 🗑
const plus_minus_symbol = "\u{00b1}"; // ±
const up_arrow_symbol = "\u{2191}"; // ↑
const down_arrow_symbol = "\u{2193}"; // ↓
class SortByFieldBase {
    constructor() {
        this.sort_class = "";
        this.sort_subclass = "";
    }
    next_field_kind_in_class() {
        return this;
    }
    symbols(ascending) {
        if (!ascending) {
            return this.sort_subclass + down_arrow_symbol;
        }
        else {
            return this.sort_subclass + up_arrow_symbol;
        }
    }
    metric(_pm) { return 0; }
    matches_class(table_field) {
        return this.sort_class == table_field.sort_class;
    }
    sort_fn(a, b) {
        return this.metric(a) - this.metric(b);
    }
    ;
}
class SortByFieldName extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "Name";
    }
    sort_fn(a, b) {
        return utils.strcmp(a.np_name_upper, b.np_name_upper);
    }
}
SortByFieldName.singleton = new SortByFieldName();
class SortByFieldExpectedX extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "Expected";
        this.sort_subclass = "X";
    }
    next_field_kind_in_class() { return SortByFieldExpectedY.singleton; }
    metric(wpm) {
        return wpm.expected_x;
    }
}
SortByFieldExpectedX.singleton = new SortByFieldExpectedX();
class SortByFieldExpectedY extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "Expected";
        this.sort_subclass = "Y";
    }
    next_field_kind_in_class() { return SortByFieldExpectedX.singleton; }
    metric(wpm) {
        return wpm.expected_y;
    }
}
SortByFieldExpectedY.singleton = new SortByFieldExpectedY();
class SortByFieldMappedX extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "Mapped";
        this.sort_subclass = "X";
    }
    next_field_kind_in_class() { return SortByFieldMappedY.singleton; }
    metric(wpm) {
        return wpm.image_x;
    }
}
SortByFieldMappedX.singleton = new SortByFieldMappedX();
class SortByFieldMappedY extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "Mapped";
        this.sort_subclass = "Y";
    }
    next_field_kind_in_class() { return SortByFieldMappedX.singleton; }
    metric(wpm) {
        return wpm.image_y;
    }
}
SortByFieldMappedY.singleton = new SortByFieldMappedY();
class SortByFieldColor extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "Color";
    }
    sort_fn(a, b) {
        return utils.strcmp(a.np_color, b.np_color);
    }
}
SortByFieldColor.singleton = new SortByFieldColor();
class SortByFieldDsq extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "Dsq";
    }
    metric(wpm) {
        return wpm.d_map_distance;
    }
}
SortByFieldDsq.singleton = new SortByFieldDsq();
class SortByFieldCursorDistance extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "CursorDistance";
    }
    metric(wpm) {
        return wpm.cursor_distance;
    }
}
SortByFieldCursorDistance.singleton = new SortByFieldCursorDistance();
class SortByFieldMappedRoll extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "MappedRollYaw";
        this.sort_subclass = "R";
    }
    next_field_kind_in_class() { return SortByFieldMappedYaw.singleton; }
    metric(wpm) {
        return wpm.image_roll;
    }
}
SortByFieldMappedRoll.singleton = new SortByFieldMappedRoll();
class SortByFieldMappedYaw extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "MappedRollYaw";
        this.sort_subclass = "Y";
    }
    next_field_kind_in_class() { return SortByFieldMappedRoll.singleton; }
    metric(wpm) {
        return wpm.image_yaw;
    }
}
SortByFieldMappedYaw.singleton = new SortByFieldMappedYaw();
class SortByFieldRollErr extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "RollErr";
    }
    metric(wpm) {
        return wpm.d_map_roll_err;
    }
}
SortByFieldRollErr.singleton = new SortByFieldRollErr();
class SortByFieldYawErr extends SortByFieldBase {
    constructor() {
        super(...arguments);
        this.sort_class = "YawErr";
    }
    metric(wpm) {
        return wpm.d_map_yaw_err;
    }
}
SortByFieldYawErr.singleton = new SortByFieldYawErr();
class SortBy {
    constructor() {
        this.field = SortByFieldName.singleton;
        this.ascending = true;
    }
    table_heading(table, text, field, callback) {
        const button = table.add_button("", "", () => { callback(field); });
        if (this.field.matches_class(field)) {
            button.add_content(this.field.symbols(this.ascending));
        }
        else {
            button.add_content(field.symbols(true));
        }
        button.add_content(text);
        return button;
    }
    clicked(field) {
        if (!this.field.matches_class(field)) {
            this.ascending = true;
            this.field = field;
        }
        else if (this.ascending) {
            this.ascending = false;
        }
        else {
            this.ascending = true;
            this.field = this.field.next_field_kind_in_class();
        }
    }
}
export class MappedNp {
    constructor(mapped_nps, wasm_np) {
        this.mapped_nps = mapped_nps;
        this.wasm_pms = new WasmPointMapping(wasm_np);
    }
    /** Accessor */
    name() {
        return this.wasm_pms.np_name;
    }
    /** Accessor */
    color() {
        return this.wasm_pms.np_color;
    }
    /** Return true if the point is actually mapped */
    has_pms() {
        return this.wasm_pms.has_pms;
    }
    color_select(parent) {
        const div = parent.add_ele("div");
        div.add_input_color({ rgb_string: this.wasm_pms.np_color }, this.set_color.bind(this));
        div.add_ele("br");
        div.add_span(this.wasm_pms.np_color);
        return div;
    }
    set_color(color) {
        this.mapped_nps.project.nps_set_color(this.wasm_pms.np_name, color);
    }
    uncertainty() {
        return 0;
    }
    move_cursor(focus) {
        this.wasm_pms.set_cursor(focus[0], focus[1]);
    }
    update_mapping(camera, pms, cursor) {
        this.wasm_pms.update(camera, pms, cursor[0], cursor[1]);
    }
    div_location(t) {
        this.wasm_pms.np_model_set_vec(this.mapped_nps.wasm_vec3);
        return utils.point_div_to_dp_vertical(t, up_arrow_symbol, this.mapped_nps.wasm_vec3.array, 3);
    }
    span_uncertainty(t) {
        return t.add_span(this.wasm_pms.np_uncertainty.toFixed(3));
    }
    div_expected_at(t) {
        return utils.point_div_to_dp_vertical(t, "", [this.wasm_pms.expected_x, this.wasm_pms.expected_y], 1);
    }
    div_pms(t) {
        this.wasm_pms.set_image_vec(this.mapped_nps.wasm_vec2);
        return utils.point_div_to_dp_vertical(t, "", [this.wasm_pms.image_x, this.wasm_pms.image_y], 1);
    }
    span_pms_uncertainty(t) {
        return t.add_span(plus_minus_symbol + this.wasm_pms.img_uncertainty.toString());
    }
    span_focus_dsq(t) {
        return t.add_span(this.wasm_pms.cursor_distance.toFixed(3));
    }
    span_pms_dsq(t) {
        return t.add_span(this.wasm_pms.d_map_distance.toFixed(3));
    }
    div_roll_yaw(t) {
        return utils.point_div_to_dp_vertical(t, "", [this.wasm_pms.image_roll * 180 / 3.1415926, this.wasm_pms.image_yaw * 180 / 3.1415926], 1);
    }
    span_map_roll_err(t) {
        return t.add_span(this.wasm_pms.d_map_roll_err.toFixed(3));
    }
    span_map_yaw_err(t) {
        return t.add_span(this.wasm_pms.d_map_yaw_err.toFixed(3));
    }
}
export class MappedNps {
    constructor(project) {
        this.center_pxy = [0, 0];
        this.focus_pxy = [0, 0];
        this.epoch = 0;
        this.pending_nps = true;
        this.pending_pms = true;
        this.pending_calcs = true;
        this.total_sq_roll_error = 0;
        this.total_sq_yaw_error = 0;
        this.wasm_quat = WasmQuatf64.unit();
        this.wasm_vec2 = WasmVec2f64.zero();
        this.wasm_vec3 = WasmVec3f64.zero();
        this.project = project;
        this.named_points = [];
        this.sort = new SortBy();
    }
    update() {
        if (this.pending_nps) {
            this.rebuild_nps();
            this.map_with_cip();
            this.epoch += 1;
        }
        else if (this.pending_pms) {
            this.map_with_cip();
            this.epoch += 1;
        }
        else if (this.pending_calcs) {
            this.map_with_cip();
            this.epoch += 1;
        }
        this.pending_nps = false;
        this.pending_pms = false;
        this.pending_calcs = false;
        return this.epoch;
    }
    /** Rebuild from scratch from the NamedPointSet for this Project */
    rebuild_nps() {
        const nps = this.project.get_wasm_nps();
        this.named_points = [];
        if (nps === null) {
            return;
        }
        for (const np_name of nps.pts()) {
            this.named_points.push(new MappedNp(this, nps.get_pt(np_name)));
        }
        this.sort_named_points();
    }
    /** Set the 'focus'- the cursor, usually
     *
     * After this, 'map_with_cip' must be called to updated the points
     */
    set_focus(x, y) {
        this.focus_pxy = [x, y];
        this.pending_calcs = true;
    }
    /** Remap the NPs with the specified Cip
     *
     * This should be invoked whenver the camera changes, CIP changes, etc
     */
    map_with_cip() {
        const cip = this.project.get_cip();
        const wasm_cip = cip.wasm_cip;
        if (wasm_cip === null) {
            return;
        }
        const camera = wasm_cip.camera;
        const pms = wasm_cip.pms;
        this.center_pxy = [camera.sensor_cx, camera.sensor_cy];
        this.total_sq_roll_error = 0;
        this.total_sq_yaw_error = 0;
        for (const mnp of this.named_points) {
            mnp.update_mapping(camera, pms, this.focus_pxy);
            this.total_sq_roll_error += 1E6 * mnp.wasm_pms.d_map_roll_err * mnp.wasm_pms.d_map_roll_err;
            this.total_sq_yaw_error += 1E6 * mnp.wasm_pms.d_map_yaw_err * mnp.wasm_pms.d_map_yaw_err;
        }
        this.sort_named_points();
    }
    /** Sort-by has been updated, to the specified field
     *
     * If currently on that field then toggle ascending/descending or similar
     */
    sort_by_clicked(field) {
        this.sort.clicked(field);
        this.sort_named_points();
        this.project.mapped_changed();
    }
    /** Fill a table of just the NamedPoints
     *
     */
    fill_np_table(table) {
        const name = this.sort.table_heading(table, "Name", SortByFieldName.singleton, this.sort_by_clicked.bind(this));
        const color = this.sort.table_heading(table, "Color", SortByFieldColor.singleton, this.sort_by_clicked.bind(this));
        table.add_headings([name, color, "Location", "Uncertainty"]);
        for (const np of this.named_points) {
            table.add_body([
                np.name(),
                np.color_select(table),
                np.div_location(table),
                np.span_uncertainty(table),
            ]);
        }
    }
    fill_table(table, client) {
        const name = this.sort.table_heading(table, "Name", SortByFieldName.singleton, this.sort_by_clicked.bind(this));
        const color = this.sort.table_heading(table, "Color", SortByFieldColor.singleton, this.sort_by_clicked.bind(this));
        const exp_at = this.sort.table_heading(table, "Expected at", SortByFieldExpectedX.singleton, this.sort_by_clicked.bind(this));
        const map_to = this.sort.table_heading(table, "Mapped to", SortByFieldMappedX.singleton, this.sort_by_clicked.bind(this));
        const dsq = this.sort.table_heading(table, "E-M-DXY", SortByFieldDsq.singleton, this.sort_by_clicked.bind(this));
        const cursor_distance = this.sort.table_heading(table, "Cursor-DXY", SortByFieldCursorDistance.singleton, this.sort_by_clicked.bind(this));
        const roll = this.sort.table_heading(table, "Roll,Yaw", SortByFieldMappedRoll.singleton, this.sort_by_clicked.bind(this));
        const roll_err = this.sort.table_heading(table, "Roll Err", SortByFieldRollErr.singleton, this.sort_by_clicked.bind(this));
        const yaw_err = this.sort.table_heading(table, "Yaw Err", SortByFieldYawErr.singleton, this.sort_by_clicked.bind(this));
        table.add_headings([name, color,
            "Location",
            "Uncertainty",
            exp_at, map_to, dsq,
            cursor_distance,
            roll, roll_err, yaw_err,
            "Action",
        ]);
        for (const mnp of this.named_points) {
            const np_x = mnp.wasm_pms.expected_x;
            const np_y = mnp.wasm_pms.expected_y;
            const np_name = mnp.name();
            const expected_at = table.add_button("", "", () => {
                client.mapped_np_select_xy(np_x, np_y);
            });
            expected_at.add_content(mnp.div_expected_at(table));
            let mapped_to = null;
            let action = null;
            if (mnp.has_pms()) {
                mnp.wasm_pms.set_image_vec(this.wasm_vec2);
                const x = mnp.wasm_pms.image_x;
                const y = mnp.wasm_pms.image_y;
                mapped_to = table.add_button("", "", () => {
                    client.mapped_np_select_xy(x, y);
                });
                mapped_to.add_content(mnp.div_pms(table));
                mapped_to.add_content(mnp.span_pms_uncertainty(table));
                action = table.add_ele("div");
                action.add_input_button(circle_symbol, () => {
                    client.mapped_np_set_mapping_for(np_name);
                });
                action.add_input_button(dustbin_symbol, () => {
                    client.mapped_np_delete_mapping_for(np_name);
                });
            }
            else {
                mapped_to = table.add_span("");
                action = table.add_input_button(plus_symbol, () => {
                    client.mapped_np_add_mapping_for(np_name);
                });
            }
            table.add_body([
                mnp.name(),
                mnp.color_select(table),
                mnp.div_location(table),
                mnp.span_uncertainty(table),
                expected_at,
                mapped_to,
                mnp.span_pms_dsq(table),
                mnp.span_focus_dsq(table),
                mnp.div_roll_yaw(table),
                mnp.span_map_roll_err(table),
                mnp.span_map_yaw_err(table),
                action,
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
                np.div_pms(table),
                np.span_pms_uncertainty(table),
                np.div_expected_at(table),
            ]);
        }
    }
    /** Sort the named points using the current sort order */
    sort_named_points() {
        const opt_invert = this.sort.ascending ? 1 : -1;
        let sort_fn = (a, b) => {
            return opt_invert * this.sort.field.sort_fn(a.wasm_pms, b.wasm_pms);
        };
        this.named_points.sort(sort_fn);
    }
    /** Get a relative distance (0 to 1) of named point index 'n' from the first named point using the sort order
     */
    relative_distance(mnp) {
        const m_mnp_0 = this.sort.field.metric(this.named_points[0].wasm_pms);
        const m_mnp_last = this.sort.field.metric(this.named_points[this.named_points.length - 1].wasm_pms);
        const m_mnp = this.sort.field.metric(mnp.wasm_pms);
        return (m_mnp_last == m_mnp_0) ? 0 : ((m_mnp - m_mnp_0) / (m_mnp_last - m_mnp_0));
    }
    /** Recolor the named points given the current order */
    recolor_nps() {
        const hue_range_min = 0;
        const hue_range_max = 240;
        const hue_range = hue_range_max - hue_range_min;
        const n = this.named_points.length;
        let sat_step = 1;
        let lig_step = 1;
        let hue_deg = 0;
        while (true) {
            hue_deg = hue_range / Math.ceil(n / sat_step / lig_step);
            if (hue_deg >= 5) {
                break;
            }
            if (sat_step < lig_step * 5) {
                sat_step += 1;
            }
            else {
                lig_step += 1;
            }
        }
        const hue_step = Math.floor(hue_range / hue_deg);
        let sat_min = 1;
        let sat_sc = 0;
        let lig_min = 0.5;
        let lig_sc = 0;
        if (sat_step > 1) {
            sat_sc = -1 / (sat_step + 1);
            sat_min = 1.0;
        }
        if (lig_step > 1) {
            lig_sc = 0.5 / (lig_step + 1);
            lig_min = 0.5;
        }
        for (let i = 0; i < n; i += 1) {
            const s = i % sat_step;
            const l = Math.floor(i / sat_step) % lig_step;
            const h = Math.floor(Math.floor(i / sat_step) / lig_step) % hue_step;
            let hue = h * hue_range / hue_step + hue_range_min;
            let saturation = s * sat_sc + sat_min;
            let lightness = l * lig_sc + lig_min;
            const rgb = rgb_of_hls(hue, saturation, lightness);
            const color = string_color(color_of_rgb(rgb[0], rgb[1], rgb[2]));
            this.project.wasm_project.nps.set_color(this.named_points[i].name(), color);
        }
        this.project.np_changed(true);
    }
    /** Recolor the named points given the current order */
    recolor_nps_by_distance() {
        for (const mnp of this.named_points) {
            let hue = this.relative_distance(mnp) * 240;
            const rgb = rgb_of_hls(hue, 1.0, 0.5);
            const color = string_color(color_of_rgb(rgb[0], rgb[1], rgb[2]));
            this.project.wasm_project.nps.set_color(mnp.name(), color);
        }
        this.project.np_changed(true);
    }
}
