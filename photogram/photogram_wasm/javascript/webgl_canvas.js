import { WasmMat4f32 } from "../pkg/photogram_wasm.js";
import { Webgl, WebglUniform } from "./web_gl.js";
import { Webgl3DObj } from "./web_gl_3d_obj.js";
import { Mouse } from "./mouse.js";
import { ImageShader, ImageOverlayShader, StarCalibrationShader, } from "./shaders.js";
export class GridLinesObj {
    constructor(w, h) {
        this.position_buf = null;
        this.draw_horizontal = true;
        this.num_lines = 0;
        this.positions = new Float32Array([
            -w / 2,
            0,
            w / 2,
            0,
            0,
            -h / 2,
            0,
            h / 2,
        ]);
        this.args = new Float32Array([0, 0, 0, 0]);
    }
    /** offset+instance_id is applied to interval_scale in the required direction */
    set_args(num_lines, horizontal, brightness, interval_scale, offset) {
        this.num_lines = num_lines;
        if (horizontal) {
            this.draw_horizontal = true;
            this.args[0] = 0;
            this.args[1] = interval_scale;
        }
        else {
            this.draw_horizontal = false;
            this.args[1] = 0;
            this.args[0] = interval_scale;
        }
        this.args[2] = offset;
        this.args[3] = brightness;
    }
    webgl_set_uniforms(wgl) {
        wgl.set_uniform_vec4(WebglUniform.Extra0, this.args);
    }
    webgl_create(webgl) {
        this.position_buf = webgl.createBuffer();
        webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
        webgl.bufferData(webgl.ARRAY_BUFFER, this.positions, webgl.STATIC_DRAW);
    }
    webgl_draw(webgl) {
        webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
        webgl.enableVertexAttribArray(0);
        webgl.vertexAttribPointer(0, 2, webgl.FLOAT, false, 0, 0);
        const first = this.draw_horizontal ? 0 : 2;
        webgl.drawArraysInstanced(webgl.LINES, first, 2, this.num_lines);
    }
}
export class StarsWebglObj {
    constructor() {
        this.position_buf = null;
        this.num_vertices = 0;
        this.positions = new Float32Array([0, 0, 0, 0]);
    }
    webgl_set_uniforms(_wgl) { }
    webgl_create(webgl) {
        this.position_buf = webgl.createBuffer();
    }
    set_position_data(webgl, num_vertices, buffer) {
        this.num_vertices = num_vertices;
        webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
        webgl.bufferData(webgl.ARRAY_BUFFER, buffer, webgl.DYNAMIC_DRAW);
    }
    webgl_draw(webgl) {
        webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
        webgl.enableVertexAttribArray(0);
        webgl.vertexAttribPointer(0, 4, webgl.FLOAT, false, 0, 0);
        webgl.drawArrays(webgl.POINTS, 0, this.num_vertices);
    }
}
export class WebglCanvas {
    constructor(application, log, webgl_canvas) {
        this.webgl = null;
        this.image_program = 0;
        this.star_program = 0;
        this.image_grid_line_program = 0;
        this.webgl_rectangle = null;
        this.model = WasmMat4f32.identity();
        this.application = application;
        this.log = log;
        this.webgl_canvas = webgl_canvas;
        this.canvas = this.webgl_canvas.add_ele("canvas").ele;
        this.mouse = new Mouse(this, this.canvas);
        this.canvas.height = 900;
        this.current_wh = [50, 50];
        this.webgl = new Webgl(application.logger(), this.canvas);
        if (!this.start_webgl()) {
            throw "Webgl was not created correctly; aborting webgl canvas";
        }
    }
    start_webgl() {
        if (!this.webgl.start_webgl()) {
            return false;
        }
        {
            const program = this.webgl.compile_program(new ImageShader());
            if (program === null) {
                return false;
            }
            this.image_program = program;
        }
        {
            const program = this.webgl.compile_program(new StarCalibrationShader());
            if (program === null) {
                return false;
            }
            this.star_program = program;
        }
        {
            const program = this.webgl.compile_program(new ImageOverlayShader());
            if (program === null) {
                return false;
            }
            this.image_grid_line_program = program;
        }
        this.webgl_rectangle = new Webgl3DObj(4, 2, [-1, 1, 0, -1, -1, 0, 1, -1, 0, 1, 1, 0], [0, 0, 0, 1, 1, 1, 1, 0], [0, 2, 1, 2, 3, 0]);
        this.webgl.create(this.webgl_rectangle);
        this.log.info(`Created full webgl content`);
        return true;
    }
    size() {
        return this.current_wh;
    }
    create(client) {
        if (this.webgl !== null) {
            client.webgl_create(this.webgl, this);
        }
    }
    redraw(client) {
        if (this.webgl !== null) {
            const wh = this.application.get_resizable_content_size();
            if (this.current_wh != wh) {
                this.canvas.width = wh[0];
                this.canvas.height = wh[1];
                this.current_wh = wh;
            }
            client.webgl_redraw(this.webgl, this);
        }
    }
    user_press(_xy, _actions) { }
    user_press_move(_start_xy, _xy) { }
    user_press_cancel(_start_xy) { }
    user_release(_start_xy, _cxy) { }
    drag_start(_start_xy, _xy) { }
    drag_to(_start_xy, _cxy0, _cxy1) { }
    drag_end(_start_xy, _xy) { }
    user_pan(_xy, _dxy) { }
    user_zoom(_cxy, _factor) { }
    user_rotate(_xy, _angle) { }
}
