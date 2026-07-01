import { WebglShaderSrc, WebglObjKind, Webgl } from "./web_gl.js";

export class Webgl3DObjSimpleShader implements WebglShaderSrc {
  id: string = "webgl_3d_simple";
  extra_uniforms: string[] = [];

  vertex: string = `#version 300 es
  uniform mat4 projection;
  uniform mat4 view;
  uniform mat4 model;

  in vec4 position;
  in vec2 tex_coord;

  out vec2 vTextureCoord;
  void main() {
    vec4 pos;
    pos = projection * view * model * position;
    gl_Position = pos;
    vTextureCoord = tex_coord;
  }
`;

  fragment: string = `#version 300 es
  precision mediump float;
  uniform vec4 color;
  uniform sampler2D uSampler;

  in vec2 vTextureCoord;

  out vec4 FragColor; // must be the only output declaration; is not implicit!

  void main() {
    FragColor = texture(uSampler, vTextureCoord);
 }
  `;
}

export class Webgl3DObj implements WebglObjKind {
  positions: Float32Array;
  tex_coords: Float32Array;
  indices: Uint16Array;
  num_vertices: number = 0;
  num_indices: number = 0;
  position_buf: WebGLBuffer | null = null;
  tex_coord_buf: WebGLBuffer | null = null;
  indices_buf: WebGLBuffer | null = null;

  constructor(max_vertices: number, max_indices: number) {
    this.positions = new Float32Array(3 * max_vertices);
    this.tex_coords = new Float32Array(2 * max_vertices);
    this.indices = new Uint16Array(3 * max_indices);
  }
  static tetrahedron(d: number): Webgl3DObj {
    //
    // The tetrahedron is specifed to have corners at
    //
    //   d,d,d; -d,d,d; d,-d,d; d,d,-d
    //
    // The triangles are then all the counter-clockwise triangles, but specifically in the texture they are
    //
    // 1------0------1
    // |  0  /|  2  /|
    // |    / |    / |
    // |   /  |   /  |
    // |  /   |  /   |
    // | /    | /    |
    // |/  1  |/  3  |
    // 2------3------2

    const tetra = new Webgl3DObj(0, 0);
    tetra.positions = new Float32Array([
      d,
      d,
      d, // 0
      d,
      -d,
      -d, // 1
      -d,
      d,
      -d, // 2
      -d,
      -d,
      d, // 3

      d,
      d,
      d, // 0
      d,
      -d,
      -d, // 1
      -d,
      d,
      -d, // 2
      -d,
      -d,
      d, // 3
    ]);
    tetra.tex_coords = new Float32Array([
      0.5,
      0, //
      0,
      0, //
      0,
      1, //
      0.5,
      1, //

      0.5,
      0, //
      1.0,
      0, //
      1.0,
      1, //
      0.5,
      1, //
    ]);
    tetra.indices = new Uint16Array([
      0,
      1,
      2,
      0,
      2,
      3, //
      4,
      5,
      7,
      5,
      7,
      6,
    ]);
    tetra.num_vertices = 8;
    tetra.num_indices = 12;
    return tetra;
  }

  static octahedron(d: number, hpx: number): Webgl3DObj {
    //
    // The octahedron is specifed to have corners at
    //
    //   d,0,0, -d,0,0, 0,d,0, 0,-d,0, 0,0,d, 0,0,-d
    //
    // The triangles are then 0,2,4 & 4,2,1; 1,3,4 & 4,3,0; 0,5,2 & 2,5,1; 1,5,3 & 3,5,0
    //
    // +------+------+
    // |0   4/|1   4/|
    // |    / |    / |
    // |   /  |   /  |
    // |  /   |  /   |
    // |2/    |3/    |
    // |/    1|/    0|
    // +------+------+
    // |0   5/|1   3/|
    // |    / |    / |
    // |   /  |   /  |
    // |  /   |  /   |
    // |2/    |5/    |
    // |/    1|/    0|
    // +------+------+
    const octa = new Webgl3DObj(0, 0);
    octa.positions = new Float32Array([
      d,
      0,
      0, // 0
      0,
      d,
      0, // 2
      0,
      0,
      d, // 4
      -d,
      0,
      0, // 1
      -d,
      0,
      0, // 1
      0,
      -d,
      0, // 3
      0,
      0,
      d, // 4
      d,
      0,
      0, // 0

      d,
      0,
      0, // 0
      0,
      0,
      -d, // 5
      0,
      d,
      0, // 2
      -d,
      0,
      0, // 1
      -d,
      0,
      0, // 1
      0,
      0,
      -d, // 5
      0,
      -d,
      0, // 3
      d,
      0,
      0, // 0
    ]);
    octa.tex_coords = new Float32Array([
      hpx,
      hpx,
      hpx,
      0.5 - hpx,
      0.5 - hpx,
      hpx,
      0.5 - hpx,
      0.5 - hpx, // first quadrant
      0.5 + hpx,
      hpx,
      0.5 + hpx,
      0.5 + hpx,
      1.0 - hpx,
      hpx,
      1.0 - hpx,
      0.5 + hpx, // second quadrant
      hpx,
      0.5 + hpx,
      hpx,
      1.0 - hpx,
      0.5 - hpx,
      0.5 + hpx,
      0.5 - hpx,
      1.0 - hpx, // third quadrant
      0.5 + hpx,
      0.5 + hpx,
      0.5 + hpx,
      1.0 - hpx,
      1.0 - hpx,
      0.5 + hpx,
      1.0 - hpx,
      1.0 - hpx, // fourth quadrant
    ]);
    octa.indices = new Uint16Array([
      0, 1, 2, 2, 1, 3,

      4, 5, 6, 6, 5, 7,

      8, 9, 10, 10, 9, 11,

      12, 13, 14, 14, 13, 15,
    ]);
    octa.num_vertices = 16;
    octa.num_indices = 24;
    return octa;
  }
  static cuboid(dx: number, dy: number, dz: number): Webgl3DObj {
    //
    // The cubemap texture is defined by
    //
    //    0 => (0, 1, Quat::look_at(&[-1., 0., 0.], &[0., 1., 0.])),
    //    1 => (1, 1, Quat::look_at(&[0., 0., -1.], &[0., 1., 0.])),
    //    2 => (2, 1, Quat::look_at(&[1., 0., 0.], &[0., 1., 0.])),
    //    3 => (3, 1, Quat::look_at(&[0., 0., 1.], &[0., 1., 0.])),
    //    4 => (1, 0, Quat::look_at(&[0., 1., 0.], &[0., 0., 1.])),
    //    5 => (1, 2, Quat::look_at(&[0., -1., 0.], &[0., 0., -1.])),
    //
    // a---bH--cG--d---e      H-----G    Z            0 : A E H D
    // |   | 4 |   |   |     /     /|    |   Y        1 : A B F E
    // fH--gE--hF--iG--jH   D-----C |    | /          2 : B C G F
    // | 0 | 1 | 2 | 3 |    |     | F    |/           3 : C G H D
    // kD--lA--mB--nC--oD   |     |/     +-----X      4 : E F G H
    // |   | 5 |   |   |    A-----B                   5 : A D C B
    // p---qD--rC--s---t
    //
    // Triangles ABC, ACD, BFG, BGC, FEH, FHG, EAD, EDH, DCG, DGH, ABF, AFE
    // as        lmr, lrq, mhi, min, hgb, hbc, glk, gkf, oni, oij, lmh, lhg
    //
    // Points/texcoords are stored lmnokhgijfbcqr
    const t0_4 = 0.0;
    const t1_4 = 0.25;
    const t2_4 = 0.5;
    const t3_4 = 0.75;
    const t4_4 = 1.0;

    const t0_3 = 0.0;
    const t1_3 = 0.33333333;
    const t2_3 = 0.66666666;
    const t3_3 = 1.0;

    const cube = new Webgl3DObj(0, 0);
    cube.positions = new Float32Array([
      -dx,
      -dy,
      -dz,
      dx,
      -dy,
      -dz,
      dx,
      -dy,
      dz,
      -dx,
      -dy,
      dz,
      -dx,
      -dy,
      dz, // near face + D repeated (l,m,n,o,k)

      -dx,
      dy,
      -dz,
      dx,
      dy,
      -dz,
      dx,
      dy,
      dz,
      -dx,
      dy,
      dz,
      -dx,
      dy,
      dz, // far face + H repeated (g,h,i,j,f)

      -dx,
      dy,
      dz,
      dx,
      dy,
      dz, // b, c

      -dx,
      -dy,
      dz,
      dx,
      -dy,
      dz, // q, r
    ]);
    cube.tex_coords = new Float32Array([
      t1_4,
      t2_3,
      t2_4,
      t2_3,
      t3_4,
      t2_3,
      t4_4,
      t2_3,
      t0_4,
      t2_3, // l, m, n, o, k
      t1_4,
      t1_3,
      t2_4,
      t1_3,
      t3_4,
      t1_3,
      t4_4,
      t1_3,
      t0_4,
      t1_3, // g,h,i,j,f
      t1_4,
      t0_3,
      t2_4,
      t0_3, // b, c
      t1_4,
      t3_3,
      t2_4,
      t3_3, // q, r
    ]);
    // as        lmr, lrq, mhi, min, hgb, hbc, glk, gkf, oni, oij, lmh, lhg
    //
    // Points/texcoords are stored lmnok hgijf bcqr
    cube.indices = new Uint16Array([
      0, 1, 13, 0, 13, 12,

      1, 6, 7, 1, 7, 2,

      3, 2, 7, 3, 7, 8,

      0, 1, 6, 0, 5, 6,

      5, 0, 4, 5, 4, 9,

      6, 5, 10, 6, 10, 11,
    ]);
    cube.num_vertices = 14;
    cube.num_indices = 36;
    return cube;
  }

  add_vertex(position: Float32Array, texcoord: Float32Array) {
    this.positions.set(position, this.num_vertices * 3);
    this.tex_coords.set(texcoord, this.num_vertices * 2);
    this.num_vertices += 1;
  }

  add_face(indices: number[]) {
    this.indices.set(indices, this.num_indices);
    this.num_indices += indices.length;
  }

  webgl_set_uniforms(_wgl: Webgl) {}

  webgl_create(webgl: WebGLRenderingContext) {
    this.position_buf = webgl.createBuffer();
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
    webgl.bufferData(
      webgl.ARRAY_BUFFER,
      this.positions.buffer,
      webgl.STATIC_DRAW,
    );

    this.tex_coord_buf = webgl.createBuffer();
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.tex_coord_buf);
    webgl.bufferData(
      webgl.ARRAY_BUFFER,
      this.tex_coords.buffer,
      webgl.STATIC_DRAW,
    );

    this.indices_buf = webgl.createBuffer();
    webgl.bindBuffer(webgl.ELEMENT_ARRAY_BUFFER, this.indices_buf);
    webgl.bufferData(
      webgl.ELEMENT_ARRAY_BUFFER,
      this.indices.buffer,
      webgl.STATIC_DRAW,
    );
  }

  webgl_draw(webgl: WebGLRenderingContext) {
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
    webgl.enableVertexAttribArray(0);
    webgl.vertexAttribPointer(0, 3, webgl.FLOAT, false, 0, 0);
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.tex_coord_buf);
    webgl.enableVertexAttribArray(1);
    webgl.vertexAttribPointer(1, 2, webgl.FLOAT, false, 0, 0);

    webgl.bindBuffer(webgl.ELEMENT_ARRAY_BUFFER, this.indices_buf);
    webgl.drawElements(
      webgl.TRIANGLES,
      this.num_indices,
      webgl.UNSIGNED_SHORT,
      0,
    );
  }
}
