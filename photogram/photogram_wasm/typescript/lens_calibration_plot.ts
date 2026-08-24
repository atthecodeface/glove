// photogram.star_calibration.camera.set_lens_poly(photogram.lens_calibration_plot.lens_poly)

import {
  WasmCameraInstance,
  WasmLensPoly,
  WasmVec2f64,
  WasmVec3f64,
} from "../pkg/photogram_wasm.js";

import { HtmlElement } from "./html.js";
import { Mouse, MousePressActions } from "./mouse.js";
import { Logger } from "./log.js";
import { Draw } from "./draw.js";
import { DataPoint, DataRange, DataXY, DataXYC, Plot, Tics } from "./plot.js";

import { Application, ApplicationTab } from "./application.js";
import { Project, ProjectClient } from "./project.js";

enum SelectedPlotType {
  Relative,
  Absolute,
  Difference,
  Rings,
}

export class LensCalibrationPlot implements ApplicationTab, ProjectClient {
  application: Application;
  log: Logger;
  div: HtmlElement;
  canvas: HTMLCanvasElement;
  mouse: Mouse;
  camera: WasmCameraInstance | null = null;

  plot_type: SelectedPlotType = SelectedPlotType.Relative;
  view_bounds: [number, number] = [0, 1];
  world_yaw_max: number = 90;
  sensor_yaw_max: number = 90;

  pending_regen: boolean = false;
  draw_world_rings_in_frame: Draw;
  draw_relative_world_sensor_graph: Draw;
  draw_world_sensor_graphs: Draw;
  draw_ws_difference_graph: Draw;

  constructor(application: Application, log: Logger, div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.div = div;
    this.div.clear();
    this.div.add_label(undefined, { classes: "set_fovh" });

    this.div
      .add_button("", "", () => {
        this.plot_type = SelectedPlotType.Relative;
        this.redraw();
      })
      .add_content("Relative");
    this.div
      .add_button("", "", () => {
        this.plot_type = SelectedPlotType.Difference;
        this.redraw();
      })
      .add_content("Difference");
    this.div
      .add_button("", "", () => {
        this.plot_type = SelectedPlotType.Absolute;
        this.redraw();
      })
      .add_content("Absolute");
    this.div
      .add_button("", "", () => {
        this.plot_type = SelectedPlotType.Rings;
        this.redraw();
      })
      .add_content("Rings");

    this.canvas = this.div.add_ele("canvas").ele as HTMLCanvasElement;
    this.mouse = new Mouse(this, this.canvas);

    this.draw_world_rings_in_frame = new Draw();
    this.draw_world_sensor_graphs = new Draw();
    this.draw_relative_world_sensor_graph = new Draw();
    this.draw_ws_difference_graph = new Draw();
    application.add_tab(this, null);
  }

  tab_name(): string {
    return "lens-calibration-plot";
  }

  tab_text(): string {
    return "Lens Calibration";
  }

  tab_deselected(): void {}

  tab_selected(): void {
    const wh = this.application.get_resizable_content_size();
    this.tab_resize(wh[0], wh[1]);
  }

  tab_project_selected(p: Project): void {
    p.add_client(this);
  }

  tab_project_updated(): void {
    const cip = this.application.current_project().get_wasm_cip();
    if (cip !== null) {
      this.camera = cip.camera;
      const d = Math.sqrt(cip.camera.sensor_mm_width * cip.camera.sensor_mm_width + cip.camera.sensor_mm_height * cip.camera.sensor_mm_height);
      this.sensor_yaw_max = (Math.atan(d/2/cip.camera.lens_sensor_distance) * 180) / 3.14;
      this.world_yaw_max = 89;  // this.camera.map_yaw_sensor_to_world(this.sensor_yaw_max*3.14/180)*180/3.14;
      this.pending_regen = true;
      this.application.set_redraw_required();
    }
  }

  /** Invoked when the tab is selected, or just prior to redraw if screen has changed size */
  tab_resize(w: number, h: number) {
    this.canvas.width = w;
    this.canvas.height = h;
    this.pending_regen = true;
  }

  tab_redraw() {
    this.redraw();
  }

  project_np_changed(_p: Project): void {
    this.pending_regen = true;
  }
  project_pm_changed(_p: Project): void {
    this.pending_regen = true;
  }
  project_camera_changed(_p: Project): void {
    this.pending_regen = true;
    this.application.set_redraw_required();
  }
  project_cip_changed(_p: Project): void {
    this.pending_regen = true;
    this.application.set_redraw_required();
  }
  project_mapped_nps_changed(_p: Project): void {
    this.pending_regen = true;
  }

  pms_world_sensor_pairs: [number, number, string][] = [];
  lens_poly: WasmLensPoly = new WasmLensPoly("rectilinear");
  generate_pms_world_sensor_pairs() {
    this.pms_world_sensor_pairs = [];
    if (this.camera === null) {
      this.lens_poly = new WasmLensPoly("rectilinear");
      return;
    }
    const camera = this.camera;
    const mapping_nps = this.application.current_project().mapped_nps();
    mapping_nps.update();

    let world_yaws = [];
    let sensor_yaws = [];
    let max_sensor_yaw = 0.0;
    for (const mnp of mapping_nps.named_points) {
      if (!mnp.has_pms()) {
        continue;
      }

      // sensor_yaw is given by the Yaw of the *mapped* point, which is based purely on the sensor geometry not the lens calibration
      mnp.wasm_pms.set_image_vec(this.wasm_vec2);
      camera.set_sensor_dir_of_pt(this.wasm_vec2, this.wasm_vec3);
      const sensor_yaw =
        (camera.camera_yaw_of_dir(this.wasm_vec3) * 180) / 3.1416;

      // world_yaw is given by the Yaw of the direction vector, which is based on the camera orientation only and not the lens calibration
      mnp.wasm_pms.np_model_set_vec(this.wasm_vec3);
      camera.set_map_world_dir_to_camera_dir(this.wasm_vec3);
      const world_yaw =
        (camera.camera_yaw_of_dir(this.wasm_vec3) * 180) / 3.1416;
      const color = mnp.wasm_pms.np_color;
      this.pms_world_sensor_pairs.push([world_yaw, sensor_yaw, color]);

      world_yaws.push(world_yaw/180*3.14);
      sensor_yaws.push(sensor_yaw / 180 * 3.14);
      max_sensor_yaw = Math.max(max_sensor_yaw, sensor_yaw / 180 * 3.14);
    }
    this.lens_poly =  WasmLensPoly.of_calibration(new Float64Array(sensor_yaws), new Float64Array(world_yaws), 0.2*3.14/180, max_sensor_yaw*1.05);
  }

  redraw() {
    const context = this.canvas.getContext("2d")!;
    context.fillStyle = "black";
    const w = this.canvas.width;
    const h = this.canvas.height;
    context.fillRect(0, 0, w, h);
    if (this.camera === null) {
      return;
    }
    if (this.pending_regen) {
      this.pending_regen = false;
      this.generate_pms_world_sensor_pairs();

      this.draw_world_rings_in_frame = this.generate_draw_world_rings_in_frame(
        this.camera,
        w,
        h,
      );
      this.draw_world_sensor_graphs = this.generate_draw_world_sensor_graphs(
        this.camera,
        w,
        h,
      );
      this.draw_ws_difference_graph = this.generate_draw_ws_difference_graph(
        this.camera,
        w,
        h,
      );

      this.draw_relative_world_sensor_graph =
        this.generate_draw_relative_world_sensor_graph(this.camera, w, h);
    }

    context.font = "24px serif";
    context.fillStyle = "#FFF";
    context.textAlign = "center";

    switch (this.plot_type) {
      case SelectedPlotType.Absolute: {
        this.draw_world_sensor_graphs.draw(context, (x) => x);
        break;
      }
      case SelectedPlotType.Relative: {
        this.draw_relative_world_sensor_graph.draw(context, (x) => x);
        break;
      }
      case SelectedPlotType.Difference: {
        this.draw_ws_difference_graph.draw(context, (x) => x);
        break;
      }
      case SelectedPlotType.Rings: {
        this.draw_world_rings_in_frame.draw(context, (x) => x);
        break;
      }
    }
  }

  filter_data_range<T extends DataPoint>(data: DataRange<T>) {
    const min_x = this.view_bounds[0] * this.sensor_yaw_max;
    const max_x = this.view_bounds[1] * this.sensor_yaw_max;
    const filter = (d: DataPoint) => { return (d.x() >= min_x) && (d.x() <= max_x); };
    data.filter_data(filter);
  }

  generate_draw_relative_world_sensor_graph(
    camera: WasmCameraInstance,
    w: number,
    h: number,
  ): Draw {
    const size = Math.min(w, h - 230) * 0.9;
    const draw = new Draw();
    const plot = new Plot([size, size]);

    const data0 = new DataRange();
    const data2 = new DataRange();
    for (let sensor_yaw = 0.1; sensor_yaw < this.sensor_yaw_max; sensor_yaw += 0.1) {
      const world_yaw =
        (camera.map_yaw_sensor_to_world((sensor_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data0.push(new DataXY(sensor_yaw, world_yaw));
      data2.push(new DataXY(sensor_yaw, this.lens_poly.stw(sensor_yaw/180*3.1415)*180/3.1415));
    }

    const data1 = new DataRange();
    for (const [world_yaw, sensor_yaw, color] of this.pms_world_sensor_pairs) {
      data1.push(new DataXYC(sensor_yaw, world_yaw, color));
    }
    this.filter_data_range(data0);
    this.filter_data_range(data1);
    this.filter_data_range(data2);

    for (const d of data0.data) {
      d.set_y(d.y() / d.x() - 1);
    }
    for (const d of data1.data) {
      d.set_y(d.y() / d.x() - 1);
    }
    for (const d of data2.data) {
      d.set_y(d.y() / d.x() - 1);
    }


    const include_zero = this.view_bounds[1] - this.view_bounds[0] > 0.9999;
    plot.set_graph_origin([w / 2 - 0.5 * size, h / 2 + 0.5 * size]);
    const xr = data0.get_xrange();
    const yr0 = data0.get_yrange({ expand_factor: 1.2, include_zero: include_zero });
    const yr1 = data1.get_yrange({ expand_factor: 1.2, include_zero: include_zero });
    const yr: [number, number] = [
      Math.min(yr0[0], yr1[0]),
      Math.max(yr0[1], yr1[1]),
    ];

    const xtics = new Tics({
      spacing: 10,
      length: 10,
      show_grid: true,
      label: true,
    });
    const ytics = new Tics({
      spacing: 10,
      length: 10,
      show_grid: true,
      label: true,
    });
    xtics.set_spacing_of_range(xr, 2);
    ytics.set_spacing_of_range(yr, 2);

    plot.xtics.push(xtics);
    plot.ytics.push(ytics);
    plot.set_data_range(xr[0], yr[0], xr[1], yr[1]);

    plot.generate_grid(draw);
    plot.generate_tics(draw);
    plot.generate_labels(draw);
    plot.generate_line_plot(draw, data0, "#FF8");
    plot.generate_line_plot(draw, data2, "#FAA");
    plot.generate_pt_plot(draw, data1);
    plot.generate_box(draw);
    return draw;
  }

  generate_draw_ws_difference_graph(
    camera: WasmCameraInstance,
    w: number,
    h: number,
  ): Draw {

    const draw = new Draw();
    const size = Math.min(w, h - 230) * 0.9;
    const plot = new Plot([size, size]);

    const data0 = new DataRange();
    const data2 = new DataRange();
    for (let sensor_yaw = 0.1; sensor_yaw < this.sensor_yaw_max; sensor_yaw += 0.1) {
      const world_yaw =
        (camera.map_yaw_sensor_to_world((sensor_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data0.push(new DataXY(sensor_yaw, world_yaw));
      data2.push(new DataXY(sensor_yaw, this.lens_poly.stw(sensor_yaw/180*3.1415)*180/3.1415));
    }

    const data1 = new DataRange();
    for (const [world_yaw, sensor_yaw, color] of this.pms_world_sensor_pairs) {
      data1.push(new DataXYC(sensor_yaw, world_yaw, color));
    }
    this.filter_data_range(data0);
    this.filter_data_range(data1);
    this.filter_data_range(data2);

    for (const d of data0.data) {
      d.set_y(d.y() - d.x());
    }
    for (const d of data1.data) {
      d.set_y(d.y() - d.x());
    }
    for (const d of data2.data) {
      d.set_y(d.y() - d.x());
    }

    const include_zero = this.view_bounds[1] - this.view_bounds[0] > 0.9999;
    plot.set_graph_origin([w / 2 - 0.5 * size, h / 2 + 0.5 * size]);
    const xr = data0.get_xrange();
    const yr0 = data0.get_yrange({ expand_factor: 1.2, include_zero: include_zero });
    const yr1 = data1.get_yrange({ expand_factor: 1.2, include_zero: include_zero });
    const yr: [number, number] = [
      Math.min(yr0[0], yr1[0]),
      Math.max(yr0[1], yr1[1]),
    ];

    let xtics = new Tics({
      spacing: 10,
      length: 10,
      show_grid: true,
      label: true,
    });
    let ytics = new Tics({
      spacing: 0.1,
      length: 10,
      show_grid: true,
      label: true,
    });
    xtics.set_spacing_of_range(xr, 2);
    ytics.set_spacing_of_range(yr, 2);

    plot.xtics.push(xtics);
    plot.ytics.push(ytics);
    plot.set_data_range(xr[0], yr[0], xr[1], yr[1]);

    plot.generate_grid(draw);
    plot.generate_tics(draw);
    plot.generate_labels(draw);
    plot.generate_line_plot(draw, data0, "#FF8");
    plot.generate_line_plot(draw, data2, "#FAA");
    plot.generate_pt_plot(draw, data1);

    plot.generate_box(draw);
    return draw;
  }
  wasm_vec2: WasmVec2f64 = WasmVec2f64.zero();
  wasm_vec3: WasmVec3f64 = WasmVec3f64.zero();

  generate_draw_world_sensor_graphs(
    camera: WasmCameraInstance,
    w: number,
    h: number,
  ): Draw {
    const draw = new Draw();
    const size = Math.min(w, h - 230) * 0.9;
    const plot = new Plot([size, size]);

    const data0 = new DataRange();
    for (let world_yaw = 0; world_yaw < this.world_yaw_max; world_yaw += 0.01) {
      const sensor_yaw =
        (camera.map_yaw_world_to_sensor((world_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data0.push(new DataXY(sensor_yaw, world_yaw));
    }
    const data1 = new DataRange();
    for (let sensor_yaw = 0.1; sensor_yaw < this.sensor_yaw_max; sensor_yaw += 0.01) {
      const world_yaw =
        (camera.map_yaw_sensor_to_world((sensor_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data1.push(new DataXY(sensor_yaw, world_yaw));
    }

    this.filter_data_range(data0);
    this.filter_data_range(data1);

    plot.set_graph_origin([w / 2 - 0.5 * size, h / 2 + 0.5 * size]);
    const xr = data0.get_xrange();
    const yr = data0.get_yrange();

    let tics = new Tics({
      spacing: 10,
      length: 10,
      show_grid: true,
      label: true,
    });
    tics.set_spacing_of_range(xr, 2);

    plot.xtics.push(tics);
    plot.ytics.push(tics);
    plot.set_data_range(xr[0], yr[0], xr[1], yr[1]);

    plot.generate_grid(draw);
    plot.generate_tics(draw);
    plot.generate_labels(draw);
    plot.generate_line_plot(draw, data0, "#FF8");
    plot.generate_line_plot(draw, data1, "#FAA");
    plot.generate_box(draw);
    return draw;
  }

  generate_draw_world_rings_in_frame(
    camera: WasmCameraInstance,
    w: number,
    h: number,
  ): Draw {
    const sensor_wh: [number, number] = [
      camera.sensor_px_width,
      camera.sensor_px_height,
    ];

    const draw = new Draw();

    const context_sensor_cxy: [number, number] = [w / 2, h / 2];
    const sensor_to_context_sc = (0.9 * w) / sensor_wh[0];
    const context_sensor_bbox: [number, number, number, number] = [
      context_sensor_cxy[0] - (sensor_wh[0] * sensor_to_context_sc) / 2,
      context_sensor_cxy[1] - (sensor_wh[1] * sensor_to_context_sc) / 2,
      context_sensor_cxy[0] + (sensor_wh[0] * sensor_to_context_sc) / 2,
      context_sensor_cxy[1] + (sensor_wh[1] * sensor_to_context_sc) / 2,
    ];

    draw.extend([
      ["W", 4.0],
      ["S", "#f33"],
      ["b"],
      ["m", context_sensor_bbox[0], context_sensor_bbox[1]],
      ["l", context_sensor_bbox[2], context_sensor_bbox[1]],
      ["l", context_sensor_bbox[2], context_sensor_bbox[3]],
      ["l", context_sensor_bbox[0], context_sensor_bbox[3]],
      ["l", context_sensor_bbox[0], context_sensor_bbox[1]],
      ["s"],
    ]);

    draw.extend([
      ["W", 1.0],
      ["S", "#722"],
    ]);

    const d = w / 20;
    for (let i = 0; i <= 10; i += 1) {
      draw.extend([
        ["b"],
        ["m", context_sensor_cxy[0] + i * d, context_sensor_bbox[1]],
        ["L", 0, sensor_wh[1] * sensor_to_context_sc],
        ["s"],
      ]);
      draw.extend([
        ["b"],
        ["m", context_sensor_cxy[0] - i * d, context_sensor_bbox[1]],
        ["L", 0, sensor_wh[1] * sensor_to_context_sc],
        ["s"],
      ]);

      if (i * d <= (sensor_wh[1] * sensor_to_context_sc) / 2) {
        draw.extend([
          ["b"],
          ["m", context_sensor_bbox[0], context_sensor_cxy[1] + i * d],
          ["L", sensor_wh[0] * sensor_to_context_sc, 0],
          ["s"],
        ]);
        draw.extend([
          ["b"],
          ["m", context_sensor_bbox[0], context_sensor_cxy[1] - i * d],
          ["L", sensor_wh[0] * sensor_to_context_sc, 0],
          ["s"],
        ]);
      }
    }

    draw.extend([["W", 1.0]]);
    const dir = new WasmVec2f64(0, 0);
    for (let yaw = 0; yaw < this.world_yaw_max; yaw += 1) {
      let sin_yaw = Math.sin((yaw * 3.1416) / 180);
      let cos_yaw = Math.cos((yaw * 3.1416) / 180);

      const hue = yaw * 4 + 60;
      let l = 50;
      if (yaw % 5 == 0) {
        l = 0;
      }
      const color = `hwb(${hue} 0 ${l})`;
      draw.extend([["S", color], ["b"]]);

      for (let roll = 0; roll <= 360; roll += 5) {
        let cos_roll = Math.cos((roll * 3.1416) / 180);
        let sin_roll = Math.sin((roll * 3.1416) / 180);

        let x = sin_yaw * cos_roll;
        let y = sin_yaw * sin_roll;
        let z = cos_yaw;
        camera.set_pt_of_camera_dir(new WasmVec3f64(x, y, z), dir);
        draw.extend([
          [
            "l",
            ((dir.array[0]! - sensor_wh[0] / 2) / sensor_wh[0]) * w + w / 2,
            ((dir.array[1]! - sensor_wh[1] / 2) / sensor_wh[0]) * w + h / 2,
          ],
        ]);
      }
      draw.extend([["s"]]);
    }
    return draw;
  }

  user_press(_xy: [number, number], actions: MousePressActions): void {
    actions.can_pan = true;
    actions.can_drag = false;
    console.log(actions);
  }
  user_press_move(_start_xy: [number, number], _xy: [number, number]): void {}
  user_press_cancel(_start_xy: [number, number]): void {}
  user_pan(_xy: [number, number], dxy: [number, number]): void {

    const w = this.canvas.width;
    const c_dx = dxy[0] / w;
    const vcx = (this.view_bounds[1] + this.view_bounds[0]) / 2;
    const vdx = (this.view_bounds[1] - this.view_bounds[0]);
    const v_dx = c_dx * vdx;
    let new_vcx = vcx + v_dx;
    new_vcx = Math.min(1 - vdx / 2, new_vcx);
    new_vcx = Math.max(vdx / 2, new_vcx);
    this.view_bounds = [new_vcx - vdx / 2, new_vcx + vdx / 2];
    this.pending_regen = true;
    this.redraw();
  }
  user_rotate(_xy: [number, number], _angle: number): void {}

  user_zoom(cxy: [number, number], factor: number): void {
    const w = this.canvas.width;
    const c_fx = cxy[0] / w - 0.5;
    const vcx = (this.view_bounds[1] + this.view_bounds[0]) / 2;
    const vdx = (this.view_bounds[1] - this.view_bounds[0]);
    const v_fx = c_fx * vdx + vcx;
    const new_vdx = Math.min(Math.max(vdx / factor, 0.01), 1);
    let new_vcx = v_fx - c_fx * new_vdx;
    new_vcx = Math.min(1 - new_vdx / 2, new_vcx);
    new_vcx = Math.max(new_vdx / 2, new_vcx);
    this.view_bounds = [new_vcx - new_vdx / 2, new_vcx + new_vdx / 2];
    this.pending_regen = true;
    this.redraw();
  }

  drag_start(_start_xy: [number, number], _xy: [number, number]): void {}

  drag_to(
    _start_xy: [number, number],
    _old_xy: [number, number],
    _new_xy: [number, number],
  ): void {}

  drag_end(_start_xy: [number, number], _xy: [number, number]): void {}

  user_release(_start_xy: [number, number], _xy: [number, number]): void {}
}
