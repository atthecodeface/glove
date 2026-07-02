import { WasmMat4f32 } from "../pkg/photogram_wasm.js";

import { HtmlElement } from "./html.js";
import { Webgl } from "./web_gl.js";
import { Webgl3DObj } from "./web_gl_3d_obj.js";
//import {
//   WebglCubicBezierShader,
//  WebglCubicBezierObj,
// } from "./web_gl_bezier.js";
import { WebglFlatShader, WebglFlatObj } from "./web_gl_flat.js";
// import { WasmMemory } from "./wasm_memory.js";
import { Logger } from "./log.js";
// import { ViewProperties } from "./view_properties.js";
import { Application } from "./application.js";
import { Mouse, MouseClient, MousePressActions } from "./mouse.js";

import {
  EarthShader,
  StarShader,
  SphereShader,
  StarMapShader,
  StarShaderProjectedOntoNear,
} from "./shaders.js";

export interface WebglCanvasClient extends MouseClient {
  redraw: (webgl: Webgl, webgl_canvas: WebglCanvas) => void;
}

export class WebglCanvas {
  application: Application;
  log: Logger;
  webgl_canvas: HtmlElement;
  canvas: HTMLCanvasElement;
  mouse: Mouse;

  webgl: Webgl | null = null;

  earth_program: number = 0;
  sphere_program: number = 0;
  flat_program: number = 0;
  bezier_program: number = 0;
  star_program: number = 0;
  star_projected_onto_near_program: number = 0;
  star_map_program: number = 0;

  webgl_icosphere: Webgl3DObj | null = null;
  webgl_axis: WebglFlatObj | null = null;
  webgl_triangle: Webgl3DObj | null = null;
  webgl_circle: WebglFlatObj | null = null;

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

  start_webgl(): boolean {
    if (!this.webgl!.start_webgl()) {
      return false;
    }

    {
      const program = this.webgl!.compile_program(new EarthShader());
      if (program === null) {
        return false;
      }
      this.earth_program = program;
    }

    {
      const program = this.webgl!.compile_program(new SphereShader());
      if (program === null) {
        return false;
      } else {
        this.sphere_program = program;
      }
    }

    {
      const program = this.webgl!.compile_program(new StarShader());
      if (program === null) {
        return false;
      } else {
        this.star_program = program;
      }
    }

    {
      const program = this.webgl!.compile_program(
        new StarShaderProjectedOntoNear(),
      );
      if (program === null) {
        return false;
      } else {
        this.star_projected_onto_near_program = program;
      }
    }

    {
      const program = this.webgl!.compile_program(new StarMapShader());
      if (program === null) {
        return false;
      } else {
        this.star_map_program = program;
      }
    }

    {
      const program = this.webgl!.compile_program(new WebglFlatShader());
      if (program === null) {
        return false;
      } else {
        this.flat_program = program;
      }
    }

    this.webgl_triangle = new Webgl3DObj(3, 3);
    this.webgl_triangle.add_vertex(
      new Float32Array([1.0, 0, 0.05773]),
      new Float32Array([0, 0]),
    );
    this.webgl_triangle.add_vertex(
      new Float32Array([1.0, -0.05, -0.02887]),
      new Float32Array([0, 0]),
    );
    this.webgl_triangle.add_vertex(
      new Float32Array([1.0, 0.05, -0.02887]),
      new Float32Array([0, 0]),
    );
    this.webgl_triangle.add_face([0, 2, 1]);
    this.webgl!.create(this.webgl_triangle);

    this.webgl_axis = WebglFlatObj.axis(2, [
      [10, 0.05],
      [2, 0.1],
    ]);
    this.webgl!.create(this.webgl_axis);

    this.webgl_circle = WebglFlatObj.circle(1.0, 20);
    this.webgl!.create(this.webgl_circle);

    this.log.info(`Created full webgl content`);
    return true;
  }

  redraw(client: WebglCanvasClient): void {
    if (this.webgl !== null) {
      const wh = this.application.get_resizable_content_size();
      if (this.current_wh != wh) {
        this.canvas.width = wh[0];
        this.canvas.height = wh[1];
        this.current_wh = wh;
      }

      client.redraw(this.webgl, this);
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
