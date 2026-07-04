export class WebglFlatShader {
    constructor() {
        this.id = "weblgl_flat";
        this.extra_uniforms = [];
        this.vertex = `#version 300 es
  uniform mat4 projection;
  uniform mat4 view;
  uniform mat4 model;

  in vec2 position;

  void main() {
    vec4 position4;
    vec4 pos;
    position4.x = position.x;
    position4.y = position.y;
    position4.z = 0.0;
    position4.w = 1.0;
    pos = projection * view * model * position4;
    gl_Position = pos;
  }
  `;
        this.fragment = `#version 300 es
  precision mediump float;
  uniform vec4 color;

  out vec4 FragColor; // must be the only output declaration; is not implicit!

  void main() {
  FragColor.r = color.r;
  FragColor.g = color.g;
  FragColor.b = color.b;
  FragColor.a = color.a;
  }
  `;
    }
}
/** A class of object that is drawn as *Lines* */
export class WebglFlatObj {
    constructor(positions, indices) {
        this.num_vertices = 0;
        this.num_indices = 0;
        this.position_buf = null;
        this.indices_buf = null;
        this.positions = positions;
        this.indices = indices;
        this.num_indices = this.indices.length;
        this.num_vertices = this.positions.length / 2;
    }
    static rectangle(width, height) {
        return new WebglFlatObj(new Float32Array([0, 0, width, 0, width, height, 0, height]), new Uint16Array([0, 1, 1, 2, 2, 3, 3, 0]));
    }
    static axis(length, ticks) {
        let pts = [];
        let lines = [];
        pts.push(0, 0, length, 0);
        lines.push(0, 1);
        let pt_index = 2;
        for (const ab of ticks) {
            const ticks_per_unit = ab[0];
            const dy = ab[1];
            for (let x = 0; x <= length * ticks_per_unit; x++) {
                const px = x / (ticks_per_unit + 0.0);
                pts.push(px, 0, px, dy);
                lines.push(pt_index, pt_index + 1);
                pt_index += 2;
            }
        }
        return new WebglFlatObj(new Float32Array(pts), new Uint16Array(lines));
    }
    static circle(radius, steps) {
        let pts = [];
        let lines = [];
        let da = Math.PI / (steps * 0.5);
        for (let i = 0; i < steps; i++) {
            let x = radius * Math.cos(i * da);
            let y = radius * Math.sin(i * da);
            pts.push(x, y);
            lines.push(i, (i + 1) % steps);
        }
        return new WebglFlatObj(new Float32Array(pts), new Uint16Array(lines));
    }
    set_point(index, x, y) {
        this.positions[index * 2] = x;
        this.positions[index * 2 + 1] = y;
    }
    set_line(index, start, end) {
        this.indices[2 * index + 0] = start;
        this.indices[2 * index + 1] = end;
    }
    webgl_create(webgl) {
        this.position_buf = webgl.createBuffer();
        webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
        webgl.bufferData(webgl.ARRAY_BUFFER, this.positions.buffer, webgl.STATIC_DRAW);
        this.indices_buf = webgl.createBuffer();
        webgl.bindBuffer(webgl.ELEMENT_ARRAY_BUFFER, this.indices_buf);
        webgl.bufferData(webgl.ELEMENT_ARRAY_BUFFER, this.indices.buffer, webgl.STATIC_DRAW);
    }
    webgl_set_uniforms(_wgl) { }
    webgl_draw(webgl) {
        webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
        webgl.enableVertexAttribArray(0);
        webgl.vertexAttribPointer(0, 2, webgl.FLOAT, false, 0, 0);
        webgl.bindBuffer(webgl.ELEMENT_ARRAY_BUFFER, this.indices_buf);
        webgl.drawElements(webgl.LINES, this.num_indices, webgl.UNSIGNED_SHORT, 0);
    }
}
