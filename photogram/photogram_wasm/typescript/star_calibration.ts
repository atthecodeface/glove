import {
  WasmCatalog,
  WasmMat4f64,
  WasmQuatf64,
  WasmStar,
  WasmVec3f64,
} from "../pkg/photogram_wasm.js";

import { Animate } from "./animate.js";
import { color_choice_as_rgb, rgb_of_color } from "./color.js";
import { HtmlElement, Table } from "./html.js";
import { Logger } from "./log.js";
import { MousePressActions } from "./mouse.js";
import { ToolsDialog, ToolsDialogClient } from "./tools_dialog.js";
import { Webgl } from "./web_gl.js";
import { ZoomedWindow } from "./zoomed_window.js";

import {
  WebglCanvas,
  StarsWebglObj,
  WebglCanvasClient,
} from "./webgl_canvas.js";

import { Application } from "./application.js";
import { Tabs } from "./tabs.js";
import { MappedNpClient, MappedNp } from "./mapped_nps.js";
import { Project, ProjectClient } from "./project.js";

class ImagePoint {
  x: number = 0;
  y: number = 0;
  movable: boolean = true;
  distance_sq(x: number, y: number): number {
    if (!this.movable) {
      return 1E6;
    }
    const dx = x - this.x;
    const dy = y - this.y;
    return dx * dx + dy * dy;
  }
  finished_drag(_parent:StarCalibration, _xy: [number, number]) {}
  draw(_webgl: Webgl, _webgl_canvas: WebglCanvas): void {}
}

class Cursor extends ImagePoint {
  override draw(webgl: Webgl, webgl_canvas: WebglCanvas): void {
    webgl.set_color([1,1,1, 1]);
    webgl.draw(webgl_canvas.webgl_asterisk!);
  }
  override finished_drag(parent:StarCalibration, xy: [number, number]) {
    parent.cursor_move_complete(xy[0], xy[1]);
  }
}

class NamedPoint extends ImagePoint {
  project: Project;
  np_name: string;
  color: [number, number, number, number];
  constructor(project: Project, mnp: MappedNp) {
    super();
    this.project = project;
    this.np_name = mnp.name();
    this.x = mnp.expected_pxy[0];
    this.y = mnp.expected_pxy[1];
    const color = color_choice_as_rgb({ rgb_string: mnp.color() });
    const rgb = rgb_of_color(color);
    this.color = [rgb[0], rgb[1], rgb[2], 1];
    this.movable = false;
    }
  override draw(webgl: Webgl, webgl_canvas: WebglCanvas): void {
    webgl.set_color(this.color);
    webgl.draw(webgl_canvas.webgl_circle!);
  }
}

class MappedPoint extends NamedPoint {
  constructor(project: Project, mnp: MappedNp) {
    super(project, mnp);
    this.x = mnp.pms_x;
    this.y = mnp.pms_y;
    this.movable = true;
    }
  override draw(webgl: Webgl, webgl_canvas: WebglCanvas): void {
    webgl.set_color(this.color);
    webgl.draw(webgl_canvas.webgl_cross!);
  }
  override finished_drag(parent:StarCalibration, xy: [number, number]) {
    parent.mapped_point_moved(this, xy);
  }
}

/** The StarCalibration tab in the application
 *
 * This utilizes a WebglCanvas to draw the stars, using a ZoomedWindow whose
 * width allows for the whole of the camera image from the CIP.
 *
 * The WebGl is used with separate drawing layers for:
 *
 * 1. The camera image for the current CIP, presented using a texture-mapped
 *    square with corner coordinates at (+-1, +-1, 0) which has to be mapped to
 *    (+-scale/2 + scroll_ofs, +-scale/ar/2 + scroll_ofs, 0). The square is mapped
 *    with its upper left corner (-1,1,0) at texture coordinate (0,0), and bottom
 *    right corner (1,-1,0) at texture coordinate (1,1) (i.e. the texture is
 *    accessed with topleft at (0,0) bottom right at (1,1)
 *
 * 2. The *star field* of selected stars from the catalog, for which a cache of
 *    camera-relative star direction vectors (plus other star data) is used; a star
 *    at the centre of the camera image will have a selected star direction of (0,0,-1); +y is up.
 *
 * 3. The overlays use *image* coordinates - i.e. camera image absolute pixel
 *    coordinates, whose origin is the top left and +X is right, +Y is down
 *
 * To manage these spaces, the WebGl uses:
 *
 * * A Viewport that is the whole of the WebglCanvas (0,0 bottom left, 1,1 top right)
 *
 * * A projection matrix which maps *uniform* space onto the viewport,
 *   which applies the zoom, view port aspect ratio, and scroll offset
 *
 * * A uniform XYZ space where the image is mapped to a rectangle of width 1,
 *   height 1/image_ar centred on the origin (so the top left of the image is
 *   at (-1, +1/ar) and the bottom right is at (+1,-1/ar)
 *
 * * View and model matrices combine to map objects the uniform space.
 *
 * 1. For the image itself the WebGl square is a (+-1,+-1,0) and it uses an
 *    identity view matrix, and a model matrx that maps (X,Y,0) to (X, Y/image_ar, 0)
 *
 * 2. For the selected stars, their sensor-relative directions are cached.
 *    To convert this to *uniform* space requires mapping (X,Y,-Z) to
 *    (X/Z * FOV_scale, Y/Z * FOV_scale, 0), where FOV_scale = 1/camera_tan_hfovh.
 *    This uses an identity view matrix and a scaling model matrix
 *
 */
export class StarCalibration
  implements WebglCanvasClient, ToolsDialogClient<number>, MappedNpClient, ProjectClient
{
  application: Application;
  log: Logger;

  html_div: HtmlElement;
  zoomed_window: ZoomedWindow;
  tools_dialog: ToolsDialog<number>;
  tools_nps_div: HtmlElement;
  animate: Animate;

  star_catalog: WasmCatalog = new WasmCatalog("hipp_bright");
  wasm_star: WasmStar;
  wasm_mat4: WasmMat4f64;
  wasm_vec: WasmVec3f64;
  wasm_vec_b: WasmVec3f64;
  wasm_quat: WasmQuatf64;

  animation_rotation: number = 0;

  selected_star_indices: number[] = [];
  cached_stars: StarsWebglObj;
  img_size: [number, number] = [0, 0];

  cursor: ImagePoint;
  image_points: ImagePoint[] = [];
  sel_pt: ImagePoint | null = null;
  drag_pt: ImagePoint | null = null;

  animation_delay: number = 30;
  animation_inactivity: number = 0;

  max_magnitude: number = 7;
  image_brightness: number = 0.6;

  tab_is_selected: boolean = false;
  must_update_selected_stars: boolean = false;
  must_update_nps_div: boolean = false;

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;
    // Quiet typescript - this will be set later
    this.tools_nps_div = html_div;
    this.wasm_star = this.star_catalog.star(0)!;
    this.wasm_mat4 = WasmMat4f64.identity();
    this.wasm_vec = new WasmVec3f64(0, 0, 1);
    this.wasm_vec_b = new WasmVec3f64(0, 0, 1);
    this.wasm_quat = WasmQuatf64.unit();

    this.cursor = new Cursor();
    this.animate = new Animate(this.animation.bind(this));
    this.zoomed_window = new ZoomedWindow([10, 10]);

    this.cached_stars = new StarsWebglObj();
    application.add_tab(this, this);
    this.tools_dialog = new ToolsDialog(this, this.html_div, 20 * 1000);
    this.html_div
      .add_button(
        "open_tools",
        "open_tools",
        this.tools_dialog.open_dialog.bind(this.tools_dialog),
        { classes: "permit-interaction" },
      )
      .add_content("Tools");

    this.update_selected_stars();
  }

  tools_dialog_add_tabs(tools_dialog: ToolsDialog<number>, tabs: Tabs<number>) {
    const action_div = tabs.add_tab(
      tools_dialog.add_tab_div("tab-sc-action"),
      "Actions",
      0,
    );
    action_div
      .add_button("", "", this.find_orientation.bind(this))
      .add_content("Orientation");
    action_div
      .add_button("", "", this.new_np_and_pm.bind(this))
      .add_content("New NP+PM at cursor");
    action_div
      .add_button("", "", this.find_star_closest_to_pm.bind(this))
      .add_content("Find closest stars");

    this.tools_nps_div = tabs.add_tab(
      tools_dialog.add_tab_div("tab-sc-nps", "dialog_inner_contents"),
      "Named Points",
      1,
    );

  }

  new_np_and_pm(): void {
    console.log("Add new NP and PM at cursor");
  }

  tools_dialog_tab_selected(_t: number, _id: string): void {
    this.repopulate_nps_div();
  }

  tab_name(): string {
    return "star-calibration";
  }

  tab_text(): string {
    return "Star Calibration";
  }

  tab_deselected(): void {
    this.animate.stop();
    this.tab_is_selected = false;
  }

  /** Invoked when the main tab is selected by the user (or on load)
   *
   * Repopulate so that the UI is up-to-date
   */
  tab_selected(): void {
    this.tab_is_selected = true;
    const wh = this.application.get_resizable_content_size();
    this.tab_resize(wh[0], wh[1]);
  }

  tab_project_selected(p: Project): void {
    p.add_client(this);
  }

  /** Invoked after tab selected or set_project_updated() are invoked, in a new tick */
  tab_project_updated(): void {
    const mapped_nps = this.application.current_project().mapped_nps();
    mapped_nps.update();

    // This does too much at present
    this.update_selected_stars();
    this.update_after_pms_change()
    this.activity_occurred();
    this.application.set_redraw_required();
  }

  tab_resize(w: number, h: number) {
    this.zoomed_window.scr_resize(w, h);
  }

  /** Nothing to do for redraw - this is a WebglCanvasClient */
  tab_redraw() {
  }

  project_np_changed(_p: Project): void {
    if (this.tab_is_selected) {
      this.application.set_project_updated();
    }
  }

  project_pm_changed(_p: Project): void {
    if (this.tab_is_selected) {
      this.application.set_project_updated();
    }
  }

  project_camera_changed(_p: Project): void {
    if (this.tab_is_selected) {
      this.application.set_project_updated();
    }
  }

  project_cip_changed(_p: Project): void {
    if (this.tab_is_selected) {
      this.application.set_project_updated();
    }
  }

  /** Update the selected stars whenever tha camera has changed */
  update_selected_stars(): void {
    this.star_catalog.clear_filter();
    this.star_catalog.filter_max_magnitude(this.max_magnitude);

    this.wasm_vec.set_array(new Float64Array([0, 0, -1]));
    const wasm_cip = this.application.current_project().get_wasm_cip();
    if (wasm_cip !== null) {
      wasm_cip.camera.orientation_set_quat(this.wasm_quat);
      this.wasm_quat.set_conjugate();
    }
    this.wasm_quat.apply_set_vec3(this.wasm_vec);

    this.selected_star_indices = this.star_catalog.find_stars_around(
      this.wasm_vec,
      (40 / 180.0) * 3.1415,
      0,
      10000,
    );
  }

  find_orientation(): void {
  const project = this.application.current_project();
    const wasm_cip = project.get_wasm_cip();
    if (wasm_cip === null) { return; }

    const result = wasm_cip.stars_of_pms(
      this.star_catalog,
      0.5, 10000000);
    // The results have quaternions that map sensor space to world space
    console.log(result.has_more());
    console.log(result.num_match_sets());
    const best_match_mappings = result.get_match(0);
    if (best_match_mappings !== undefined) {
      console.log("Angle mean",best_match_mappings.angle_mean);
      console.log("Quality",best_match_mappings.quality);
      for (let i = 0; i < best_match_mappings.num_mappings; i++) {
        const m = best_match_mappings.mapping(i)!;
        const img_idx = m.img_index;
        const star_idx = m.star;
        this.star_catalog.set_star(this.wasm_star, star_idx);
        const star_id = this.wasm_star.id;
        console.log(img_idx, star_id);
      }
      best_match_mappings.set_quat(this.wasm_quat);
      // Convert the img-to-star quaternion from the match to a world-to-img quaternion required by the camera
      this.wasm_quat.set_conjugate();
      this.application.current_project().camera_set_orientation(this.wasm_quat);
    }
  }

  find_star_closest_to_pm(): void {
    const wasm_cip = this.application.current_project().get_wasm_cip();
    if (wasm_cip === null) { return; }
    for (let i = 0; i < 100000; i++) {
      if (!wasm_cip.set_pms_world_dir_vec(i, this.wasm_vec)) {
        break;
      }
      let pts = this.star_catalog.find_stars_around(this.wasm_vec, 0.1 / 180 * 3.1415, 0, 10);
      if (pts.length === 1) {
        this.star_catalog.set_star(this.wasm_star, pts[0]);
        this.wasm_star.set_vector(this.wasm_vec_b);
        const angle = Math.acos(this.wasm_vec.dot(this.wasm_vec_b)) * 180 / 3.14159;
        const np_name = wasm_cip.pms.get_name(i);
        console.log(i, np_name, pts[0], this.wasm_star.id, angle);
        if (angle < 0.1) {
          // If NP is already pointing at this star then don't change it!
          this.application.current_project().nps_set_model(np_name, true, this.wasm_vec_b, 0);
        }
      }
    }
  }

  update_after_pms_change() {
    const project = this.application.current_project();
    const mapped_nps = project.mapped_nps();

    this.image_points = [];
    this.image_points.push(this.cursor);
    for (const mnp of mapped_nps.named_points) {
      if (mnp.has_pms) {
        const op = new MappedPoint(project, mnp);
        this.image_points.push(op);
      }
      const op = new NamedPoint(project, mnp);
      this.image_points.push(op);
    }
    this.repopulate_nps_div();
  }

  /** At end of drag - update the whole project */
  cursor_move_complete(x:number, y:number) {
    this.cursor.x = x;
    this.cursor.y = y;
    this.application.current_project().set_focus(x, y);
    this.application.set_redraw_required();
  }

  /** At end of drag of point mapping */
  mapped_point_moved(mp: MappedPoint, xy: [number, number]) {
    this.application.current_project().pms_move(mp.np_name, xy);
  }

  /** Dragging of cursor - do not update all of the project */
  cursor_move(x: number, y: number): void {
    this.cursor.x = x;
    this.cursor.y = y;
    this.application.set_project_updated();
  }

  /** NP selected in tools */
  mapped_np_select_xy(x: number, y: number): void {
    this.cursor_move_complete(x, y);
  }

  /** PM add mapping selected in tools */
  mapped_np_add_mapping_for(np_name: string): void {
    this.application.current_project().pms_add(np_name, [this.cursor.x, this.cursor.y], 0);
  }

  /** PM delete mapping selected in tools */
  mapped_np_delete_mapping_for(np_name: string): void {
    this.application.current_project().pms_delete(np_name);
  }

  /** *Set* the mapping for a particualr np selected in tools
   *
   * set it to the current cursor
   */
  mapped_np_set_mapping_for(np_name: string): void {
    this.application.current_project().pms_move(np_name, [this.cursor.x, this.cursor.y]);
  }

  /** Activity occurred, so keep spinning the animations for another number of frames */
  activity_occurred() {
    this.animation_inactivity = 300;
  }

  /** Schedule the next animation, unless inactivity has sent in for long enough */
  animation_step() {
    if (this.animation_inactivity > 0) {
      this.animation_inactivity -= 1;
      this.animate.schedule(this.animation_delay);
    }
  }

  /**
   * Animation step - update rotation and mark as needing update
   */
  animation(_time: number) {
    this.animation_rotation += 10;
    if (this.animation_rotation > 360) {
      this.animation_rotation -= 360;
    }
    this.application.set_redraw_required();
  }

  /** Repopulate the tools NPS div using the current mapped_nps */
  repopulate_nps_div() {
    this.tools_nps_div.clear();

    const table = new Table({ classes: "sticky-table" });
    this.application.current_project().mapped_nps().fill_table(table, this);
    this.tools_nps_div.add_content(table.as_html());
  }

  webgl_create(webgl: Webgl, _webgl_canvas: WebglCanvas): void {
    this.cached_stars.webgl_create(webgl.webgl!);
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
      camera.set_map_world_dir_to_sensor_dir(this.wasm_vec);
      const m = this.wasm_star.magnitude;
      const t = Math.min(Math.max(Math.floor((this.wasm_star.temperature - 2300) / 7700 * 15.9), 0), 15);
      // wasm_vec is a unit world direction, with (0,0,-1) being a star at the center of the image
      pts.set([vxyz[0]!/-vxyz[2]!, vxyz[1]!/-vxyz[2]!, m, t], i * 4);
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

    const m_a = this.application.wasm_memory.float_array_of_mat4f64(
      this.wasm_mat4,
    );

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
    this.wasm_mat4.set_identity();
    m_a[0] = zoom;
    m_a[5] = zoom * view_ar;
    m_a[12] = zoom * (1 - ofs[0] * 2);
    m_a[13] = zoom * view_ar * (ofs[1] * 2 - 1);
    webgl.projection.set(m_a);

    this.wasm_mat4.set_identity();
    webgl.view.set(m_a);

    // Model for *image* should be set to map from (x, +-1, 0) to
    // (x,+-1/image_ar,z); this is so that the model Y of 1 maps to 1/image_ar
    // so that (post-model) it is a rectangle of the same shape as the image,
    // i.e. where the space is uniform (pixels are squares)
    webgl.use_program(webgl_canvas.image_program);
    this.wasm_mat4.set_identity();
    m_a[5] = 1 / image_ar;
    webgl.model.set(m_a);

    webgl.set_uniform_projection();
    webgl.set_uniform_model();
    webgl.set_uniform_view();
    webgl.set_color([this.image_brightness, this.image_brightness, this.image_brightness, 1]);
    webgl.set_texture(texture);
    webgl.draw(webgl_canvas.webgl_rectangle!);

    // Model for *stars* should be set to map from (+-1, +-1, 0) to (+-1/tanhfov, +-1/1/tanhfov)
    webgl.use_program(webgl_canvas.star_program);
    const tan_hfovh = camera.tan_hfovh;
    this.wasm_mat4.set_identity();
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
    this.wasm_mat4.set_identity();
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
    this.wasm_quat.set_unit();
    this.wasm_quat.set_mul_rotate_z((this.animation_rotation / 180) * 3.1415);

    webgl.set_uniform_projection();
    this.wasm_quat.mat4_set_rotation(this.wasm_mat4);
    this.wasm_mat4.set_scale3(0.03 / zoom);
    webgl.model.set(m_a);
    webgl.set_uniform_model();

    this.wasm_mat4.set_identity();
    for (const o of this.image_points) {
      const uxy = this.uniform_xy_of_img_xy(o.x, o.y);
      m_a[3] = uxy[0];
      m_a[7] = uxy[1];
      webgl.view.set(m_a);
      webgl.set_uniform_view();
      o.draw(webgl, webgl_canvas);
    }

    this.animation_step();
  }

  uniform_xy_of_img_xy(x: number, y: number): [number, number] {
    return [
      (2 * x) / this.img_size[0] - 1,
      (this.img_size[1] - 2 * y) / this.img_size[0],
    ];
  }

  img_xy_of_uniform_xy(x: number, y: number): [number, number] {
    return [
      ((x + 1) * this.img_size[0]) / 2,
      (this.img_size[1] - y * this.img_size[0]) / 2,
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
    for (const o of this.image_points) {
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
    this.activity_occurred();
  }

  user_release(_start_xy: [number, number], xy: [number, number]): void {
    const uni_xy = this.uniform_xy_of_scr_xy(xy[0], xy[1]);
    const img_xy = this.img_xy_of_uniform_xy(uni_xy[0], uni_xy[1]);
    this.activity_occurred();
    this.cursor_move(img_xy[0], img_xy[1]);
  }
  user_press_move(_start_xy: [number, number], _xy: [number, number]): void {}
  user_press_cancel(_start_xy: [number, number]): void {}

  user_rotate(_xy: [number, number], _angle: number): void {}

  user_pan(xy: [number, number], dxy: [number, number]): void {
    this.zoomed_window.user_pan(xy, dxy);
    this.application.set_redraw_required();
    this.activity_occurred();
  }
  user_zoom(cxy: [number, number], factor: number): void {
    this.zoomed_window.user_zoom(cxy, factor);
    this.application.set_redraw_required();
    this.activity_occurred();
  }

  drag_start(_start_xy: [number, number], _xy: [number, number]): void {
    if (this.sel_pt !== null) {
      this.drag_pt = this.sel_pt;
      this.sel_pt = null;
      this.activity_occurred();
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
      this.application.set_redraw_required();
      this.activity_occurred();
    }
  }

  drag_end(_start_xy: [number, number], xy: [number, number]): void {
    if (this.drag_pt !== null) {
      const uni_xy = this.uniform_xy_of_scr_xy(xy[0], xy[1]);
      const img_xy = this.img_xy_of_uniform_xy(uni_xy[0], uni_xy[1]);
      this.drag_pt.finished_drag(this, img_xy);
      this.activity_occurred();
    }
    this.drag_pt = null;
  }
}
