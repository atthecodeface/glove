import {
  WasmCatalog,
  WasmMat4f64,
  WasmStar,
  WasmVec3f64,
} from "../pkg/photogram_wasm.js";

import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";
import { MousePressActions } from "./mouse.js";
import { Webgl } from "./web_gl.js";
import { ZoomedWindow } from "./zoomed_window.js";

import {
  WebglCanvas,
  StarsWebglObj,
  GridLinesObj,
  WebglCanvasClient,
} from "./webgl_canvas.js";

import { Application } from "./application.js";

class ImagePoint {
  x: number = 0;
  y: number = 0;
}

class OrientationPoint extends ImagePoint {}

export class StarCalibration implements WebglCanvasClient {
  application: Application;
  log: Logger;
  html_div: HtmlElement;
  zoomed_window: ZoomedWindow;
  star_catalog: WasmCatalog = new WasmCatalog("hipp_bright");
  wasm_star: WasmStar;
  wasm_vec: WasmVec3f64;

  selected_star_indices: number[] = [];
  cached_stars: StarsWebglObj;
  img_size: [number, number] = [0, 0];

  orientation_points: OrientationPoint[] = [];

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;
    this.wasm_star = this.star_catalog.star(0)!;
    this.wasm_vec = new WasmVec3f64(0, 0, 1);

    this.zoomed_window = new ZoomedWindow([10, 10]);

    this.star_catalog.clear_filter();
    this.star_catalog.filter_max_magnitude(6);
    this.selected_star_indices = this.star_catalog.find_stars_around(
      this.wasm_vec,
      (40 / 180.0) * 3.1415,
      0,
      1000,
    );
    this.cached_stars = new StarsWebglObj();
  }

  webgl_create(webgl: Webgl, _webgl_canvas: WebglCanvas): void {
    this.cached_stars.webgl_create(webgl.webgl!);
  }

  webgl_resize(w: number, h: number) {
    this.zoomed_window.scr_resize(w, h);
  }

  /** Map the selected stars through the camera orientation and calibration to sensor space x,y,1 */
  map_of_selected_stars(): Float32Array {
    const cip = this.application.current_project().get_wasm_cip();
    const pts = new Float32Array(this.selected_star_indices.length * 4);
    if (cip === null) {
      return pts;
    }
    const camera = cip.camera;
    const vxyz = this.application.wasm_memory.float_array_of_vec3f64(
      this.wasm_vec,
    );
    let i = 0;
    for (const s of this.selected_star_indices) {
      this.star_catalog.set_star(this.wasm_star, s);
      this.wasm_star.set_vector(this.wasm_vec);
      // wasm_vec is a unit world direction
      camera.set_map_camera_dir_to_sensor_dir(this.wasm_vec);
      pts.set([vxyz[0]!, vxyz[1]!, vxyz[2]!, 0], i * 4);
      i += 1;
    }
    return pts;
  }

  webgl_redraw(webgl: Webgl, webgl_canvas: WebglCanvas): void {
    const pts = this.map_of_selected_stars();
    this.cached_stars.set_position_data(webgl.webgl!, pts.length / 4, pts);

    const size = webgl_canvas.size();
    const view_ar = size[0] / size[1];

    // Set the whole canvas as viewport
    webgl.set_viewport([0, 0, 0, 0]);
    webgl.clear_buffer({ depth_test: false });

    const project = this.application.current_project();
    const cip = project.get_cip();
    if (!cip.is_valid()) {
      return;
    }
    const camera = cip.wasm_cip!.camera;

    const texture_ready = cip.cip_image.webgl_texture_ready();
    const texture = cip.cip_image.get_webgl_texture(webgl);
    if (texture === null) {
      return;
    }

    if (!texture_ready) {
      this.img_size = cip.cip_image.get_size();
      this.zoomed_window.set_img(this.img_size[0], this.img_size[1]);
    }

    const image_ar = this.img_size[0] / this.img_size[1];
    const zoom = this.zoomed_window.get_zoom();
    const ofs = this.zoomed_window.rel_cxy();

    const m = WasmMat4f64.identity();
    m.set_identity();
    const m_a = this.application.wasm_memory.float_array_of_mat4f64(m);

    // In this rendering, model maps the points onto a z-independent XY frame
    // which has +-1 in the X for the horizontal edges of the (landscape) image,
    // and +-1/(2*image_ar) for the vertical edges of the image.
    //
    // The view matrix maps this uniform image space onto the viewport given the
    // current zoom and the view port aspect ratio

    // Map the ofs of the *texture* (in range 0 to 1) to the rectangle in space which is -1 to 1
    // View should be set to (+-1,+-1,z) to (zoom/ofs, zoom/ofs * (w/h), z)
    m_a[0] = zoom;
    m_a[3] = zoom * (1 - ofs[0] * 2);
    m_a[5] = zoom * view_ar;
    m_a[7] = zoom * view_ar * (ofs[1] * 2 - 1);
    webgl.view.set(m_a);

    // Model for *image* should be set to map from (x, +-1, 0) to
    // (x,+-1/image_ar,z); this is so that the model Y of 1 maps to 1/image_ar
    // so that (post-model) it is a rectangle of the same shape as the image,
    // i.e. where the space is uniform (pixels are squares)
    webgl.use_program(webgl_canvas.image_program);
    m.set_identity();
    m_a[5] = 1 / image_ar;
    webgl.model.set(m_a);

    webgl.set_uniform_model();
    webgl.set_uniform_view();
    webgl.set_color([1, 1, 1, 1]);
    webgl.set_texture(texture);
    webgl.draw(webgl_canvas.webgl_rectangle!);

    // Model for *stars* should be set to map from (+-1, +-1, 0) to (+-1/tanhfov, +-1/1/tanhfov)
    webgl.use_program(webgl_canvas.star_program);
    const tan_hfovh = camera.tan_hfovh;
    m.set_identity();
    m_a[0] = 1 / tan_hfovh;
    m_a[5] = 1 / tan_hfovh;
    webgl.model.set(m_a);
    webgl.set_uniform_model();
    webgl.set_uniform_view();
    webgl.set_color([0.2, 1, 0.2, 1]);
    webgl.draw(this.cached_stars);

    // Plot the interesting points etc; these are in the +-1 in the X/Y space which map through the model to (+-1,+-1/image_ar,z)
    webgl.use_program(webgl_canvas.image_grid_line_program);
    m.set_identity();
    m_a[5] = 1 / image_ar;
    webgl.model.set(m_a);
    webgl.set_uniform_model();
    webgl.set_uniform_view();
    webgl.set_color([1.0, 1, 1.0, 1]);
    const c = new GridLinesObj(2, 2);
    webgl.create(c);

    const img_px_visible = this.img_size[0] / zoom;
    // const blah = zoom * (1 - ofs[0] * 2);
    const x_space_of_1000px = (2 * 1000) / this.img_size[0];
    const y_space_of_1000px = (2 * 1000) / this.img_size[1];
    let grid_line_spacing_in_px = 1;
    let grid_spacing_is_pwr_of_ten = true;
    for (let i = 0; i < 5; i++) {
      if (img_px_visible / grid_line_spacing_in_px >= 25) {
        grid_line_spacing_in_px *= 5;
        grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
      }
      if (img_px_visible / grid_line_spacing_in_px >= 25) {
        grid_line_spacing_in_px *= 2;
        grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
      }
    }

    c.set_args(
      2000,
      true,
      0.2,
      (y_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(c);
    c.set_args(
      2000,
      false,
      0.2,
      (x_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(c);

    if (grid_spacing_is_pwr_of_ten) {
      grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
      grid_line_spacing_in_px *= 5;
    } else {
      grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
      grid_line_spacing_in_px *= 2;
    }
    c.set_args(
      2000,
      true,
      0.7,
      (y_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(c);
    c.set_args(
      2000,
      false,
      0.7,
      (x_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(c);

    // Plot the axes as *bright*
    c.set_args(1, true, 1, 0, 0);
    webgl.draw(c);
    c.set_args(1, false, 1, 0, 0);
    webgl.draw(c);
  }

  user_press(_xy: [number, number], actions: MousePressActions): void {
    actions.can_pan = true;
    actions.can_drag = false;
  }
  user_press_move(_start_xy: [number, number], _xy: [number, number]): void {}
  user_press_cancel(_start_xy: [number, number]): void {}

  user_rotate(_xy: [number, number], _angle: number): void {}

  user_pan(xy: [number, number], dxy: [number, number]): void {
    this.zoomed_window.user_pan(xy, dxy);
    this.application.set_view_needs_update();
  }
  user_zoom(cxy: [number, number], factor: number): void {
    this.zoomed_window.user_zoom(cxy, factor);
    this.application.set_view_needs_update();
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
