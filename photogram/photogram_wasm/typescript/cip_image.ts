import { Webgl, WebglTexture } from "./web_gl.js";

export class CipImage {
  image: HTMLImageElement;
  webgl_texture: WebglTexture | null = null;
  image_loaded: boolean = false;

  constructor() {
    this.image = new Image();
  }

  reset_image_data() {
    this.webgl_texture = null;
    this.image_loaded = false;
  }

  set_image_data(data: Blob) {
    // Note WebglTexture has bind_to_image(this.image!);
    // However, getting a blob into an image (it is JPEG encoded probably) requires going through the Image element for some reason
    this.image.src = URL.createObjectURL(data);
    this.image_loaded = true;
    this.webgl_texture = null;
  }

  get_size(): [number, number] {
    return [this.image.width, this.image.height];
  }

  webgl_texture_ready(): boolean {
    return this.webgl_texture !== null && this.image_loaded;
  }

  get_webgl_texture(webgl: Webgl): WebglTexture | null {
    if (this.webgl_texture === null && this.image_loaded) {
      this.webgl_texture = new WebglTexture(webgl, this.image);
      this.webgl_texture.bind_to_image(this.image);
    }
    return this.webgl_texture;
  }
}
