import { WasmMat4f32 } from "../pkg/photogram_wasm.js";

import { HtmlElement } from "./html.js";
import { Webgl } from "./web_gl.js";
import { Webgl3DObj } from "./web_gl_3d_obj.js";
import { Logger } from "./log.js";
import { Application } from "./application.js";
import { Mouse, MouseClient, MousePressActions } from "./mouse.js";

import { ImageShader } from "./shaders.js";

export interface WebglCanvasClient extends MouseClient {
  resize(w: number, h: number): void;
  redraw(webgl: Webgl, webgl_canvas: WebglCanvas): void;
}

export class WebglCanvas {
  application: Application;
  log: Logger;
  webgl_canvas: HtmlElement;
  canvas: HTMLCanvasElement;
  mouse: Mouse;

  webgl: Webgl | null = null;

  image_program: number = 0;

  webgl_rectangle: Webgl3DObj | null = null;

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
      const program = this.webgl!.compile_program(new ImageShader());
      if (program === null) {
        return false;
      }
      this.image_program = program;
    }

    this.webgl_rectangle = new Webgl3DObj(
      4,
      2,
      [-1, 1, 0, -1, -1, 0, 1, -1, 0, 1, 1, 0],
      [0, 0, 0, 1, 1, 1, 1, 0],
      [0, 2, 1, 2, 3, 0],
    );
    this.webgl!.create(this.webgl_rectangle);

    this.log.info(`Created full webgl content`);
    return true;
  }

  size(): [number, number] {
    return this.current_wh;
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
