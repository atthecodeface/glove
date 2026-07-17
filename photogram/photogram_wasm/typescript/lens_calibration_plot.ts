import {
  WasmCameraInstance,
  WasmVec2f64,
  WasmVec3f64,
} from "../pkg/photogram_wasm.js";

import { HtmlElement } from "./html.js";
import { Mouse, MousePressActions } from "./mouse.js";
import { Logger } from "./log.js";
import { Draw } from "./draw.js";
import { DataRange, Plot, Tics } from "./plot.js";

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
  yaw_max: number = 90;

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

    this.div.add_button("", "", () => {
      this.plot_type = SelectedPlotType.Relative; this.redraw();
    }).add_content("Relative");
    this.div.add_button("", "", () => {
      this.plot_type = SelectedPlotType.Difference; this.redraw();
    }).add_content("Difference");
    this.div.add_button("", "", () => {
      this.plot_type = SelectedPlotType.Absolute; this.redraw();
    }).add_content("Absolute");
    this.div.add_button("", "", () => {
      this.plot_type = SelectedPlotType.Rings; this.redraw();
    }).add_content("Rings");

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

  tab_deselected(): void { }

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
      this.yaw_max = (Math.atan(cip.camera.tan_hfovd) * 180) / 3;
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
  }
  project_pm_changed(_p: Project): void {
  }
  project_camera_changed(_p: Project): void {
    this.application.set_redraw_required();
  }
  project_cip_changed(_p: Project): void {
    this.application.set_redraw_required();
  }
  project_mapped_nps_changed(_p: Project): void {
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

  generate_draw_relative_world_sensor_graph(
    camera: WasmCameraInstance,
    w: number,
    h: number,
  ): Draw {
    const size = Math.min(w, h - 230) * 0.9;
    const draw = new Draw();
    const plot = new Plot([size, size]);

    const data = new DataRange();
    for (let sensor_yaw = 0.1; sensor_yaw < this.yaw_max; sensor_yaw += 0.1) {
      const world_yaw =
        (camera.map_yaw_sensor_to_world((sensor_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data.push(sensor_yaw, world_yaw / sensor_yaw - 1);
    }
    plot.set_graph_origin([w / 2 - 0.5 * size, h / 2 + 0.5 * size]);
    const xr = data.get_xrange();
    const yr = data.get_yrange({ expand_factor: 1.2, include_zero: true });

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
    plot.generate_line_plot(draw, data);
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
    for (let world_yaw = 0; world_yaw < this.yaw_max; world_yaw += 1) {
      const sensor_yaw =
        (camera.map_yaw_world_to_sensor((world_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data0.push(sensor_yaw, world_yaw - sensor_yaw);
    }
    plot.set_graph_origin([w / 2 - 0.5 * size, h / 2 + 0.5 * size]);
    const xr = data0.get_xrange();
    const yr = data0.get_yrange({ expand_factor: 1.2, include_zero: true });

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
    plot.generate_line_plot(draw, data0);

    const mapping_nps = this.application.current_project().mapped_nps();
    mapping_nps.update();
    const data1 = new DataRange();
    for (const mnp of mapping_nps.named_points) {
      if (!mnp.has_pms) { continue; }
      this.wasm_vec2.set_array(new Float64Array([mnp.pms_x, mnp.pms_y]));
      camera.set_sensor_dir_of_pt(this.wasm_vec2, this.wasm_vec3);
      const sensor_yaw = camera.camera_yaw_of_dir(this.wasm_vec3)* 180 /   3.1416;;
      mnp.wasm_np.set_model_vec(this.wasm_vec3);
      camera.set_map_world_dir_to_camera_dir(this.wasm_vec3);
      const world_yaw = camera.camera_yaw_of_dir(this.wasm_vec3)* 180 /   3.1416;
      data1.push(sensor_yaw, world_yaw - sensor_yaw);
    }
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
    for (let world_yaw = 0; world_yaw < this.yaw_max; world_yaw += 1) {
      const sensor_yaw =
        (camera.map_yaw_world_to_sensor((world_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data0.push(sensor_yaw, world_yaw);
    }
    const data1 = new DataRange();
    for (let sensor_yaw = 0.1; sensor_yaw < this.yaw_max; sensor_yaw += 0.1) {
      const world_yaw =
        (camera.map_yaw_sensor_to_world((sensor_yaw * 3.1416) / 180) * 180) /
        3.1416;
      data1.push(sensor_yaw, world_yaw);
    }
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
    plot.generate_line_plot(draw, data0);
    plot.generate_line_plot(draw, data1);
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
    for (let yaw = 0; yaw < this.yaw_max; yaw += 1) {
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

  user_press(_xy: [number, number], _actions: MousePressActions): void {}
  user_press_move(_start_xy: [number, number], _xy: [number, number]): void {}
  user_press_cancel(_start_xy: [number, number]): void {}
  user_pan(_xy: [number, number], _dxy: [number, number]): void {}
  user_rotate(_xy: [number, number], _angle: number): void {}

  user_zoom(_cxy: [number, number], factor: number): void {
    this.yaw_max = Math.max(5.0, Math.min(90, this.yaw_max / factor));
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
