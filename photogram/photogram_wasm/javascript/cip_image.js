import { WebglTexture } from "./web_gl.js";
export class CipImage {
    constructor() {
        this.webgl_texture = null;
        this.image_loaded = false;
        this.image = new Image();
    }
    reset_image_data() {
        this.webgl_texture = null;
        this.image_loaded = false;
    }
    set_image_data(data) {
        // Note WebglTexture has bind_to_image(this.image!);
        // However, getting a blob into an image (it is JPEG encoded probably) requires going through the Image element for some reason
        this.image.src = URL.createObjectURL(data);
        this.image_loaded = true;
        this.webgl_texture = null;
    }
    get_size() {
        return [this.image.width, this.image.height];
    }
    webgl_texture_ready() {
        return this.webgl_texture !== null && this.image_loaded;
    }
    get_webgl_texture(webgl) {
        if (this.webgl_texture === null && this.image_loaded) {
            this.webgl_texture = new WebglTexture(webgl, this.image);
            this.webgl_texture.bind_to_image(this.image);
        }
        return this.webgl_texture;
    }
}
