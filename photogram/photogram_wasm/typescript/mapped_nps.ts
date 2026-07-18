import {
  WasmCameraInstance,
  WasmNamedPoint,
  WasmPointMappingSet,
  WasmQuatf64,
  WasmVec2f64,
  WasmVec3f64,
} from "../pkg/photogram_wasm.js";

import * as utils from "./utils.js";
import { HtmlElement, Table } from "./html.js";

import { Project } from "./project.js";

const plus_symbol = "\u{271A}"; // ✚
const circle_symbol = "\u{25ef}"; // ◯
const dustbin_symbol = "\u{1f5d1}"; // 🗑
const plus_minus_symbol = "\u{00b1}"; // ±
const up_arrow_symbol = "\u{2191}"; // ↑
const down_arrow_symbol = "\u{2193}"; // ↓

enum SortByField {
  Name,
  Color,
  ExpectedX,
  ExpectedY,
  MappedX,
  MappedY,
  FocusDsq,
  Dsq,
  RollErr,
  YawErr,
}

class SortBy {
  field: SortByField = SortByField.Name;
  ascending: boolean = true;
  table_heading(table: HtmlElement, text:string, field: SortByField, callback: (field: SortByField) => void): HtmlElement {
    const field_matches = this.matches(field);
    const button = table.add_button("", "", () => { callback(field); });
    if (field_matches) {
      button.add_content(SortBy.symbols(this.ascending, this.field));
    } else {
      button.add_content(SortBy.symbols(true, field));
    }
    button.add_content(text);
    return button;
  }
  matches(field: SortByField): boolean {
    switch (this.field) {
      case SortByField.ExpectedX:
      case SortByField.ExpectedY: { return field == SortByField.ExpectedX || field == SortByField.ExpectedY; }
      case SortByField.MappedX:
      case SortByField.MappedY: { return field == SortByField.MappedX || field == SortByField.MappedY; }
      default: { return this.field == field; }
    }
  }
  clicked(field: SortByField) {
    if (!this.matches(field)) {
      this.ascending = true;
      this.field = field;
      return;
    }
    if (this.ascending) {
      this.ascending = false;
      return;
    }
    this.ascending = true;
    switch (this.field) {
      case SortByField.ExpectedX: {
        this.field = SortByField.ExpectedY;
        return;
      }
      case SortByField.MappedX: {
        this.field = SortByField.MappedY;
        return;
      }
      case SortByField.ExpectedY: {
        this.field = SortByField.ExpectedX;
        return;
      }
      case SortByField.MappedY: {
        this.field = SortByField.MappedX;
        return;
      }
      default: {
        return;
      }
    }
  }

  static symbols(ascending:boolean, field:SortByField): string {
    let symbol = up_arrow_symbol;
    if (!ascending) {
      symbol = down_arrow_symbol;
    }
    switch (field) {
      case SortByField.ExpectedX:
      case SortByField.MappedX: {
        return "X" + symbol;
      }
      case SortByField.ExpectedY:
      case SortByField.MappedY: {
        return "Y" + symbol;
      }
      default: {
        return symbol;
      }
    }
  }
}

export interface MappedNpClient {
  mapped_np_select_xy(x: number, y: number): void;
  mapped_np_add_mapping_for(np_name: string): void;
  mapped_np_delete_mapping_for(np_name: string): void;
  mapped_np_set_mapping_for(np_name: string): void;
}

export class MappedNp {
  mapped_nps: MappedNps;
  wasm_np: WasmNamedPoint;
  name_upper: string;

  /** named point mapped onto the sensor pxy through the camera */
  expected_pxy: [number, number];

  /** Distance from focus */
  focus_dsq: number;

  /** True if this has a PMS mapping */
  has_pms: boolean = false;

  /** Data from the PMS */
  pms_x: number = 0;
  pms_y: number = 0;
  pms_error: number = 0;
  d_map_yaw_err: number = 0;
  d_map_roll_err: number = 0;
  d_map_sq: number = 0;

  constructor(mapped_nps: MappedNps, wasm_np: WasmNamedPoint) {
    this.mapped_nps = mapped_nps;
    this.wasm_np = wasm_np;
    this.name_upper = this.wasm_np.name.toUpperCase();
    this.expected_pxy = [0, 0];
    this.focus_dsq = 0;
  }

  /** Accessor */
  x(): number {
    return this.expected_pxy[0];
  }
  /** Accessor */
  y(): number {
    return this.expected_pxy[1];
  }
    /** Accessor */
  name(): string {
    return this.wasm_np.name;
  }
    /** Accessor */
  color(): string {
    return this.wasm_np.color;
  }

  color_select(parent: HtmlElement): HtmlElement {
    const div = parent.add_ele("div");
    div.add_input_color({ rgb_string: this.wasm_np.color }, this.set_color.bind(this));
    div.add_ele("br");
    div.add_span(this.wasm_np.color);
    return div;
  }

  set_color(color: string) {
    this.mapped_nps.project.nps_set_color(this.wasm_np.name, color);
  }

  uncertainty(): number {
    return 0;
  }

  map_model_with_camera(camera: WasmCameraInstance, focus: [number, number]) {
    // GJS FIXME
    const np_pxy = camera.map_model(this.wasm_np.model_as_array());
    this.expected_pxy = [np_pxy[0]!, np_pxy[1]!];
    const dx = this.expected_pxy[0] - focus[0];
    const dy = this.expected_pxy[1] - focus[1];
    this.focus_dsq = Math.sqrt(dx * dx + dy * dy);
  }

  get_pms_mapping(camera: WasmCameraInstance, pms: WasmPointMappingSet, n: number) {
    const pxye = pms.get_xy_err(n)!;
    this.has_pms = true;
    this.pms_x = pxye[0]!;
    this.pms_y = pxye[1]!;
    this.pms_error = pxye[2]!;
    const dx = this.expected_pxy[0] - this.pms_x;
    const dy = this.expected_pxy[1] - this.pms_y;
    this.d_map_sq = Math.sqrt(dx * dx + dy * dy);

    const wasm_vec2 = this.mapped_nps.wasm_vec2;
    const wasm_vec3 = this.mapped_nps.wasm_vec3;
    const wasm_quat = this.mapped_nps.wasm_quat;

    // Convert the placed mapped position to a roll/yaw
    //
    // Note that the sensor_dir_of_pt uses the sensor centre and pixel aspect
    // ratio to map to a pure positionq
    //
    // This does NOT use the lens mapping
    wasm_vec2.x = pxye[0]!;
    wasm_vec2.y = pxye[1]!;
    camera.set_sensor_dir_of_pt(wasm_vec2, wasm_vec3);
    const map_roll = camera.camera_roll_of_dir(wasm_vec3);
    wasm_quat.set_unit();
    wasm_quat.set_mul_rotate_z(-map_roll);
    wasm_vec3.set_apply_q3(wasm_quat);
    const placed_yaw = wasm_vec3.x / wasm_vec3.z;

    // Convert the NP expected position, given orientation and lens calibration,
    // to a yaw for yaw error
    //
    // This does NOT use the lens mapping - but the expected position did
    wasm_vec2.x = this.expected_pxy[0];
    wasm_vec2.y = this.expected_pxy[1];
    camera.set_sensor_dir_of_pt(wasm_vec2, wasm_vec3);

    // Rotate the direction for the NP expected position by -map_roll around -Z to
    // generate an (x,y,z) whose x is 'yaw' error, y is 'roll' error, scaled down by
    // z (which should be 1-epsilon)
    wasm_vec3.set_apply_q3(wasm_quat);

    this.d_map_yaw_err = 1000 * (wasm_vec3.x / wasm_vec3.z - placed_yaw);
    this.d_map_roll_err = 1000 * wasm_vec3.y / wasm_vec3.z;
  }

  div_location(t: HtmlElement): HtmlElement {
    return utils.point_div_to_dp_vertical(t, up_arrow_symbol, this.wasm_np.model_as_array(), 3);
  }

  span_uncertainty(t: HtmlElement): HtmlElement {
    return t.add_span(this.wasm_np.error.toFixed(3));
  }

  div_expected_at(t: HtmlElement): HtmlElement {
    return utils.point_div_to_dp_vertical(t, "", [this.expected_pxy[0], this.expected_pxy[1]], 1);
  }

  div_pms(t: HtmlElement): HtmlElement {
    return utils.point_div_to_dp_vertical(t, "", [this.pms_x, this.pms_y], 1);
  }

  span_pms_uncertainty(t: HtmlElement): HtmlElement {
    return t.add_span(plus_minus_symbol + this.pms_error.toString());
  }

  span_focus_dsq(t: HtmlElement): HtmlElement {
    return t.add_span(this.focus_dsq.toFixed(3));
  }
  span_pms_dsq(t: HtmlElement): HtmlElement {
    return t.add_span(this.d_map_sq.toFixed(3));
  }
  span_map_roll(t: HtmlElement): HtmlElement {
    return t.add_span(this.d_map_roll_err.toFixed(3));
  }
  span_map_yaw(t: HtmlElement): HtmlElement {
    return t.add_span(this.d_map_yaw_err.toFixed(3));
  }
}

export class MappedNps {
  project: Project;
  named_points: MappedNp[];
  center_pxy: [number, number] = [0, 0];
  focus_pxy: [number, number] = [0, 0];

  sort: SortBy;
  epoch: number = 0;
  pending_nps: boolean = true;
  pending_pms: boolean = true;
  pending_calcs: boolean = true;

  total_sq_roll_error: number= 0;
  total_sq_yaw_error: number= 0;

  wasm_quat : WasmQuatf64= WasmQuatf64.unit();
  wasm_vec2 : WasmVec2f64= WasmVec2f64.zero();
  wasm_vec3 : WasmVec3f64= WasmVec3f64.zero();

  constructor(project: Project) {
    this.project = project;
    this.named_points = [];
    this.sort = new SortBy();
  }

  update(): number {
    if (this.pending_nps) {
      this.rebuild_nps();
      this.map_with_cip();
      this.epoch += 1;
    } else if (this.pending_pms) {
      this.map_with_cip();
      this.epoch += 1;
    } else if (this.pending_calcs) {
      this.map_with_cip();
      this.epoch += 1;
    }
    this.pending_nps = false;
    this.pending_pms = false;
    this.pending_calcs = false;
    return this.epoch;
  }

  rebuild_nps() {
    const nps = this.project.get_wasm_nps();
    this.named_points = [];
    if (nps === null) { return; }
    for (const np_name of nps.pts()) {
      this.named_points.push(new MappedNp(this, nps.get_pt(np_name)!));
    }
    this.sort_named_points();
  }

  /** Set the 'focus'- the cursor, usually
   *
   * After this, 'map_with_cip' must be called to updated the points
   */
  set_focus(x: number, y: number) {
    console.log("Napping nps set focus");
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
    for (const np of this.named_points) {
      np.map_model_with_camera(camera, this.focus_pxy);
      const pm_n = pms.mapping_of_name(np.name());
      if (pm_n !== undefined) {
        np.get_pms_mapping(camera, pms, pm_n);
        this.total_sq_roll_error += np.d_map_roll_err * np.d_map_roll_err;
        this.total_sq_yaw_error += np.d_map_yaw_err * np.d_map_yaw_err;
      }
    }
    this.sort_named_points();
  }

  sort_by_clicked(field: SortByField) {
    this.sort.clicked(field);
    this.sort_named_points();
    this.project.mapped_changed();
  }

  fill_np_table(table: Table) {
    const name = this.sort.table_heading(table, "Name", SortByField.Name, this.sort_by_clicked.bind(this));
    const color = this.sort.table_heading(table, "Color", SortByField.Color, this.sort_by_clicked.bind(this));
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

  fill_table(table: Table, client: MappedNpClient) {
    const name = this.sort.table_heading(table, "Name", SortByField.Name, this.sort_by_clicked.bind(this));
    const color = this.sort.table_heading(table, "Color", SortByField.Color, this.sort_by_clicked.bind(this));
    const exp_at = this.sort.table_heading(table, "Expected at", SortByField.ExpectedX, this.sort_by_clicked.bind(this));
    const map_to = this.sort.table_heading(table, "Mapped to", SortByField.MappedX, this.sort_by_clicked.bind(this));
    const dsq = this.sort.table_heading(table, "DXY^2", SortByField.Dsq, this.sort_by_clicked.bind(this));
    const focus_dsq = this.sort.table_heading(table, "Focus DXY^2", SortByField.FocusDsq, this.sort_by_clicked.bind(this));
    const roll_err = this.sort.table_heading(table, "Roll Err", SortByField.RollErr, this.sort_by_clicked.bind(this));
    const yaw_err = this.sort.table_heading(table, "Yaw Err", SortByField.YawErr, this.sort_by_clicked.bind(this));
    table.add_headings([name, color,
      "Location",
      "Uncertainty",
      exp_at, map_to,
      focus_dsq,
      dsq, roll_err, yaw_err,
      "Action",
    ]);

    for (const np of this.named_points) {
      const np_x = np.x();
      const np_y = np.y();
      const np_name = np.name();

      const expected_at = table.add_button("", "", () => {
        client.mapped_np_select_xy(np_x, np_y);
      });
      expected_at.add_content(np.div_expected_at(table));

      let mapped_to :HtmlElement | null= null;
      let action:HtmlElement | null = null;

      if (np.has_pms) {
        const x = np.pms_x;
        const y = np.pms_y;
        mapped_to = table.add_button("", "", () => {
          client.mapped_np_select_xy(x, y)
        });
            mapped_to.add_content(np.div_pms(table));
            mapped_to.add_content(np.span_pms_uncertainty(table));
            action = table.add_ele("div");
            action.add_input_button(circle_symbol, () => {
              client.mapped_np_set_mapping_for(np_name);
            });
            action.add_input_button(dustbin_symbol, () => {
              client.mapped_np_delete_mapping_for(np_name);
            });
        } else {
          mapped_to = table.add_span("");
          action = table.add_input_button(plus_symbol, () => {
            client.mapped_np_add_mapping_for(np_name);
          });
      }

      table.add_body([
        np.name(),
        np.color_select(table),
        np.div_location(table),
        np.span_uncertainty(table),
        expected_at,
        mapped_to,
        np.span_focus_dsq(table),
        np.span_pms_dsq(table),
        np.span_map_roll(table),
        np.span_map_yaw(table),
        action,
      ]);
    }
  }

  fill_pms_table(table: Table) {
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

  sort_named_points() {
    const opt_invert = this.sort.ascending ? 1 : -1;
    let sort_fn = (a:MappedNp, b:MappedNp) => {
      return opt_invert * utils.strcmp(a.name_upper, b.name_upper);
    };
    switch (this.sort.field) {
      case SortByField.ExpectedX: {
        sort_fn = (a:MappedNp, b:MappedNp) => {
          return opt_invert *(a.expected_pxy[0] - b.expected_pxy[0]);
        };
        break;
      }
      case SortByField.ExpectedY: {
        sort_fn = (a:MappedNp, b:MappedNp) => {
          return opt_invert *(a.expected_pxy[1] - b.expected_pxy[1]);
        };
        break;
      }
      case SortByField.Color: {
        sort_fn = (a:MappedNp, b:MappedNp) => {
          return opt_invert * utils.strcmp(a.wasm_np.color, b.wasm_np.color);
        };
        break;
      }
      case SortByField.Dsq: {
        sort_fn = (a:MappedNp, b:MappedNp) => {
          return opt_invert * (a.d_map_sq - b.d_map_sq)
        };
        break;
      }
      case SortByField.FocusDsq: {
        sort_fn = (a:MappedNp, b:MappedNp) => {
          return opt_invert * (a.focus_dsq - b.focus_dsq)
        };
        break;
      }
      case SortByField.RollErr: {
        sort_fn = (a:MappedNp, b:MappedNp) => {
          return opt_invert * (a.d_map_roll_err - b.d_map_roll_err)
        };
        break;
      }
      case SortByField.YawErr: {
        sort_fn = (a:MappedNp, b:MappedNp) => {
          return opt_invert * (a.d_map_yaw_err - b.d_map_yaw_err)
        };
        break;
      }
    }
    this.named_points.sort(sort_fn);
  }

}
