import { WasmMat4f32 } from "../pkg/photogram_wasm.js";

import { HtmlElement } from "./html.js";
import { Webgl, WebglObjKind, WebglUniform, WebglShaderSrc } from "./web_gl.js";
import { Webgl3DObj } from "./web_gl_3d_obj.js";
import { WebglFlatShader, WebglFlatObj } from "./web_gl_flat.js";
import { Logger } from "./log.js";
import { Application } from "./application.js";
import { Mouse, MouseClient, MousePressActions } from "./mouse.js";

import {
  ImageShader,
  ImageOverlayShader,
  StarCalibrationShader,
} from "./shaders.js";

export class GridLinesObj implements WebglObjKind {
  positions: Float32Array;
  position_buf: WebGLBuffer | null = null;
  args: Float32Array;
  draw_horizontal: boolean = true;
  num_lines: number = 0;

  constructor(w: number, h: number) {
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
  set_args(
    num_lines: number,
    horizontal: boolean,
    brightness: number,
    interval_scale: number,
    offset: number,
  ) {
    this.num_lines = num_lines;
    if (horizontal) {
      this.draw_horizontal = true;
      this.args[0] = 0;
      this.args[1] = interval_scale;
    } else {
      this.draw_horizontal = false;
      this.args[1] = 0;
      this.args[0] = interval_scale;
    }
    this.args[2] = offset;
    this.args[3] = brightness;
  }

  webgl_set_uniforms(wgl: Webgl) {
    wgl.set_uniform_vec4(WebglUniform.Extra0, this.args);
  }

  webgl_create(webgl: WebGLRenderingContext) {
    this.position_buf = webgl.createBuffer();
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
    webgl.bufferData(webgl.ARRAY_BUFFER, this.positions, webgl.STATIC_DRAW);
  }

  webgl_draw(webgl: WebGL2RenderingContext) {
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
    webgl.enableVertexAttribArray(0);
    webgl.vertexAttribPointer(0, 2, webgl.FLOAT, false, 0, 0);

    const first = this.draw_horizontal ? 0 : 2;
    webgl.drawArraysInstanced(webgl.LINES, first, 2, this.num_lines);
  }
}

export class StarsWebglObj implements WebglObjKind {
  positions: Float32Array;
  position_buf: WebGLBuffer | null = null;
  num_vertices: number = 0;

  constructor() {
    this.positions = new Float32Array([0, 0, 0, 0]);
  }

  webgl_set_uniforms(_wgl: Webgl) {}

  webgl_create(webgl: WebGLRenderingContext) {
    this.position_buf = webgl.createBuffer();
  }

  set_position_data(
    webgl: WebGLRenderingContext,
    num_vertices: number,
    buffer: AllowSharedBufferSource,
  ) {
    this.num_vertices = num_vertices;
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
    webgl.bufferData(webgl.ARRAY_BUFFER, buffer, webgl.DYNAMIC_DRAW);
  }

  webgl_draw(webgl: WebGLRenderingContext) {
    webgl.bindBuffer(webgl.ARRAY_BUFFER, this.position_buf);
    webgl.enableVertexAttribArray(0);
    webgl.vertexAttribPointer(0, 4, webgl.FLOAT, false, 0, 0);
    webgl.drawArrays(webgl.POINTS, 0, this.num_vertices);
  }
}

export interface WebglCanvasClient extends MouseClient {
  webgl_resize(w: number, h: number): void;
  webgl_create(webgl: Webgl, webgl_canvas: WebglCanvas): void;
  webgl_redraw(webgl: Webgl, webgl_canvas: WebglCanvas): void;
}

export class WebglCanvas {
  application: Application;
  log: Logger;
  webgl_canvas: HtmlElement;
  canvas: HTMLCanvasElement;
  mouse: Mouse;

  webgl: Webgl | null = null;
  start_webgl_failed: boolean = false;

  image_program: number = 0;
  star_program: number = 0;
  image_grid_line_program: number = 0;
  flat_program: number = 0;

  webgl_rectangle: Webgl3DObj | null = null;
  webgl_grid: GridLinesObj | null = null;
  webgl_asterisk: WebglFlatObj | null = null;

  model: WasmMat4f32 = WasmMat4f32.identity();
  current_wh: [number, number];

  constructor(
    application: Application,
    log: Logger,
    webgl_canvas: HtmlElement,
  ) {
    this.application = application;
    this.log = log;
    this.webgl_canvas = webgl_canvas;

    this.canvas = this.webgl_canvas.add_ele("canvas")!.ele as HTMLCanvasElement;

    this.mouse = new Mouse(this, this.canvas);

    this.canvas.height = 900;
    this.current_wh = [50, 50];
    this.webgl = new Webgl(application.logger(), this.canvas);

    if (!this.start_webgl()) {
      throw "Webgl was not created correctly; aborting webgl canvas";
    }
  }

  compile_program(shader: WebglShaderSrc): number {
    const program = this.webgl!.compile_program(shader);
    if (program === null) {
      this.start_webgl_failed = true;
      return 0;
    }
    return program;
  }

  start_webgl(): boolean {
    if (!this.webgl!.start_webgl()) {
      return false;
    }
    this.start_webgl_failed = false;
    this.image_program = this.compile_program(new ImageShader());
    this.star_program = this.compile_program(new StarCalibrationShader());
    this.image_grid_line_program = this.compile_program(
      new ImageOverlayShader(),
    );
    this.flat_program = this.compile_program(new WebglFlatShader());

    if (this.start_webgl_failed) {
      return false;
    }

    this.webgl_rectangle = new Webgl3DObj(
      4,
      2,
      [-1, 1, 0, -1, -1, 0, 1, -1, 0, 1, 1, 0],
      [0, 0, 0, 1, 1, 1, 1, 0],
      [0, 2, 1, 2, 3, 0],
    );
    this.webgl_grid = new GridLinesObj(2, 2);
    this.webgl_asterisk = WebglFlatObj.asterisk(1);

    this.webgl!.create(this.webgl_rectangle);
    this.webgl!.create(this.webgl_grid);
    this.webgl!.create(this.webgl_asterisk);

    this.log.info(`Created full webgl content`);
    return true;
  }

  size(): [number, number] {
    return this.current_wh;
  }

  create(client: WebglCanvasClient): void {
    if (this.webgl !== null) {
      client.webgl_create(this.webgl, this);
    }
  }

  redraw(client: WebglCanvasClient): void {
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

  user_press(_xy: [number, number], _actions: MousePressActions): void {}
  user_press_move(_start_xy: [number, number], _xy: [number, number]): void {}
  user_press_cancel(_start_xy: [number, number]): void {}
  user_release(_start_xy: [number, number], _cxy: [number, number]): void {}
  drag_start(_start_xy: [number, number], _xy: [number, number]): void {}
  drag_to(
    _start_xy: [number, number],
    _cxy0: [number, number],
    _cxy1: [number, number],
  ): void {}
  drag_end(_start_xy: [number, number], _xy: [number, number]): void {}
  user_pan(_xy: [number, number], _dxy: [number, number]): void {}
  user_zoom(_cxy: [number, number], _factor: number): void {}
  user_rotate(_xy: [number, number], _angle: number): void {}
}
