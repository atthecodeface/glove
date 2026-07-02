import { HtmlElement } from "./html.js";
import { Logger } from "./log.js";
import { MousePressActions } from "./mouse.js";
import { Webgl } from "./web_gl.js";
import { ZoomedWindow } from "./zoomed_window.js";

import { WebglCanvas, WebglCanvasClient } from "./webgl_canvas.js";

import { Application } from "./application.js";

export class StarCalibration implements WebglCanvasClient {
  application: Application;
  log: Logger;
  html_div: HtmlElement;
  zoomed_window: ZoomedWindow;

  constructor(application: Application, log: Logger, html_div: HtmlElement) {
    this.application = application;
    this.log = log;
    this.html_div = html_div;

    this.zoomed_window = new ZoomedWindow([10, 10]);
  }

  resize(w: number, h: number) {
    this.zoomed_window.scr_resize(w, h);
  }

  redraw(webgl: Webgl, webgl_canvas: WebglCanvas): void {
    const size = webgl_canvas.size();
    const w = size[0];
    const h = size[1];

    const ar = 1.6;
    let xsc = 1.0;
    let ysc = w / h / ar;

    if (ysc > 1.0) {
      xsc /= ysc;
      ysc = 1.0;
    }
    // Set the whole canvas as viewport
    webgl.set_viewport([0, 0, 0, 0]);
    webgl.clear_buffer();

    const project = this.application.current_project();
    const cip = project.get_cip();
    if (cip.is_valid()) {
      const texture_ready = cip.cip_image.webgl_texture_ready();
      const texture = cip.cip_image.get_webgl_texture(webgl);
      if (texture !== null) {
        if (!texture_ready) {
          const img_size = cip.cip_image.get_size();
          this.zoomed_window.set_img(img_size[0], img_size[1]);
        }
        webgl.use_program(webgl_canvas.image_program);
        const zoom = this.zoomed_window.get_zoom();
        const ofs = this.zoomed_window.rel_cxy();
        // Map the ofs of the *texture* (in range 0 to 1) to the rectangle in space which is -1 to 1
        ofs[0] = ofs[0] * 2 - 1;
        ofs[1] = ofs[1] * 2 - 1;
        // Translate by ofs and then zoom
        webgl.model.set([
          zoom,
          0,
          0,
          -(zoom * ofs[0]),

          0,
          zoom,
          0,
          zoom * ofs[1],

          0,
          0,
          1,
          0,

          0,
          0,
          0,
          1,
        ]);
        webgl.set_uniform_model();
        webgl.set_texture(texture);
        webgl.draw(webgl_canvas.webgl_rectangle!);
      }
    }
  }

  user_press(_xy: [number, number], actions: MousePressActions): void {
    actions.can_pan = true;
    actions.can_drag = false;
  }
  user_press_move(_start_xy: [number, number], _xy: [number, number]): void {}
  user_press_cancel(_start_xy: [number, number]): void {}

  user_rotate(_xy: [number, number], _angle: number): void {}

  user_pan(xy: [number, number], dxy: [number, number]): void {
    this.zoomed_window.user_pan(xy, dxy);
    this.application.set_view_needs_update();
  }
  user_zoom(cxy: [number, number], factor: number): void {
    this.zoomed_window.user_zoom(cxy, factor);
    this.application.set_view_needs_update();
  }

  drag_start(_start_xy: [number, number], _xy: [number, number]): void {}

  drag_to(
    _start_xy: [number, number],
    _old_xy: [number, number],
    _new_xy: [number, number],
  ): void {}

  drag_end(_start_xy: [number, number], _xy: [number, number]): void {}

  user_release(_start_xy: [number, number], _xy: [number, number]): void {}
}
