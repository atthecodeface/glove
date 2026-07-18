import { HtmlElement } from "./html.js";
import { CipImage } from "./cip_image.js";
export class Cip {
    constructor(log) {
        this.wasm_cip = null;
        this.log = log;
        this.wasm_cip = null;
        this.cip_name = "";
        this.cip_image = new CipImage();
    }
    set_cip(cip_name, cip) {
        this.cip_name = cip_name;
        this.wasm_cip = cip;
        /*
        const dl = this.project_set.decode_locator(this.project_name);
        const me = this;
        this.mesh = [];
        if (dl[0] == "server") {
            this.image.src = this.server_project.image_uri(this.cip_of_project);
            this.server_project.issue_fetch_mesh(this.cip_of_project,function() {
                                                             me.update_mesh();});
            this.server_project.issue_fetch_interestings(this.cip_of_project,function() {me.update_interesting();});
        } else {
            this.image.src = cip.img;
        }
        */
    }
    name() {
        if (this.wasm_cip === null) {
            return null;
        }
        else {
            return this.cip_name;
        }
    }
    cip() {
        return this.wasm_cip;
    }
    is_valid() {
        return this.wasm_cip !== null;
    }
    set_cip_image_data(cip_name, data) {
        if (this.cip_name == cip_name) {
            this.cip_image.set_image_data(data);
        }
    }
    orient_camera_using_model_directions(max_np_error) {
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
            ? (Math.floor((2 * (Math.atan(this.wasm_cip.camera.tan_hfovd) * 18000)) / 3.14159) / 100).toString() + "°"
            : "<no CIP>";
        const fovh = this.wasm_cip
            ? (Math.floor((2 * (Math.atan(this.wasm_cip.camera.tan_hfovh) * 18000)) / 3.14159) / 100).toString() + "°"
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
