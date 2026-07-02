import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";
import { MousePressActions } from "./mouse.js";
import { Webgl } from "./web_gl.js";

import { WebglCanvas, WebglCanvasClient } from "./webgl_canvas.js";

import { Application } from "./application.js";

export class StarCalibration implements WebglCanvasClient {
  application: Application;
  log: Logger;
  html_div: HtmlElement;

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;

    this.repopulate();
  }

  repopulate() {}
  redraw(_webgl: Webgl, _webgl_canvas: WebglCanvas): void {}
  user_press(_xy: [number, number], _actions: MousePressActions): void {}
  user_press_move(_start_xy: [number, number], _xy: [number, number]): void {}
  user_press_cancel(_start_xy: [number, number]): void {}
  user_pan(_xy: [number, number], _dxy: [number, number]): void {}
  user_rotate(_xy: [number, number], _angle: number): void {}

  user_zoom(_cxy: [number, number], _factor: number): void {}

  drag_start(_start_xy: [number, number], _xy: [number, number]): void {}

  drag_to(
    _start_xy: [number, number],
    _old_xy: [number, number],
    _new_xy: [number, number],
  ): void {}

  drag_end(_start_xy: [number, number], _xy: [number, number]): void {}

  user_release(_start_xy: [number, number], _xy: [number, number]): void {}
}
