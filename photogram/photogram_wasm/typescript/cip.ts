import { WasmCip } from "../pkg/photogram_wasm.js";

import { Logger } from "./log.js";
import { HtmlElement } from "./html.js";

import { CipImage } from "./cip_image.js";

export class Cip {
  log: Logger;
  /** The name within the WasmProject used to select this CIP */
  cip_name: string;
  /** The CIP index within the project */
  cip_number: number;
  wasm_cip: WasmCip | null = null;
  cip_image: CipImage;

  constructor(log: Logger) {
    this.log = log;
    this.wasm_cip = null;
    this.cip_name = "";
    this.cip_number = 0;
    this.cip_image = new CipImage();
  }

  /// Invoked by the project to set the cip appropriately
  set_cip(cip_number: number, cip_name: string, cip: WasmCip | null) {
    this.cip_number = cip_number;
    this.cip_name = cip_name;
    this.wasm_cip = cip;
  }

  name(): string | null {
    if (this.wasm_cip === null) {
      return null;
    } else {
      return this.cip_name;
    }
  }

  cip(): WasmCip | null {
    return this.wasm_cip;
  }

  is_valid(): boolean {
    return this.wasm_cip !== null;
  }

  set_cip_image_data(cip_name: string, data: Blob) {
    if (this.cip_name == cip_name) {
      this.cip_image.set_image_data(data);
    }
  }

  orient_camera_using_model_directions(max_np_error: number) {
    if (this.wasm_cip !== null) {
      this.wasm_cip.orient_camera_using_model_directions(max_np_error);
    }
  }

  repopulate() {
    /*
                ["Focus at", focus_at],
                ["Location", location],
                ["Orientation", orientation],
                ["Focused on", focused_on],
                ["Direction", direction],
                ["Up", up],
       */
    const cip_name = this.wasm_cip ? this.wasm_cip.image : "<no CIP>";
    const body_name = this.wasm_cip ? this.wasm_cip.camera.body : "<no CIP>";
    const lens_name = this.wasm_cip ? this.wasm_cip.camera.lens : "<no CIP>";
    const focal_length = this.wasm_cip
      ? this.wasm_cip.camera.focal_length.toString() + "mm"
      : "<no CIP>";
    const fovd = this.wasm_cip
      ? (
          Math.floor(
            (2 * (Math.atan(this.wasm_cip.camera.tan_hfovd) * 18000)) / 3.14159,
          ) / 100
        ).toString() + "°"
      : "<no CIP>";
    const fovh = this.wasm_cip
      ? (
          Math.floor(
            (2 * (Math.atan(this.wasm_cip.camera.tan_hfovh) * 18000)) / 3.14159,
          ) / 100
        ).toString() + "°"
      : "<no CIP>";

    HtmlElement.fold_all_of(".set-cip-name", null, (a, e) => {
      e.ele.innerHTML = cip_name;
      return a;
    });
    HtmlElement.fold_all_of(".set-body", null, (a, e) => {
      e.ele.innerHTML = body_name;
      return a;
    });
    HtmlElement.fold_all_of(".set-lens", null, (a, e) => {
      e.ele.innerHTML = lens_name;
      return a;
    });
    HtmlElement.fold_all_of(".set-focal-length", null, (a, e) => {
      e.ele.innerHTML = focal_length;
      return a;
    });
    HtmlElement.fold_all_of(".set-fovd", null, (a, e) => {
      e.ele.innerHTML = fovd;
      return a;
    });
    HtmlElement.fold_all_of(".set-fovh", null, (a, e) => {
      e.ele.innerHTML = fovh;
      return a;
    });
  }
}
