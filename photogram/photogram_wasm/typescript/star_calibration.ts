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
  WebglCanvasClient,
} from "./webgl_canvas.js";

import { Application } from "./application.js";

class ImagePoint {
  x: number = 0;
  y: number = 0;
  distance_sq(x: number, y: number): number {
    const dx = x - this.x;
    const dy = y - this.y;
    return dx * dx + dy * dy;
  }
  finished_drag() {}
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
  sel_pt: ImagePoint | null = null;
  drag_pt: ImagePoint | null = null;

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
    this.orientation_points.push(new OrientationPoint());
    this.orientation_points[0]!.x = 3000;
    this.orientation_points[0]!.y = 1000;
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
    const m_a = this.application.wasm_memory.float_array_of_mat4f64(m);

    // In this rendering, model matrix applies any transformations such as
    // rotations for animations; if these are required they operate local to the
    // object in a uniform space, so for 'orientation points' etc the actual
    // units used in model space must be uniform.
    //
    //  However, for the image itself
    // the model space is +-1 X and Y for the whole image, so it has a different
    // model matrix. This maps the points onto a z-independent XY frame
    // which has +-1 in the X for the horizontal edges of the (landscape) image,
    // and +-1/(2*image_ar) for the vertical edges of the image; i.e. post-model
    // the XY space is uniform relative to the pixels.
    //
    // The view is the identity matrix
    //
    // The projection matrix maps this uniform image space onto the viewport given the
    // current zoom and the view port aspect ratio

    // Map the ofs of the *texture* (in range 0 to 1) to the rectangle in space which is -1 to 1
    // View should be set to (+-1,+-1,z) to (zoom/ofs, zoom/ofs * (w/h), z)
    m.set_identity();
    m_a[0] = zoom;
    m_a[5] = zoom * view_ar;
    m_a[12] = zoom * (1 - ofs[0] * 2);
    m_a[13] = zoom * view_ar * (ofs[1] * 2 - 1);
    webgl.projection.set(m_a);

    m.set_identity();
    webgl.view.set(m_a);

    // Model for *image* should be set to map from (x, +-1, 0) to
    // (x,+-1/image_ar,z); this is so that the model Y of 1 maps to 1/image_ar
    // so that (post-model) it is a rectangle of the same shape as the image,
    // i.e. where the space is uniform (pixels are squares)
    webgl.use_program(webgl_canvas.image_program);
    m.set_identity();
    m_a[5] = 1 / image_ar;
    webgl.model.set(m_a);

    webgl.set_uniform_projection();
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
    webgl.set_uniform_projection();
    webgl.set_uniform_model();
    webgl.set_uniform_view();
    webgl.set_color([0.2, 1, 0.2, 1]);
    webgl.draw(this.cached_stars);

    // Plot the grid which is in image space, so needs the image-to-uniform model mapping
    webgl.use_program(webgl_canvas.image_grid_line_program);
    m.set_identity();
    m_a[5] = 1 / image_ar;
    webgl.model.set(m_a);
    webgl.set_uniform_projection();
    webgl.set_uniform_model();
    webgl.set_uniform_view();
    webgl.set_color([1.0, 1, 1.0, 1]);

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

    webgl_canvas.webgl_grid!.set_args(
      2000,
      true,
      0.2,
      (y_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(webgl_canvas.webgl_grid!);
    webgl_canvas.webgl_grid!.set_args(
      2000,
      false,
      0.2,
      (x_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(webgl_canvas.webgl_grid!);

    if (grid_spacing_is_pwr_of_ten) {
      grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
      grid_line_spacing_in_px *= 5;
    } else {
      grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
      grid_line_spacing_in_px *= 2;
    }
    webgl_canvas.webgl_grid!.set_args(
      2000,
      true,
      0.7,
      (y_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(webgl_canvas.webgl_grid!);
    webgl_canvas.webgl_grid!.set_args(
      2000,
      false,
      0.7,
      (x_space_of_1000px / 1000) * grid_line_spacing_in_px,
      -1000,
    );
    webgl.draw(webgl_canvas.webgl_grid!);

    // Plot the axes as *bright*
    webgl_canvas.webgl_grid!.set_args(1, true, 1, 0, 0);
    webgl.draw(webgl_canvas.webgl_grid!);
    webgl_canvas.webgl_grid!.set_args(1, false, 1, 0, 0);
    webgl.draw(webgl_canvas.webgl_grid!);

    // Plot the interesting points etc
    //
    // The animations must occur in 'uniform' space (1 pixel to 1 pixel), so
    // that rotations appear correctly. This means that any subsequent
    // translation (to put the animated points at the correct positions) is in
    // uniform +-1,+-1/image_ar space.
    //
    // The subsequent translation is accomplished here with the view matrix (as otherwise it is the identity)
    //
    webgl.use_program(webgl_canvas.flat_program);
    // Apply rotation for animation
    webgl.set_uniform_projection();
    m.set_identity();
    m.set_scale3(0.03 / zoom);
    webgl.model.set(m_a);
    webgl.set_uniform_model();

    m.set_identity();
    webgl.set_color([1.0, 1, 0.0, 1]);
    for (const o of this.orientation_points) {
      const uxy = this.uniform_xy_of_img_xy(o.x, o.y);
      m_a[3] = uxy[0];
      m_a[7] = uxy[1];
      webgl.view.set(m_a);
      webgl.set_uniform_view();
      webgl.draw(webgl_canvas.webgl_asterisk!);
    }
  }

  uniform_xy_of_img_xy(x: number, y: number): [number, number] {
    return [
      (2 * x) / this.img_size[0] - 1,
      (2 * y - this.img_size[1]) / this.img_size[0],
    ];
  }

  img_xy_of_uniform_xy(x: number, y: number): [number, number] {
    return [
      ((x + 1) * this.img_size[0]) / 2,
      (y * this.img_size[0] + this.img_size[1]) / 2,
    ];
  }

  /** Map from a screen XY to a +-1 uniform XY (i.e. the uniform space that the image rectangle and stars are displayed into) */
  uniform_xy_of_scr_xy(x: number, y: number): [number, number] {
    // screen is 0->width, 0->height coordinates, effectively nonuniform 'squished')
    // scr_rel is +-0.5,+-0.5 coordinates (units of uniform, not screen-squished)
    let scr_rel_x = x / this.zoomed_window.get_scr_wh()[0] - 0.5;
    let scr_rel_y =
      (y - this.zoomed_window.get_scr_wh()[1] / 2) /
      this.zoomed_window.get_scr_wh()[0];

    // Uniform is unzoom of scr_rel with the scroll offset, rescaled to +-1 instead of +-0.5
    let uni_x =
      2 *
        (scr_rel_x / this.zoomed_window.get_zoom() +
          this.zoomed_window.rel_cxy()[0]) -
      1;

    let uni_y =
      1 -
      2 *
        (scr_rel_y / this.zoomed_window.get_zoom() +
          this.zoomed_window.rel_cxy()[1]);

    return [uni_x, uni_y];
  }

  /** Map from a +-1 uniform XY  to a screen XY */
  scr_xy_of_uniform_xy(x: number, y: number): [number, number] {
    // Uniform is in +-1 space; map this to 0->1 then apply inverse window offset, then zoom
    //
    // scr_rel is +-0.5,+-0.5 coordinates (units of uniform, not screen-squished)
    let scr_rel_x =
      this.zoomed_window.get_zoom() *
      ((x + 1) / 2 - this.zoomed_window.rel_cxy()[0]);
    let scr_rel_y =
      this.zoomed_window.get_zoom() *
      ((1 - y) / 2 - this.zoomed_window.rel_cxy()[1]);

    let scr_x = (scr_rel_x + 0.5) * this.zoomed_window.get_scr_wh()[0];
    let scr_y =
      scr_rel_y * this.zoomed_window.get_scr_wh()[0] +
      this.zoomed_window.get_scr_wh()[1] / 2;
    return [scr_x, scr_y];
  }

  find_closest_point(
    img_x: number,
    img_y: number,
  ): [ImagePoint, number] | null {
    let min_dsq = 1e16;
    let pt = null;
    for (const o of this.orientation_points) {
      const dsq = o.distance_sq(img_x, img_y);
      if (dsq < min_dsq) {
        pt = o;
        min_dsq = dsq;
      }
    }
    if (pt === null) {
      return null;
    }
    return [pt, min_dsq];
  }

  user_press(xy: [number, number], actions: MousePressActions): void {
    const uni_xy = this.uniform_xy_of_scr_xy(xy[0], xy[1]);
    const img_xy = this.img_xy_of_uniform_xy(uni_xy[0], uni_xy[1]);
    const pt_dsq = this.find_closest_point(img_xy[0], img_xy[1]);
    if (pt_dsq !== null) {
      const pt = pt_dsq[0];
      const uxy = this.uniform_xy_of_img_xy(pt.x, pt.y);
      const sxy = this.scr_xy_of_uniform_xy(uxy[0], uxy[1]);
      if (Math.abs(sxy[0] - xy[0]) < 10 && Math.abs(sxy[1] - xy[1]) < 10) {
        this.sel_pt = pt;
        actions.can_pan = false;
        actions.can_drag = true;
        return;
      }
    }
    this.sel_pt = null;
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

  drag_start(_start_xy: [number, number], _xy: [number, number]): void {
    if (this.sel_pt !== null) {
      this.drag_pt = this.sel_pt;
      this.sel_pt = null;
    }
  }

  drag_to(
    _start_xy: [number, number],
    _old_xy: [number, number],
    xy: [number, number],
  ): void {
    if (this.drag_pt !== null) {
      const uni_xy = this.uniform_xy_of_scr_xy(xy[0], xy[1]);
      const img_xy = this.img_xy_of_uniform_xy(uni_xy[0], uni_xy[1]);
      this.drag_pt.x = img_xy[0];
      this.drag_pt.y = img_xy[1];
      this.application.set_view_needs_update();
    }
  }

  drag_end(_start_xy: [number, number], _xy: [number, number]): void {
    if (this.drag_pt !== null) {
      this.drag_pt.finished_drag();
    }
    this.drag_pt = null;
  }

  user_release(_start_xy: [number, number], _xy: [number, number]): void {}
}
