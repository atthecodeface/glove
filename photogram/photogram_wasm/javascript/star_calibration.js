import { WasmCatalog, WasmMat4f64, WasmQuatf64, WasmVec2f64, WasmVec3f64, } from "../pkg/photogram_wasm.js";
import { Animate } from "./animate.js";
import { color_choice_as_rgb, rgb_of_color } from "./color.js";
import { Table } from "./html.js";
import { ToolsDialog } from "./tools_dialog.js";
import { ZoomedWindow } from "./zoomed_window.js";
import { StarsWebglObj, } from "./webgl_canvas.js";
class ImagePoint {
    constructor() {
        this.x = 0;
        this.y = 0;
        this.movable = true;
    }
    distance_sq(x, y) {
        if (!this.movable) {
            return 1E6;
        }
        const dx = x - this.x;
        const dy = y - this.y;
        return dx * dx + dy * dy;
    }
    finished_drag(_parent, _xy) { }
    draw(_webgl, _webgl_canvas) { }
}
class Cursor extends ImagePoint {
    draw(webgl, webgl_canvas) {
        webgl.set_color([1, 1, 1, 1]);
        webgl.draw(webgl_canvas.webgl_asterisk);
    }
    finished_drag(parent, xy) {
        parent.cursor_move_complete(xy[0], xy[1]);
    }
}
/** A NamedPoint is drawn as a circle at the *expected pxy* from the MappedNp
 *
 * It has no user interaction
 */
class NamedPoint extends ImagePoint {
    constructor(project, mnp) {
        super();
        this.project = project;
        this.np_name = mnp.name();
        this.x = mnp.wasm_pms.expected_x;
        this.y = mnp.wasm_pms.expected_y;
        const color = color_choice_as_rgb({ rgb_string: mnp.color() });
        const rgb = rgb_of_color(color);
        this.color = [rgb[0], rgb[1], rgb[2], 1];
        this.movable = false;
    }
    draw(webgl, webgl_canvas) {
        webgl.set_color(this.color);
        webgl.draw(webgl_canvas.webgl_circle);
    }
}
/** A MappedPoint is drawn as a cross, and is the *mapped* location for a named point
 *
 * Dragging a mapped point moves it in the database (when the drag completes)
 */
class MappedPoint extends NamedPoint {
    constructor(project, mnp) {
        super(project, mnp);
        this.x = mnp.wasm_pms.image_x;
        this.y = mnp.wasm_pms.image_y;
        this.movable = true;
    }
    draw(webgl, webgl_canvas) {
        webgl.set_color(this.color);
        webgl.draw(webgl_canvas.webgl_cross);
    }
    finished_drag(parent, xy) {
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
 * * A *uniform* XYZ space where X, Y and Z are in mm. Note that the image has a
 * sensor size that is provided by the camera database in *mm* not just in
 * pixels, as pixels are not square. Hence the image is mapped to the sensor
 * size centred appropriately at Z=0.
 *
 * * A projection matrix which maps *uniform* space onto the viewport,
 *   which applies the zoom, view port aspect ratio, and scroll offset
 *
 * * View and model matrices combine to map objects the uniform space.
 *
 * 1. For the image itself the WebGl square is a (+-1,+-1,0) and it uses an
 * identity view matrix, and a model matrx that translates image X by the
 * (center-half pixel width), and scales it by X pixels per mm for the sensor;
 * similarly for Y
 *
 * 2. For the selected stars, their sensor-relative directions are cached.
 *    To convert this to *uniform* space requires mapping (X,Y,-Z) to
 *    (X/Z * FOV_scale, Y/Z * FOV_scale, 0), where FOV_scale = 1/camera_tan_hfovh.
 *    This uses an identity view matrix and a scaling model matrix
 *    The data X/Z and Y/Z is stored in the star vector buffer and given to the shader as the X/Y positions
 *
 */
export class StarCalibration {
    constructor(application, log, html_div) {
        this.star_catalog = new WasmCatalog("hipp_full");
        this.camera = null;
        this.animation_rotation = 0;
        this.selected_star_indices = [];
        this.img_size = [0, 0];
        this.image_points = [];
        this.sel_pt = null;
        this.drag_pt = null;
        // The number of mm wide the screen is said to be; this is the 'extent' of the zoom window
        this.mm_scr_width = 40;
        // mm_scr_width / current width of screen
        this.mm_per_scr_px = 40 / 800.0;
        this.animation_delay = 30;
        this.animation_inactivity = 0;
        this.max_magnitude = 6;
        this.max_stars = 5000;
        this.image_brightness = 0.6;
        this.tab_is_selected = false;
        this.must_update_selected_stars = false;
        this.must_update_nps_div = false;
        /** Map the selected stars through the camera orientation and calibration to sensor space x,y,1
         *
         * Invoke this if the camera changes or the max magnitude changes
         */
        this.selected_star_pts = new Float32Array(0);
        this.selected_star_pts_epoch = 0;
        this.cached_stars_epoch = 0;
        this.application = application;
        this.log = log;
        this.html_div = html_div;
        // Quiet typescript - this will be set later
        this.tools_nps_div = html_div;
        this.tools_stats_div = html_div;
        this.wasm_star = this.star_catalog.star(0);
        this.wasm_mat4 = WasmMat4f64.identity();
        this.wasm_vec2 = new WasmVec2f64(0, 0);
        this.wasm_vec = new WasmVec3f64(0, 0, 1);
        this.wasm_vec_b = new WasmVec3f64(0, 0, 1);
        this.wasm_quat = WasmQuatf64.unit();
        this.cursor = new Cursor();
        this.animate = new Animate(this.animation.bind(this));
        // Zoomed window is 40mm by 40mm
        this.zoomed_window = new ZoomedWindow([800, 800]);
        // To zoom from 10000 pixels across to 10 pixels across, need zoom range of 1000
        this.zoomed_window.min_zoom = 1;
        this.zoomed_window.max_zoom = 1000;
        this.zoomed_window.zoom_set(this.zoomed_window.min_zoom);
        this.cached_stars = new StarsWebglObj();
        application.add_tab(this, this);
        this.tools_dialog = new ToolsDialog(this, this.html_div, 20 * 1000);
        this.html_div
            .add_button("open_tools", "open_tools", this.tools_dialog.open_dialog.bind(this.tools_dialog), { classes: "permit-interaction" })
            .add_content("Tools");
        this.update_selected_stars();
    }
    tools_dialog_add_tabs(tools_dialog, tabs) {
        const action_div = tabs.add_tab(tools_dialog.add_tab_div("tab-sc-action"), "Actions", 0);
        action_div
            .add_button("", "", this.find_orientation.bind(this))
            .add_content("Orientation");
        action_div
            .add_button("", "", this.new_np_and_pm.bind(this))
            .add_content("New NP+PM at cursor");
        action_div
            .add_button("", "", this.find_star_closest_to_all_pms.bind(this))
            .add_content("Find closest stars");
        action_div
            .add_button("", "", this.reorient_using_mappings.bind(this))
            .add_content("Reorient using PMS");
        action_div.add_ele("hr");
        action_div
            .add_button("", "", () => this.application.current_project().mapped_nps().recolor_nps())
            .add_content("Recolor NPs in order");
        action_div
            .add_button("", "", () => this.application.current_project().mapped_nps().recolor_nps_by_distance())
            .add_content("Recolor NPs by distance");
        action_div.add_ele("hr");
        action_div
            .add_button("", "", () => this.refocus(2))
            .add_content("Focus +");
        action_div
            .add_button("", "", () => this.refocus(-2))
            .add_content("Focus -");
        action_div.add_ele("hr");
        action_div
            .add_button("", "", () => this.rotate_camera(0, 1.0))
            .add_content("Rot X +");
        action_div
            .add_button("", "", () => this.rotate_camera(0, -1.0))
            .add_content("Rot X -");
        action_div
            .add_button("", "", () => this.rotate_camera(1, 1.0))
            .add_content("Rot Y +");
        action_div
            .add_button("", "", () => this.rotate_camera(1, -1.0))
            .add_content("Rot Y -");
        action_div
            .add_button("", "", () => this.rotate_camera(2, 1.0))
            .add_content("Rot Z +");
        action_div
            .add_button("", "", () => this.rotate_camera(2, -1.0))
            .add_content("Rot Z -");
        action_div.add_ele("hr");
        action_div
            .add_button("", "", () => this.rotate_camera(0, 0.01))
            .add_content("Rot X +");
        action_div
            .add_button("", "", () => this.rotate_camera(0, -0.01))
            .add_content("Rot X -");
        action_div
            .add_button("", "", () => this.rotate_camera(1, 0.01))
            .add_content("Rot Y +");
        action_div
            .add_button("", "", () => this.rotate_camera(1, -0.01))
            .add_content("Rot Y -");
        action_div
            .add_button("", "", () => this.rotate_camera(2, 0.01))
            .add_content("Rot Z +");
        action_div
            .add_button("", "", () => this.rotate_camera(2, -0.01))
            .add_content("Rot Z -");
        action_div.add_ele("hr");
        action_div
            .add_button("", "", () => this.move_optical_axis_camera(-1, 0))
            .add_content("Opt axis X -");
        action_div
            .add_button("", "", () => this.move_optical_axis_camera(1, 0))
            .add_content("Opt axis X +");
        action_div
            .add_button("", "", () => this.move_optical_axis_camera(0, -1))
            .add_content("Opt axis Y -");
        action_div
            .add_button("", "", () => this.move_optical_axis_camera(0, 1))
            .add_content("Opt axis Y +");
        this.tools_stats_div = action_div.add_ele("div");
        this.tools_nps_div = tabs.add_tab(tools_dialog.add_tab_div("tab-sc-nps", "dialog_inner_contents"), "Named Points", 1);
    }
    new_np_and_pm() {
        const project = this.application.current_project();
        const np_name = project.nps_get_new_name();
        project.nps_add(np_name);
        this.application.current_project().pms_add(np_name, [this.cursor.x, this.cursor.y], 0);
        this.find_star_closest_to_pm(np_name, 0.1);
    }
    tools_dialog_tab_selected(_t, _id) {
        console.log(_t, _id);
        this.repopulate_nps_div();
    }
    tab_name() {
        return "star-calibration";
    }
    tab_text() {
        return "Star Calibration";
    }
    tab_deselected() {
        this.animate.stop();
        this.tab_is_selected = false;
    }
    /** Invoked when the main tab is selected by the user (or on load)
     *
     * Repopulate so that the UI is up-to-date
     */
    tab_selected() {
        this.tab_is_selected = true;
        const wh = this.application.get_resizable_content_size();
        this.tab_resize(wh[0], wh[1]);
        this.camera = null;
        const wasm_cip = this.application.current_project().get_wasm_cip();
        if (wasm_cip !== null) {
            this.camera = wasm_cip.camera;
        }
    }
    tab_project_selected(p) {
        p.add_client(this);
    }
    /** Invoked after tab selected or set_project_updated() are invoked, in a new tick */
    tab_project_updated() {
        const mapped_nps = this.application.current_project().mapped_nps();
        mapped_nps.update();
        // This does too much at present
        this.repopulate_nps_div();
        this.update_selected_stars();
        this.update_after_pms_change();
        this.update_map_of_selected_stars();
        this.activity_occurred();
        this.application.set_redraw_required();
    }
    tab_resize(w, h) {
        this.zoomed_window.scr_resize(w, h);
        this.mm_per_scr_px = this.mm_scr_width / w;
    }
    /** Nothing to do for redraw - this is a WebglCanvasClient */
    tab_redraw() {
    }
    project_np_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_pm_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_camera_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_cip_changed(_p) {
        if (this.tab_is_selected) {
            this.application.set_project_updated();
        }
    }
    project_mapped_nps_changed(_p) {
        if (this.tab_is_selected) {
            this.repopulate_nps_div();
        }
    }
    /** Update the selected stars whenever tha camera has changed */
    update_selected_stars() {
        this.wasm_vec.x = 0;
        this.wasm_vec.y = 0;
        this.wasm_vec.z = -1;
        const wasm_cip = this.application.current_project().get_wasm_cip();
        let hfovh = Math.atan(0.1);
        if (wasm_cip !== null) {
            wasm_cip.camera.orientation_set_quat(this.wasm_quat);
            this.wasm_quat.set_conjugate();
            hfovh = Math.atan(wasm_cip.camera.tan_hfovd);
        }
        this.wasm_quat.apply_set_vec3(this.wasm_vec);
        console.log(hfovh);
        this.star_catalog.clear_filter();
        this.star_catalog.filter_max_magnitude(this.max_magnitude);
        this.selected_star_indices = this.star_catalog.find_stars_around(this.wasm_vec, hfovh, 0, this.max_stars);
    }
    refocus(amount) {
        if (this.camera === null) {
            return;
        }
        this.camera.focus_distance = this.camera.focus_distance + amount;
        this.application.current_project().camera_changed(true);
    }
    move_optical_axis_camera(dx, dy) {
        if (this.camera === null) {
            return;
        }
        this.camera.optical_axis_offset_set_vec(this.wasm_vec2);
        this.wasm_vec2.x += dx;
        this.wasm_vec2.y += dy;
        this.camera.optical_axis_offset = this.wasm_vec2;
        console.log("Optical axis offset now", this.wasm_vec2.array);
        this.application.current_project().camera_changed(true);
    }
    rotate_camera(axis, amount) {
        if (this.camera === null) {
            return;
        }
        this.camera.orientation_set_quat(this.wasm_quat);
        switch (axis) {
            case 1: {
                this.wasm_quat.set_premul_rotate_y(amount / 180 * 3.1415);
                break;
            }
            case 2: {
                this.wasm_quat.set_premul_rotate_z(amount / 180 * 3.1415);
                break;
            }
            default: {
                this.wasm_quat.set_premul_rotate_x(amount / 180 * 3.1415);
                break;
            }
        }
        this.camera.orientation = this.wasm_quat;
        this.application.current_project().camera_changed(true);
    }
    reorient_using_mappings() {
        this.application.current_project().get_cip().orient_camera_using_model_directions(10);
        this.application.current_project().camera_changed(true);
    }
    find_orientation() {
        const project = this.application.current_project();
        const wasm_cip = project.get_wasm_cip();
        if (wasm_cip === null) {
            return;
        }
        const max_magnitude = 6;
        console.log("Clearing filter");
        this.star_catalog.clear_filter();
        console.log("Setting max magnitude to ", max_magnitude);
        this.star_catalog.filter_max_magnitude(max_magnitude);
        const result = wasm_cip.stars_of_pms(this.star_catalog, 0.5, 10000000);
        // The results have quaternions that map sensor space to world space
        if (result.has_more()) {
            console.log("Warning - not all possibilities were analyzed");
        }
        console.log("Found ", result.num_match_sets(), "sets of stars that match");
        const best_match_mappings = result.get_match(0);
        if (best_match_mappings !== undefined) {
            console.log("Angle mean of best_match", best_match_mappings.angle_mean);
            console.log("Quality of best_match", best_match_mappings.quality);
            for (let i = 0; i < best_match_mappings.num_mappings; i++) {
                const m = best_match_mappings.mapping(i);
                const img_idx = m.img_index;
                const star_idx = m.star;
                this.star_catalog.set_star(this.wasm_star, star_idx);
                const star_id = this.wasm_star.id;
                console.log("Mapping NP ", img_idx, "to star ID", star_id);
            }
            best_match_mappings.set_quat(this.wasm_quat);
            // Convert the img-to-star quaternion from the match to a world-to-img quaternion required by the camera
            this.wasm_quat.set_conjugate();
            this.application.current_project().camera_set_orientation(this.wasm_quat);
        }
    }
    find_star_closest_to_pm(np_name, max_angle) {
        const wasm_cip = this.application.current_project().get_wasm_cip();
        if (wasm_cip === null) {
            return false;
        }
        const pm = wasm_cip.pms.mapping_of_name(np_name);
        if (pm === undefined) {
            return false;
        }
        this.star_catalog.clear_filter();
        this.star_catalog.filter_max_magnitude(this.max_magnitude);
        if (!wasm_cip.set_pms_world_dir_vec(pm, this.wasm_vec)) {
            return false;
        }
        let pts = this.star_catalog.find_stars_around(this.wasm_vec, max_angle / 180 * 3.1415, 0, 10);
        if (pts.length === 1) {
            this.star_catalog.set_star(this.wasm_star, pts[0]);
            this.wasm_star.set_vector(this.wasm_vec_b);
            const angle = Math.acos(this.wasm_vec.dot(this.wasm_vec_b)) * 180 / 3.14159;
            if (angle < max_angle) {
                // If NP is already pointing at this star then don't change it!
                const wasm_np = this.application.current_project().get_wasm_nps().get_pt(np_name);
                wasm_np.model_set_vec(this.wasm_vec);
                if (this.wasm_vec.distance(this.wasm_vec_b) > 1E-10) {
                    this.application.current_project().nps_set_model(np_name, true, this.wasm_vec_b, 0);
                    console.log("Moved np to star", np_name, pts[0], this.wasm_star.id, angle);
                }
            }
        }
        return true;
    }
    find_star_closest_to_all_pms() {
        const max_angle = 0.3;
        const wasm_cip = this.application.current_project().get_wasm_cip();
        if (wasm_cip === null) {
            return;
        }
        for (let i = 0; i < 100000; i++) {
            const np_name = wasm_cip.pms.get_name(i);
            if (np_name === undefined) {
                break;
            }
            if (!this.find_star_closest_to_pm(np_name, max_angle)) {
                break;
            }
        }
    }
    update_after_pms_change() {
        const project = this.application.current_project();
        const mapped_nps = project.mapped_nps();
        this.image_points = [];
        this.image_points.push(this.cursor);
        for (const mnp of mapped_nps.named_points) {
            if (mnp.wasm_pms.has_pms) {
                const op = new MappedPoint(project, mnp);
                this.image_points.push(op);
            }
            const op = new NamedPoint(project, mnp);
            this.image_points.push(op);
        }
        this.repopulate_nps_div();
        this.tools_stats_div.clear();
        this.tools_stats_div.add_span(this.application.current_project().mapped_nps().total_sq_roll_error.toFixed(3));
        this.tools_stats_div.add_span(" ");
        this.tools_stats_div.add_span(this.application.current_project().mapped_nps().total_sq_yaw_error.toFixed(3));
    }
    /** At end of drag - update the whole project */
    cursor_move_complete(x, y) {
        this.cursor.x = x;
        this.cursor.y = y;
        this.application.current_project().set_focus(x, y);
        this.application.set_project_updated();
    }
    /** At end of drag of point mapping */
    mapped_point_moved(mp, xy) {
        this.application.current_project().pms_move(mp.np_name, xy);
    }
    /** Dragging of cursor - do not update all of the project */
    cursor_move(x, y) {
        this.cursor.x = x;
        this.cursor.y = y;
        this.application.set_project_updated();
    }
    /** NP selected in tools */
    mapped_np_select_xy(x, y) {
        this.center_on_xy(x, y);
        this.cursor_move_complete(x, y);
    }
    /** PM add mapping selected in tools */
    mapped_np_add_mapping_for(np_name) {
        this.application.current_project().pms_add(np_name, [this.cursor.x, this.cursor.y], 0);
    }
    /** PM delete mapping selected in tools */
    mapped_np_delete_mapping_for(np_name) {
        this.application.current_project().pms_delete(np_name);
    }
    /** *Set* the mapping for a particualr np selected in tools
     *
     * set it to the current cursor
     */
    mapped_np_set_mapping_for(np_name) {
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
    animation(_time) {
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
    webgl_create(webgl, _webgl_canvas) {
        this.cached_stars.webgl_create(webgl.webgl);
    }
    update_map_of_selected_stars() {
        this.selected_star_pts = new Float32Array(this.selected_star_indices.length * 4);
        this.selected_star_pts_epoch += 1;
        if (this.camera === null) {
            return;
        }
        let i = 0;
        for (const s of this.selected_star_indices) {
            this.star_catalog.set_star(this.wasm_star, s);
            this.wasm_star.set_vector(this.wasm_vec);
            this.camera.set_map_world_dir_to_sensor_dir(this.wasm_vec);
            // wasm_vec is now a sensor-relative XYZ (accounting for lens mapping)
            //
            // The sensor position (in mm) of this is lens_distance / wasm_vec.z * wasm_vec.x
            const m = this.wasm_star.magnitude;
            const t = Math.min(Math.max(Math.floor((this.wasm_star.temperature - 2300) / 7700 * 15.9), 0), 15);
            // wasm_vec is a unit world direction, with (0,0,-1) being a star at the center of the image
            this.selected_star_pts.set([this.wasm_vec.x / -this.wasm_vec.z,
                this.wasm_vec.y / -this.wasm_vec.z,
                m,
                t], i * 4);
            i += 1;
        }
    }
    /** Redraw the image
     *
     * Must map by sensor_cx/cy
     */
    redraw_image(webgl, webgl_canvas, texture) {
        if (this.camera === null) {
            return;
        }
        const m_a = this.application.wasm_memory.float_array_of_mat4f64(this.wasm_mat4);
        const mm_w = this.camera.sensor_mm_width;
        const mm_h = this.camera.sensor_mm_height;
        const px_w = this.camera.sensor_px_width;
        const px_h = this.camera.sensor_px_height;
        const px_cx = this.camera.sensor_cx;
        const px_cy = this.camera.sensor_cy;
        // Model for *image* maps +-1 in X and Y to +-mm_w/2, +-mm_h/2 (and translate if sensor is not centred).
        //
        // should be set to map from (x, +-1, 0) to
        // (x,+-1/image_ar,z); this is so that the model Y of 1 maps to 1/image_ar
        // so that (post-model) it is a rectangle of the same shape as the image,
        // i.e. where the space is uniform (pixels are squares)
        webgl.use_program(webgl_canvas.image_program);
        this.wasm_mat4.set_identity();
        webgl.view.set(m_a);
        m_a[0] = mm_w / 2;
        m_a[3] = (0.5 - px_cx / px_w) * mm_w;
        m_a[5] = mm_h / 2;
        m_a[7] = (0.5 - px_cy / px_h) * mm_h;
        webgl.model.set(m_a);
        webgl.set_uniform_projection();
        webgl.set_uniform_model();
        webgl.set_uniform_view();
        webgl.set_color([this.image_brightness, this.image_brightness, this.image_brightness, 1]);
        webgl.set_texture(texture);
        webgl.draw(webgl_canvas.webgl_rectangle);
    }
    /**
     * Redraw the grid, which is many lines using an Indexed draw element for the
     * scale of grid lines required in both directions
     */
    redraw_grid(webgl, webgl_canvas, mm_scr_width) {
        if (this.camera === null) {
            return;
        }
        const m_a = this.application.wasm_memory.float_array_of_mat4f64(this.wasm_mat4);
        const mm_w = this.camera.sensor_mm_width;
        const mm_h = this.camera.sensor_mm_height;
        const px_w = this.camera.sensor_px_width;
        const px_h = this.camera.sensor_px_height;
        const px_cx = this.camera.sensor_cx;
        const px_cy = this.camera.sensor_cy;
        // Plot the grid which is in image space, so needs the image-to-uniform model mapping
        webgl.use_program(webgl_canvas.image_grid_line_program);
        this.wasm_mat4.set_identity();
        webgl.view.set(m_a);
        m_a[0] = mm_w / 2;
        m_a[3] = (0.5 - px_cx / px_w) * mm_w;
        m_a[5] = mm_h / 2;
        m_a[7] = (0.5 - px_cy / px_h) * mm_h;
        webgl.model.set(m_a);
        webgl.set_uniform_projection();
        webgl.set_uniform_model();
        webgl.set_uniform_view();
        webgl.set_color([1.0, 1, 1.0, 1]);
        // Number of *image* pixels that are visible horizontally, so the correct grid scale can be generated
        const img_px_visible = mm_scr_width * px_w / mm_w;
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
        const x_space_of_1000px = (2 * 1000) / this.camera.sensor_px_width;
        const y_space_of_1000px = (2 * 1000) / this.camera.sensor_px_height;
        webgl_canvas.webgl_grid.set_args(2000, true, 0.45, (y_space_of_1000px / 1000) * grid_line_spacing_in_px, -1000);
        webgl.draw(webgl_canvas.webgl_grid);
        webgl_canvas.webgl_grid.set_args(2000, false, 0.45, (x_space_of_1000px / 1000) * grid_line_spacing_in_px, -1000);
        webgl.draw(webgl_canvas.webgl_grid);
        if (grid_spacing_is_pwr_of_ten) {
            grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
            grid_line_spacing_in_px *= 5;
        }
        else {
            grid_spacing_is_pwr_of_ten = !grid_spacing_is_pwr_of_ten;
            grid_line_spacing_in_px *= 2;
        }
        webgl_canvas.webgl_grid.set_args(2000, true, 0.7, (y_space_of_1000px / 1000) * grid_line_spacing_in_px, -1000);
        webgl.draw(webgl_canvas.webgl_grid);
        webgl_canvas.webgl_grid.set_args(2000, false, 0.7, (x_space_of_1000px / 1000) * grid_line_spacing_in_px, -1000);
        webgl.draw(webgl_canvas.webgl_grid);
        // Plot the axes as *bright*
        webgl_canvas.webgl_grid.set_args(1, true, 1, 0, 0);
        webgl.draw(webgl_canvas.webgl_grid);
        webgl_canvas.webgl_grid.set_args(1, false, 1, 0, 0);
        webgl.draw(webgl_canvas.webgl_grid);
    }
    /**
     * Redraw the stars whose model coordinates are sensor_vec.x / sensor_vec.z
     *
     * The model matrix must map these directions to mm; this requires scaling by the lens distance in mm
     */
    redraw_stars(webgl, webgl_canvas) {
        if (this.camera === null) {
            return;
        }
        const m_a = this.application.wasm_memory.float_array_of_mat4f64(this.wasm_mat4);
        // Model for *stars* should be set to map from (+-1, +-1, 0) to (+-sensor_mm_width/2/tanhfov, +-sensor_mm_width/2/tanhfov)
        webgl.use_program(webgl_canvas.star_program);
        // const tan_hfovh = this.camera.tan_hfovh;
        //const scale = this.camera.sensor_mm_width / 2 / tan_hfovh;
        const scale = this.camera.lens_sensor_distance;
        this.wasm_mat4.set_identity();
        m_a[0] = scale;
        m_a[5] = scale;
        webgl.model.set(m_a);
        this.wasm_mat4.set_identity();
        webgl.view.set(m_a);
        webgl.set_uniform_projection();
        webgl.set_uniform_model();
        webgl.set_uniform_view();
        webgl.set_color([0.2, 1, 0.2, 1]);
        webgl.draw(this.cached_stars);
    }
    /**
     * Redraw the grid, which is many lines using an Indexed draw element for the
     * scale of grid lines required in both directions
     */
    redraw_interesting_points(webgl, webgl_canvas, projection_zoom) {
        if (this.camera === null) {
            return;
        }
        const m_a = this.application.wasm_memory.float_array_of_mat4f64(this.wasm_mat4);
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
        this.wasm_mat4.set_scale3(0.03 / projection_zoom);
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
    }
    webgl_redraw(webgl, webgl_canvas) {
        if (this.cached_stars_epoch != this.selected_star_pts_epoch) {
            this.cached_stars.set_position_data(webgl.webgl, this.selected_star_pts.length / 4, this.selected_star_pts);
            this.cached_stars_epoch = this.selected_star_pts_epoch;
        }
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
        cip.cip_image.webgl_texture_ready();
        const texture = cip.cip_image.get_webgl_texture(webgl);
        if (texture === null) {
            return;
        }
        if (this.camera === null) {
            return;
        }
        const m_a = this.application.wasm_memory.float_array_of_mat4f64(this.wasm_mat4);
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
        // Projection maps uniform space to the viewport
        //
        // Uniform space is in mm for XYZ, with an orthogonal projection (i.e. no
        // perspective). It is a scaling (different in X and Y) and a translation.
        // This is column-major
        //
        // The number of mm wide the actual screen is
        const mm_scr_width = this.mm_per_scr_px * this.zoomed_window.get_scr_wh()[0] / this.zoomed_window.get_zoom();
        let ofs = this.zoomed_window.rel_cxy();
        const zoom = this.zoomed_window.get_zoom();
        // The projection zoom maps the mm space to view port +-1 for the full
        // width; if the screen is (orthogonally projected) 100mm wide, then the
        // projection matrix has to divide by 50, to get the resultant +-1 width of 2
        const projection_zoom = 2 / mm_scr_width;
        this.wasm_mat4.set_identity();
        m_a[0] = projection_zoom;
        m_a[5] = projection_zoom * view_ar;
        m_a[12] = zoom * (1 - ofs[0] * 2);
        m_a[13] = zoom * view_ar * (ofs[1] * 2 - 1);
        webgl.projection.set(m_a);
        // Map the ofs of the *texture* (in range 0 to 1) to the rectangle in space which is -1 to 1
        // View should be set to (+-1,+-1,z) to (zoom/ofs, zoom/ofs * (w/h), z)
        this.redraw_image(webgl, webgl_canvas, texture);
        this.redraw_stars(webgl, webgl_canvas);
        this.redraw_grid(webgl, webgl_canvas, mm_scr_width);
        this.redraw_interesting_points(webgl, webgl_canvas, projection_zoom);
        this.animation_step();
    }
    /** Calculate the uniform XY (i.e. in mm) of an image XY
     *
     * This should take into account the non-centered optical axis in the future
     */
    uniform_xy_of_img_xy(x, y) {
        if (this.camera === null) {
            return [x, y];
        }
        return [
            (x - this.camera.sensor_cx) * this.camera.sensor_mm_width / this.camera.sensor_px_width,
            (this.camera.sensor_cy - y) * this.camera.sensor_mm_height / this.camera.sensor_px_height,
        ];
    }
    /** Calculate the image XY of an XY in mm (with uniform 0,0 being the center of the sensor)
     *
     * This should take into account the non-centered optical axis in the future
     */
    img_xy_of_uniform_xy(x, y) {
        if (this.camera === null) {
            return [x, y];
        }
        return [
            this.camera.sensor_cx + x / this.camera.sensor_mm_width * this.camera.sensor_px_width,
            this.camera.sensor_cy - y / this.camera.sensor_mm_height * this.camera.sensor_px_height,
        ];
    }
    /** Map from a screen XY to a +-1 uniform XY (i.e. the uniform space in mm
     *
     * The screen XY * zoom will be in mm,as the zoomed_window is in mm
     */
    uniform_xy_of_scr_xy(x, y) {
        // screen is 0->width, 0->height coordinates, effectively nonuniform 'squished')
        // scr_rel is +-0.5,+-0.5 coordinates (units of uniform, not screen-squished)
        let scr_rel_x = x / this.zoomed_window.get_scr_wh()[0] - 0.5;
        let scr_rel_y = (y - this.zoomed_window.get_scr_wh()[1] / 2) /
            this.zoomed_window.get_scr_wh()[0];
        // Uniform is unzoom of scr_rel with the scroll offset, rescaled to +-1 instead of +-0.5
        let uni_x_unscaled = 2 *
            (scr_rel_x / this.zoomed_window.get_zoom() +
                this.zoomed_window.rel_cxy()[0]) -
            1;
        let uni_y_unscaled = 1 -
            2 *
                (scr_rel_y / this.zoomed_window.get_zoom() +
                    this.zoomed_window.rel_cxy()[1]);
        return [uni_x_unscaled * this.mm_scr_width / 2, uni_y_unscaled * this.mm_scr_width / 2];
    }
    /** Map from a +- mm uniform XY  to a screen XY */
    scr_xy_of_uniform_xy(x, y) {
        x = x * 2 / this.mm_scr_width;
        y = y * 2 / this.mm_scr_width;
        // Uniform is in +-1 space; map this to 0->1 then apply inverse window offset, then zoom
        //
        // scr_rel is +-0.5,+-0.5 coordinates (units of uniform, not screen-squished)
        let scr_rel_x = this.zoomed_window.get_zoom() *
            ((x + 1) / 2 - this.zoomed_window.rel_cxy()[0]);
        let scr_rel_y = this.zoomed_window.get_zoom() *
            ((1 - y) / 2 - this.zoomed_window.rel_cxy()[1]);
        let scr_x = (scr_rel_x + 0.5) * this.zoomed_window.get_scr_wh()[0];
        let scr_y = scr_rel_y * this.zoomed_window.get_scr_wh()[0] +
            this.zoomed_window.get_scr_wh()[1] / 2;
        return [scr_x, scr_y];
    }
    find_closest_point(img_x, img_y) {
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
    center_on_xy(x, y) {
        // We want scr_xy of the middle of the screen to be image (x,y)
        //
        // Find the uniform (+- in mm) coordinates of the image point required
        const uni_xy = this.uniform_xy_of_img_xy(x, y);
        const ux = uni_xy[0] * 2 / this.mm_scr_width;
        const uy = uni_xy[1] * 2 / this.mm_scr_width;
        let scr_rel_x = this.zoomed_window.get_zoom() *
            ((ux + 1) / 2);
        let scr_rel_y = this.zoomed_window.get_zoom() *
            ((1 - uy) / 2);
        const wh = this.zoomed_window.get_scr_wh();
        let scr_x = (scr_rel_x - 0.5) * wh[0];
        let scr_y = (scr_rel_y - 0.5) * wh[1];
        this.zoomed_window.set_zoom_scr(scr_x, scr_y);
        this.activity_occurred();
    }
    user_press(xy, actions) {
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
    user_release(_start_xy, xy) {
        const uni_xy = this.uniform_xy_of_scr_xy(xy[0], xy[1]);
        const img_xy = this.img_xy_of_uniform_xy(uni_xy[0], uni_xy[1]);
        this.activity_occurred();
        this.cursor_move_complete(img_xy[0], img_xy[1]);
    }
    user_press_move(_start_xy, _xy) { }
    user_press_cancel(_start_xy) { }
    user_rotate(_xy, _angle) { }
    user_pan(xy, dxy) {
        this.zoomed_window.user_pan(xy, dxy);
        this.application.set_redraw_required();
        this.activity_occurred();
    }
    user_zoom(cxy, factor) {
        this.zoomed_window.user_zoom(cxy, factor);
        this.application.set_redraw_required();
        this.activity_occurred();
    }
    drag_start(_start_xy, _xy) {
        if (this.sel_pt !== null) {
            this.drag_pt = this.sel_pt;
            this.sel_pt = null;
            this.activity_occurred();
        }
    }
    drag_to(_start_xy, _old_xy, xy) {
        if (this.drag_pt !== null) {
            const uni_xy = this.uniform_xy_of_scr_xy(xy[0], xy[1]);
            const img_xy = this.img_xy_of_uniform_xy(uni_xy[0], uni_xy[1]);
            this.drag_pt.x = img_xy[0];
            this.drag_pt.y = img_xy[1];
            this.application.set_redraw_required();
            this.activity_occurred();
        }
    }
    drag_end(_start_xy, xy) {
        if (this.drag_pt !== null) {
            const uni_xy = this.uniform_xy_of_scr_xy(xy[0], xy[1]);
            const img_xy = this.img_xy_of_uniform_xy(uni_xy[0], uni_xy[1]);
            this.drag_pt.finished_drag(this, img_xy);
            this.activity_occurred();
        }
        this.drag_pt = null;
    }
}
