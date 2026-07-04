import { Logger } from "./log.js";
export var WebglUniform;
(function (WebglUniform) {
    WebglUniform[WebglUniform["Projection"] = 0] = "Projection";
    WebglUniform[WebglUniform["View"] = 1] = "View";
    WebglUniform[WebglUniform["Model"] = 2] = "Model";
    WebglUniform[WebglUniform["Color"] = 3] = "Color";
    WebglUniform[WebglUniform["Sampler"] = 4] = "Sampler";
    WebglUniform[WebglUniform["Extra0"] = 5] = "Extra0";
    WebglUniform[WebglUniform["Extra1"] = 6] = "Extra1";
    WebglUniform[WebglUniform["Extra2"] = 7] = "Extra2";
    WebglUniform[WebglUniform["Extra3"] = 8] = "Extra3";
    WebglUniform[WebglUniform["Extra4"] = 9] = "Extra4";
})(WebglUniform || (WebglUniform = {}));
export class WebglProgram {
    constructor(shader, webgl, program) {
        this.matrix = new Float32Array(16);
        this.owner = shader.id;
        this.program = program;
        // At least up to 'Extra' uniforms...
        this.uniforms = [null, null, null, null, null];
        const u_projection = webgl.getUniformLocation(program, "projection");
        const u_view = webgl.getUniformLocation(program, "view");
        const u_model = webgl.getUniformLocation(program, "model");
        const u_color = webgl.getUniformLocation(program, "color");
        const u_sampler = webgl.getUniformLocation(program, "uSampler");
        this.uniforms[WebglUniform.Projection] = u_projection;
        this.uniforms[WebglUniform.View] = u_view;
        this.uniforms[WebglUniform.Model] = u_model;
        this.uniforms[WebglUniform.Sampler] = u_sampler;
        this.uniforms[WebglUniform.Color] = u_color;
        this.webgl = webgl;
        for (const e of shader.extra_uniforms) {
            const u = webgl.getUniformLocation(program, e);
            this.uniforms.push(u);
        }
    }
    set_uniform_ivec4(uniform, matrix) {
        const u = this.uniforms[uniform];
        if (u !== null) {
            this.webgl.uniform4iv(u, matrix);
        }
    }
    set_uniform_mat4(uniform, matrix, transpose = false) {
        const u = this.uniforms[uniform];
        if (u !== null) {
            if (transpose) {
                for (let i = 0; i < 4; i++) {
                    for (let j = 0; j < 4; j++) {
                        this.matrix[i * 4 + j] = matrix[j * 4 + i];
                    }
                }
            }
            else {
                this.matrix.set(matrix);
            }
            this.webgl.uniformMatrix4fv(u, false, this.matrix);
        }
    }
    set_uniform_float(uniform, value) {
        const u = this.uniforms[uniform];
        if (u !== null) {
            this.webgl.uniform1f(u, value);
        }
    }
    set_uniform_vec2(uniform, value) {
        const u = this.uniforms[uniform];
        if (u !== null) {
            this.webgl.uniform2fv(u, value);
        }
    }
    set_uniform_vec3(uniform, value) {
        const u = this.uniforms[uniform];
        if (u !== null) {
            this.webgl.uniform3fv(u, value);
        }
    }
    set_uniform_vec4(uniform, value) {
        const u = this.uniforms[uniform];
        if (u !== null) {
            this.webgl.uniform4fv(u, value);
        }
    }
    set_texture(uniform, texture) {
        const u = this.uniforms[uniform];
        if (u !== null) {
            this.webgl.activeTexture(this.webgl.TEXTURE0);
            this.webgl.bindTexture(this.webgl.TEXTURE_2D, texture.texture);
            this.webgl.uniform1i(u, 0);
        }
    }
}
export class WebglTexture {
    constructor(webgl, image = null) {
        this.image = null;
        this.image_load_completed = false;
        this.texture_bound = false;
        this.webgl = webgl;
        const w = webgl.webgl;
        if (w === null) {
            throw "Webgl not constructed correctly for a WebglTexture";
        }
        this.texture = w.createTexture();
        this.image = image;
        if (this.image !== null) {
            this.image.addEventListener("load", this.image_loaded.bind(this));
        }
        w.bindTexture(w.TEXTURE_2D, this.texture);
        w.texImage2D(w.TEXTURE_2D, 0, w.RGBA, 1, 1, 0, w.RGBA, w.UNSIGNED_BYTE, new Uint8Array([0, 0, 255, 255]));
        w.texParameteri(w.TEXTURE_2D, w.TEXTURE_MIN_FILTER, w.NEAREST);
        w.texParameteri(w.TEXTURE_2D, w.TEXTURE_MAG_FILTER, w.LINEAR);
        w.texParameteri(w.TEXTURE_2D, w.TEXTURE_WRAP_S, w.CLAMP_TO_EDGE);
        w.texParameteri(w.TEXTURE_2D, w.TEXTURE_WRAP_T, w.CLAMP_TO_EDGE);
    }
    image_loaded(_event) {
        this.image_load_completed = true;
        this.bind_to_image(this.image);
    }
    bind_to_image(source) {
        const w = this.webgl.webgl;
        w.bindTexture(w.TEXTURE_2D, this.texture);
        w.texImage2D(w.TEXTURE_2D, 0, w.RGBA, w.RGBA, w.UNSIGNED_BYTE, source);
        this.texture_bound = true;
    }
}
export class Webgl {
    constructor(log, canvas) {
        this.programs = [];
        this.current_program = null;
        this.webgl = null;
        this.identity = new Float32Array([
            1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1,
        ]);
        this.logger = new Logger(log, "webgl");
        this.canvas = canvas;
        this.projection = new Float32Array([
            1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1,
        ]);
        this.view = new Float32Array([
            1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1,
        ]);
        this.model = new Float32Array([
            1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1,
        ]);
    }
    /** Start WebGL - invoke this when the window has loaded
     *
     */
    start_webgl() {
        var gl;
        try {
            gl = this.canvas.getContext("webgl2");
        }
        catch (x) {
            this.logger.error("webgl", `Failed to get WebGL context`);
            return false;
        }
        this.webgl = gl;
        return gl !== null;
    }
    /** Load a shader
     *
     */
    load_shader(vertex, src) {
        const webgl = this.webgl;
        let kind = webgl.VERTEX_SHADER;
        if (!vertex) {
            kind = webgl.FRAGMENT_SHADER;
        }
        const shader = webgl.createShader(kind);
        if (shader == null) {
            this.logger.error("Failed to create shader");
            return null;
        }
        webgl.shaderSource(shader, src);
        webgl.compileShader(shader);
        if (!webgl.getShaderParameter(shader, webgl.COMPILE_STATUS)) {
            this.logger.error(`Failed to compile shader ${webgl.getShaderInfoLog(shader)}`);
            return null;
        }
        return shader;
    }
    compile_program(shader) {
        if (this.webgl === null) {
            return null;
        }
        const webgl = this.webgl;
        const program = webgl.createProgram();
        const owner = shader.id;
        const vs = this.load_shader(true, shader.vertex);
        if (vs == null) {
            this.logger.error("webgl", `Failed to compile vertex shader for ${owner}`);
            return null;
        }
        webgl.attachShader(program, vs);
        webgl.deleteShader(vs);
        const fs = this.load_shader(false, shader.fragment);
        if (fs == null) {
            this.logger.error("webgl", `Failed to compile fragment shader for ${owner}`);
            return null;
        }
        webgl.attachShader(program, fs);
        webgl.deleteShader(fs);
        webgl.linkProgram(program);
        webgl.useProgram(program);
        if (!webgl.getProgramParameter(program, webgl.LINK_STATUS)) {
            this.logger.error("webgl", `Failed to load shaders for ${owner} ${webgl.getProgramInfoLog(program)}`);
            return null;
        }
        const n = this.programs.length;
        this.programs.push(new WebglProgram(shader, webgl, program));
        return n;
    }
    set_viewport(bbox = [0, 0, 0, 0]) {
        if (bbox[3] === 0) {
            const w = this.canvas.width;
            const h = this.canvas.height;
            this.webgl.viewport(0, 0, w, h);
        }
        else {
            this.webgl.viewport(bbox[0], bbox[1], bbox[2], bbox[3]);
        }
    }
    clear_buffer(options = {}) {
        if (this.webgl === null) {
            return;
        }
        const depth_test = options.depth_test !== undefined ? options.depth_test : true;
        if (depth_test) {
            this.webgl.enable(this.webgl.DEPTH_TEST); // Enable depth testing
        }
        this.webgl.depthFunc(this.webgl.LEQUAL); // Near things obscure far things
        this.webgl.clear(this.webgl.COLOR_BUFFER_BIT | this.webgl.DEPTH_BUFFER_BIT);
    }
    clear_depth_buffer() {
        if (this.webgl === null) {
            return;
        }
        this.webgl.clear(this.webgl.DEPTH_BUFFER_BIT);
    }
    use_program(p) {
        if (this.webgl === null) {
            return;
        }
        const program = this.programs[p];
        if (program === undefined) {
            return;
        }
        this.current_program = program;
        this.webgl.useProgram(this.current_program.program);
    }
    create(obj) {
        if (this.webgl !== null) {
            obj.webgl_create(this.webgl);
        }
    }
    set_view_camera(cxyz, direction, up) {
        this.view.fill(0);
        const ddl_sq = direction[0] * direction[0] +
            direction[1] * direction[1] +
            direction[2] * direction[2];
        const upl_sq = up[0] * up[0] + up[1] * up[1] + up[2] * up[2];
        const upl = Math.sqrt(upl_sq);
        const ddl = Math.sqrt(ddl_sq);
        const upl_ddl = upl * ddl;
        const normal = [
            (direction[1] * up[2] - direction[2] * up[1]) / upl_ddl,
            (direction[2] * up[0] - direction[0] * up[2]) / upl_ddl,
            (direction[0] * up[1] - direction[1] * up[0]) / upl_ddl,
        ];
        this.view[0] = normal[0] / ddl;
        this.view[1] = normal[1] / ddl;
        this.view[2] = normal[2] / ddl;
        this.view[4] = up[0] / ddl;
        this.view[5] = up[1] / ddl;
        this.view[6] = up[2] / ddl;
        this.view[8] = direction[0] / ddl;
        this.view[9] = direction[1] / ddl;
        this.view[10] = direction[2] / ddl;
        this.view[11] = -cxyz[0];
        this.view[12] = -cxyz[1];
        this.view[13] = -cxyz[2];
        this.view[15] = 1;
    }
    /**
     * Set the projection matrix such that the camera is at (0,0,0) looking in the
     * direction of +Z, such that the Z range 'near' to 'far' (far>near>0) is the
     * Z clip range, and that every coordinate is scaled by 1/z
     *
     * Sets it *COLUMN MAJOR* so need to transpose
     *
     * near is the closest value of Z (normally +ve) that should be visible
     *
     * far is the furthest value of Z (normally +ve) that should be visible
     *
     * aspect_ratio is the width / height of the view
     *
     * tan_hfovh is the tan of half of the horizontal field of view
     *
     * The projection is effectively mapping (x,y,z) to (X,Y,Z,W), with X/W,Y/W,Z/W in the +-1 cube at the origin; the 'Z/W' value is the depth in the depth buffer.
     *
     * @param tan_hfovh
     * @param aspect_ratio  width of window divide by height of window
     * @param near Z value that maps to
     * @param far
     */
    set_projection_perspective(tan_hfovh, aspect_ratio, near, // closest 'z' to use
    far) {
        // Post-scale Z = z * (near + far) / (far - near) - (near * far * 2) / (far - near) =  (z * near + z * far - near * far * 2) / (far - near)
        // Post-scale W = z
        // Post perspective Z_out = (near + far - near * far * 2 / z) / (far - near);
        //   if z in is near, Z_out = (near + far - far * 2) / (far - near);
        //                          = -1;
        //   if z in is far, Z_out = (near + far - near * 2) / (far - near);
        //                          = (far - near) / (far - near);
        //                          = 1;
        const f = 1.0 / tan_hfovh;
        this.projection.fill(0);
        this.projection[0] = f; // Note resultant X out is f * x / w = f.x/z
        this.projection[5] = f * aspect_ratio; // Note resultant Y out is f * y * ar / w = f.x/z
        this.projection[10] = (1.0 * (near + far)) / (far - near); // Scale z by this
        this.projection[11] = 1; // Scale of 'z' to get 'w', which is used to divide x, y, z
        this.projection[14] = (2.0 * (near * far)) / (near - far); // Add this to scaled z for Z
    }
    /**
     * Set the projection matrix such that the camera is at (0,0,0) looking in the
     * direction of +Z, such that the Z range 'near' to 'far' (far>near>0) is the
     * Z clip range, and that every coordinate is not scaled by distance
     *
     *
     * Sets it *COLUMN MAJOR* so need to transpose
     *
     * near is the closest value of Z (normally +ve) that should be visible
     *
     * far is the furthest value of Z (normally +ve) that should be visible
     *
     * aspect_ratio is the width / height of the view
     *
     * tan_hfovh is the tan of half of the horizontal field of view
     *
     * The projection is effectively mapping (x,y,z) to (X,Y,Z,1), with X,Y,Z in the +-1 cube at the origin; the 'Z' value is the depth in the depth buffer.
     *
     * @param tan_hfovh
     * @param aspect_ratio  width of window divide by height of window
     * @param near z value that maps to -1
     * @param far z value that maps to +1
     */
    set_projection_no_perspective(tan_hfovh, aspect_ratio, near, // closest 'z' to use
    far) {
        // Post-scale Z = z * 2 / (far - near) - (near + far) / (far - near) = (2z - near - far) / (far - near)
        // Post-scale W = 1
        // Post perspective Z_out = (2z - near - far) / (far - near) / 1
        //   if z in is near, Z_out = (2*near - near - far) / (far - near);
        //                          = -1;
        //   if z in is far, Z_out = (2*far - near - far) / (far - near);
        //                          = (far - near) / (far - near);
        //                          = 1;
        const f = 1.0 / tan_hfovh;
        this.projection.fill(0);
        this.projection[0] = f; // Note resultant X out is f * x / w = f.x/z
        this.projection[5] = f * aspect_ratio; // Note resultant Y out is f * y * ar / w = f.x/z
        this.projection[10] = 2.0 / (far - near); // Scale z by this
        this.projection[15] = 1; // W is 1 for no perspective
        this.projection[14] = (near + far) / (near - far); // Add this to scaled z for Z
    }
    set_uniform_float(uniform, value) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_float(uniform, value);
    }
    set_uniform_vec2(uniform, value) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_vec2(uniform, value);
    }
    set_uniform_vec3(uniform, value) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_vec3(uniform, value);
    }
    set_uniform_vec4(uniform, value) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_vec4(uniform, value);
    }
    set_uniform_mat4(uniform, matrix, transpose = false) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_mat4(uniform, matrix, transpose);
    }
    set_uniform_ivec4(uniform, matrix) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_ivec4(uniform, matrix);
    }
    set_uniform_projection() {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_mat4(WebglUniform.Projection, this.projection);
    }
    set_uniform_model() {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_mat4(WebglUniform.Model, this.model, true);
    }
    set_uniform_view() {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_mat4(WebglUniform.View, this.view, true);
    }
    set_color(color) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_uniform_vec4(WebglUniform.Color, color);
    }
    set_texture(texture) {
        if (this.current_program === null) {
            return;
        }
        this.current_program.set_texture(WebglUniform.Sampler, texture);
    }
    draw(obj) {
        if (this.webgl !== null) {
            obj.webgl_set_uniforms(this);
            obj.webgl_draw(this.webgl);
        }
    }
}
