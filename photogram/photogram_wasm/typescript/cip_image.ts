import { Webgl, WebglTexture } from "./web_gl.js";

export class CipImage {
  image: HTMLImageElement;
  webgl_texture: WebglTexture | null = null;
  filename: string | null = null;

  constructor() {
    this.image = new Image();
  }

  set_image_name(filename: string) {
    this.image.src = filename;
    this.webgl_texture = null;
    this.filename = filename;
    // Note WebglTexture has bind_to_image(this.image!);
  }

  create_webgl_texture(webgl: Webgl) {
    this.webgl_texture = new WebglTexture(webgl, this.image);
  }

  texture(): WebglTexture | null {
    return this.webgl_texture;
  }
}
