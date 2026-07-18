import { WasmNamedPoint, WasmQuatf64, } from "../pkg/photogram_wasm.js";
import { HtmlElement } from "./html.js";
import { UndoBuffer } from "./undo.js";
import { MappedNps } from "./mapped_nps.js";
class UndoableNpAdd {
    constructor(project, np_name, np_color) {
        if (project.wasm_project !== null) {
            if (project.wasm_project.nps.get_pt(np_name) === undefined) {
                this.np_name = np_name;
                this.np_color = np_color;
                return;
            }
        }
        throw new Error("No WasmProject or np_name already exists");
    }
    fwd_text() {
        return [`NpAdd(${this.np_name}, ${this.np_color})`];
    }
    rev_text() {
        return [`NpDelete(${this.np_name})`];
    }
    fwd(p) {
        const wasm_np = new WasmNamedPoint(this.np_name, this.np_color);
        p.wasm_project.nps.add_pt(wasm_np);
        p.np_changed(false, this.np_name);
    }
    rev(p) {
        p.wasm_project.nps.delete_pt(this.np_name);
        p.np_changed(false, this.np_name);
    }
}
class UndoableNpDelete {
    constructor(project, np_name) {
        if (project.wasm_project !== null) {
            const np = project.wasm_project.nps.get_pt(np_name);
            if (np !== undefined) {
                this.np_name = np_name;
                this.np = np;
                return;
            }
        }
        throw new Error("No WasmProject or np_name does not exist");
    }
    fwd_text() {
        return [`NpDelete(${this.np_name})`];
    }
    rev_text() {
        return [
            `NpAdd(${this.np_name}, ${this.np.color})`,
            `NpSetModel(${this.np_name}, ${this.np.at_infinity}, ${this.np.model_as_array()}, ${this.np.error})`
        ];
    }
    fwd(p) {
        p.wasm_project.nps.delete_pt(this.np_name);
        p.np_changed(false, this.np_name);
    }
    rev(p) {
        p.wasm_project.nps.add_pt(this.np);
        if (this.np.at_infinity) {
            p.wasm_project.nps.set_direction(this.np_name, this.np.model_as_array());
        }
        else {
            p.wasm_project.nps.set_model(this.np_name, this.np.model_as_array(), this.np.error);
        }
        p.np_changed(false, this.np_name);
    }
}
class UndoableNpRename {
    constructor(project, np_name, new_np_name) {
        if (project.wasm_project !== null && false) {
            const np = project.wasm_project.nps.get_pt(np_name);
            const new_np = project.wasm_project.nps.get_pt(new_np_name);
            if (np !== undefined && new_np === undefined) {
                this.np_name = np_name;
                this.new_np_name = new_np_name;
                return;
            }
        }
        throw new Error("No WasmProject or np_name does not exist or new np name *does* exist");
    }
    fwd_text() {
        return [`NpRename(${this.np_name}, ${this.new_np_name})`];
    }
    rev_text() {
        return [`NpRename(${this.new_np_name}, ${this.np_name})`];
    }
    fwd(p) {
        p.np_changed(false, this.np_name);
        //     p.wasm_project!.nps.rename_pt(this.np_name, this.new_np_name);
    }
    rev(p) {
        p.np_changed(false, this.new_np_name);
        //    p.wasm_project!.nps.rename_pt(this.new_np_name, this.np_name);
    }
}
class UndoableNpSetModel {
    constructor(project, np_name, at_infinity, location, error) {
        if (project.wasm_project !== null) {
            const np = project.wasm_project.nps.get_pt(np_name);
            if (np !== undefined) {
                this.np_name = np_name;
                this.orig_data = [np.at_infinity, np.model_as_array(), np.error];
                this.new_data = [np.at_infinity, np.model_as_array(), np.error];
                if (at_infinity !== undefined) {
                    this.new_data[0] = at_infinity;
                }
                if (location !== undefined) {
                    this.new_data[1] = location.array;
                }
                if (error !== undefined) {
                    this.new_data[2] = error;
                }
                return;
            }
        }
        throw new Error("No WasmProject or np_name does not exist");
    }
    fwd_text() {
        return [`NpSetModel(${this.np_name}, ${this.new_data[0]}, ${this.new_data[1]}, ${this.new_data[2]})`];
    }
    rev_text() {
        return [`NpSetModel(${this.np_name}, ${this.orig_data[0]}, ${this.orig_data[1]}, ${this.orig_data[2]})`];
    }
    fwd(p) {
        if (this.new_data[0]) {
            p.wasm_project.nps.set_direction(this.np_name, this.new_data[1]);
        }
        else {
            p.wasm_project.nps.set_model(this.np_name, this.new_data[1], this.new_data[2]);
        }
        p.np_changed(true, this.np_name);
    }
    rev(p) {
        if (this.orig_data[0]) {
            p.wasm_project.nps.set_direction(this.np_name, this.orig_data[1]);
        }
        else {
            p.wasm_project.nps.set_model(this.np_name, this.orig_data[1], this.orig_data[2]);
        }
        p.np_changed(true, this.np_name);
    }
}
class UndoableNpSetColor {
    constructor(project, np_name, np_color) {
        if (project.wasm_project !== null) {
            const np = project.wasm_project.nps.get_pt(np_name);
            if (np !== undefined) {
                this.np_name = np_name;
                this.new_np_color = np_color;
                this.orig_np_color = np.color;
                return;
            }
        }
        throw new Error("No WasmProject or np_name does not exist");
    }
    fwd_text() {
        return [`NpSetColor(${this.np_name}, ${this.new_np_color})`];
    }
    rev_text() {
        return [`NpSetColor(${this.np_name}, ${this.orig_np_color})`];
    }
    fwd(p) {
        p.wasm_project.nps.set_color(this.np_name, this.new_np_color);
        p.np_changed(true, this.np_name);
    }
    rev(p) {
        p.wasm_project.nps.set_color(this.np_name, this.orig_np_color);
        p.np_changed(true, this.np_name);
    }
}
class UndoablePmsMove {
    constructor(project, np_name, pxy) {
        if (project.wasm_project !== null) {
            if (project.wasm_project.nps.get_pt(np_name) !== undefined) {
                const cip = project.get_wasm_cip();
                if (cip !== null) {
                    const n = cip.pms.mapping_of_name(np_name);
                    if (n !== undefined) {
                        this.cip_name = project.get_cip().name();
                        this.np_name = np_name;
                        this.pxy = pxy;
                        const xy = cip.pms.get_xy(n);
                        this.prev_pxy = [xy[0], xy[1]];
                        return;
                    }
                }
            }
        }
        throw new Error("Project did not have cip name and np_name");
    }
    fwd_text() {
        return [`PmsMove(${this.cip_name}, ${this.np_name}, ${this.pxy})`];
    }
    rev_text() {
        return [`PmsMove(${this.cip_name}, ${this.np_name}, ${this.prev_pxy})`];
    }
    fwd(p) {
        const pms = p.get_cip_by_name(this.cip_name).pms;
        const n = pms.mapping_of_name(this.np_name);
        pms.set_xy(n, this.pxy[0], this.pxy[1]);
        p.pm_changed(true, this.np_name);
    }
    rev(p) {
        const pms = p.get_cip_by_name(this.cip_name).pms;
        const n = pms.mapping_of_name(this.np_name);
        pms.set_xy(n, this.prev_pxy[0], this.prev_pxy[1]);
        p.pm_changed(true, this.np_name);
    }
}
class UndoablePmsAdd {
    constructor(project, np_name, pxy, uncertainty = 0) {
        if (project.wasm_project !== null) {
            if (project.wasm_project.nps.get_pt(np_name) !== undefined) {
                const cip = project.get_wasm_cip();
                if (cip !== null) {
                    const n = cip.pms.mapping_of_name(np_name);
                    if (n === undefined) {
                        this.cip_name = project.get_cip().name();
                        this.np_name = np_name;
                        this.pxy = pxy;
                        this.uncertainty = uncertainty;
                        return;
                    }
                }
            }
        }
        throw new Error("Project did not have cip name and np_name");
    }
    fwd_text() {
        return [`PmsAdd(${this.cip_name}, ${this.np_name}, ${this.pxy}, ${this.uncertainty})`];
    }
    rev_text() {
        return [`PmsDelete(${this.cip_name}, ${this.np_name})`];
    }
    fwd(p) {
        const pms = p.get_cip_by_name(this.cip_name).pms;
        const n = pms.add_mapping(p.get_wasm_nps(), this.np_name);
        if (n !== undefined) {
            pms.set_xy(n, this.pxy[0], this.pxy[1]);
            // pms.set_uncertainty(n, this.uncertainty);
        }
        // Adding a mapping impacts the NPs
        p.np_changed(false, this.np_name);
    }
    rev(p) {
        const pms = p.get_cip_by_name(this.cip_name).pms;
        const n = pms.mapping_of_name(this.np_name);
        if (n !== undefined) {
            pms.remove_mapping(n);
        }
        // Deleting a mapping impacts the NPs
        p.np_changed(false, this.np_name);
    }
}
class UndoablePmsDelete {
    constructor(project, np_name) {
        if (project.wasm_project !== null) {
            if (project.wasm_project.nps.get_pt(np_name) !== undefined) {
                const cip = project.get_wasm_cip();
                if (cip !== null) {
                    const n = cip.pms.mapping_of_name(np_name);
                    if (n !== undefined) {
                        this.cip_name = project.get_cip().name();
                        this.np_name = np_name;
                        const xy = cip.pms.get_xy_err(n);
                        this.orig_pxy = [xy[0], xy[1]];
                        this.orig_uncertainty = xy[2];
                        return;
                    }
                }
            }
        }
        throw new Error("Project did not have cip name and np_name");
    }
    fwd_text() {
        return [`PmsDelete(${this.cip_name}, ${this.np_name})`];
    }
    rev_text() {
        return [`PmsAdd(${this.cip_name}, ${this.np_name}, ${this.orig_pxy}, ${this.orig_uncertainty})`];
    }
    fwd(p) {
        const pms = p.get_cip_by_name(this.cip_name).pms;
        const n = pms.mapping_of_name(this.np_name);
        if (n !== undefined) {
            pms.remove_mapping(n);
        }
        // Deleting a mapping impacts the NPs
        p.np_changed(false, this.np_name);
    }
    rev(p) {
        const pms = p.get_cip_by_name(this.cip_name).pms;
        const n = pms.add_mapping(p.get_wasm_nps(), this.np_name);
        if (n !== undefined) {
            pms.set_xy(n, this.orig_pxy[0], this.orig_pxy[1]);
            // pms.set_uncertainty(n, this.uncertainty);
        }
        // Adding a mapping impacts the NPs
        p.np_changed(false, this.np_name);
    }
}
class UndoableCameraSetOrientation {
    constructor(project, orientation) {
        if (project.wasm_project !== null) {
            const wasm_cip = project.get_wasm_cip();
            if (wasm_cip !== null) {
                this.cip_name = project.get_cip().name();
                this.orig_orientation = wasm_cip.camera.orientation;
                this.new_orientation = WasmQuatf64.unit();
                this.new_orientation.set_array(orientation.array);
            }
        }
        throw new Error("Project did not have cip name");
    }
    fwd_text() {
        return [`CameraSetOrientation(${this.cip_name}, ${this.new_orientation})`];
    }
    rev_text() {
        return [`CameraSetOrientation(${this.cip_name}, ${this.orig_orientation})`];
    }
    fwd(p) {
        const camera = p.get_cip_by_name(this.cip_name).camera;
        camera.orientation = this.new_orientation;
        p.camera_changed(true);
    }
    rev(p) {
        const camera = p.get_cip_by_name(this.cip_name).camera;
        camera.orientation = this.orig_orientation;
        p.camera_changed(true);
    }
}
export class Project {
    constructor(application, log, project_set, cip) {
        this.locator = null;
        this.wasm_project = null;
        this.modified = false;
        this.promise_epoch = 0;
        this.thumbnail_width = 256;
        this.clients = [];
        this.application = application;
        this.log = log;
        this.project_set = project_set;
        this.cip = cip;
        this.thumbnails = new Map();
        this.undo_buffer = new UndoBuffer();
        this._mapped_nps = new MappedNps(this);
        this.clients = [];
    }
    /** Invoked when the project is 'deselected' */
    clear_clients() { this.clients = []; }
    /** Invoked by clients when the project is 'selected' */
    add_client(c) { this.clients.push(c); }
    get_undo_buffer() {
        return this.undo_buffer;
    }
    undo() {
        const x = this.undo_buffer.undo();
        if (x === null) {
            return false;
        }
        x.rev(this);
        return true;
    }
    redo() {
        const x = this.undo_buffer.redo();
        if (x === null) {
            return false;
        }
        x.fwd(this);
        return true;
    }
    mapped_nps() {
        return this._mapped_nps;
    }
    set_focus(x, y) {
        this.mapped_nps().set_focus(x, y);
    }
    invoke_clients(cb) {
        for (const c of this.clients) {
            cb(c);
        }
    }
    /** Invoked whenever an NP is changed, or the set as a whole if np_name is undefined  */
    np_changed(_data_only, _np_name) {
        this._mapped_nps.rebuild_nps();
        this._mapped_nps.map_with_cip();
        this.invoke_clients((c) => c.project_np_changed(this));
    }
    /** Invoked whenever the camera is changed (lens, body, orientation, position, etc */
    camera_changed(_data_only) {
        this._mapped_nps.map_with_cip();
        this.invoke_clients((c) => c.project_camera_changed(this));
    }
    /** Invoked whenever a point mapping is changed, or the set as a whole if np_name is undefined */
    pm_changed(_data_only, _np_name) {
        this._mapped_nps.map_with_cip();
        this.invoke_clients((c) => c.project_pm_changed(this));
    }
    /** Invoked whenever the CIP changes (e.g. to new CIP, etc) */
    cip_changed(_data_only) {
        this._mapped_nps.map_with_cip();
        this.invoke_clients((c) => c.project_cip_changed(this));
    }
    /** Invoked whenever the MappedNPs changes (sort order etc) */
    mapped_changed() {
        this.invoke_clients((c) => c.project_mapped_nps_changed(this));
    }
    nps_get_new_name() {
        if (this.wasm_project === null) {
            return "No NPS loaded";
        }
        const wasm_nps = this.wasm_project.nps;
        let i = 0;
        let np_name = "";
        while (true) {
            np_name = `np_${i}`;
            if (wasm_nps.get_pt(np_name) === undefined) {
                break;
            }
            i += 1;
        }
        return np_name;
    }
    nps_add(name) {
        try {
            const np_add = new UndoableNpAdd(this, name, "#FF0");
            this.undo_buffer.do_action(np_add);
            np_add.fwd(this);
            this.log.info(`Added NP ${name}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to add NP ${name}`);
            return false;
        }
    }
    nps_delete(name) {
        try {
            const np_del = new UndoableNpDelete(this, name);
            this.undo_buffer.do_action(np_del);
            np_del.fwd(this);
            this.log.info(`Deleted NP ${name}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to delete NP ${name}`);
            return false;
        }
    }
    nps_rename(name, new_name) {
        try {
            const np_set = new UndoableNpRename(this, name, new_name);
            this.undo_buffer.do_action(np_set);
            np_set.fwd(this);
            this.log.info(`Renamed NP ${name} to ${new_name}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to rename NP ${name} to ${new_name}`);
            return false;
        }
    }
    nps_set_model(name, at_infinity, model, error) {
        try {
            const np_set = new UndoableNpSetModel(this, name, at_infinity, model, error);
            this.undo_buffer.do_action(np_set);
            np_set.fwd(this);
            this.log.info(`Set NP model ${name}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to set NP model ${name}`);
            return false;
        }
    }
    nps_set_color(name, color) {
        try {
            const np_set = new UndoableNpSetColor(this, name, color);
            this.undo_buffer.do_action(np_set);
            np_set.fwd(this);
            this.log.info(`Set NP color ${name} to ${color}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to set NP color ${name} to ${color}`);
            return false;
        }
    }
    pms_move(name, pxy) {
        try {
            const pms_move = new UndoablePmsMove(this, name, pxy);
            this.undo_buffer.do_action(pms_move);
            pms_move.fwd(this);
            this.log.info(`Moved point mapping for ${name} to ${pxy}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to moved point mapping for ${name} to ${pxy}`);
            return false;
        }
    }
    pms_add(name, pxy, uncertainty) {
        try {
            const pms_add = new UndoablePmsAdd(this, name, pxy, uncertainty);
            this.undo_buffer.do_action(pms_add);
            pms_add.fwd(this);
            this.log.info(`Added point mapping for ${name} to ${pxy}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to add point mapping for ${name} to ${pxy}`);
            return false;
        }
    }
    pms_delete(name) {
        try {
            const pms_delete = new UndoablePmsDelete(this, name);
            this.undo_buffer.do_action(pms_delete);
            pms_delete.fwd(this);
            this.log.info(`Deleted point mapping for ${name}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to delete point mapping for ${name}`);
            return false;
        }
    }
    camera_set_orientation(quat) {
        try {
            const undoable = new UndoableCameraSetOrientation(this, quat);
            this.undo_buffer.do_action(undoable);
            undoable.fwd(this);
            this.log.info(`Reoriented camera for CIP ${this.get_cip().cip_name} to ${quat}`);
            return true;
        }
        catch (e) {
            this.log.error(`Failed to reorient camera`);
            return false;
        }
    }
    get_cip() {
        return this.cip;
    }
    get_cip_name(cip) {
        var _a;
        const name = (_a = this.wasm_project) === null || _a === void 0 ? void 0 : _a.cip_name(cip);
        return name ? name : null;
    }
    repopulate() {
        const project_locator = this.locator ? this.locator : "<no project open>";
        const nps_length = this.wasm_project
            ? this.wasm_project.nps.num_points().toString()
            : "<no project open>";
        HtmlElement.fold_all_of(".set-project-name", null, (a, e) => {
            e.ele.innerHTML = project_locator;
            return a;
        });
        HtmlElement.fold_all_of(".set-nps-length", null, (a, e) => {
            e.ele.innerHTML = nps_length;
            return a;
        });
    }
    is_modified() {
        return this.modified;
    }
    get_wasm_nps() {
        if (this.wasm_project === null) {
            return null;
        }
        return this.wasm_project.nps;
    }
    get_wasm_cip() {
        return this.cip.wasm_cip;
    }
    cancel_all_promises() {
        this.promise_epoch += 1;
    }
    load_project(locator) {
        this.cancel_all_promises();
        this.cip.set_cip("", null);
        this.locator = locator;
        this.modified = false;
        this.thumbnails.clear();
        this.project_set.load_project(this.locator, this.project_loaded.bind(this), this.project_load_error.bind(this));
    }
    project_loaded(wasm_project) {
        this.wasm_project = wasm_project;
        this.application.project_load_completed(true);
        this.log.info(`Project ${this.locator} loaded`);
        this.thumbnails = new Map();
        for (let i = 0; i < this.wasm_project.ncips(); i++) {
            const cip_name = this.wasm_project.cip_name(i);
            const promise = this.project_set.promise_fetch_thumbnail(this.locator, cip_name, this.thumbnail_width);
            if (promise !== null) {
                promise
                    .then((jpg) => {
                    this.thumbnail_loaded(this.promise_epoch, cip_name, jpg);
                })
                    .catch(this.log_exception.bind(this));
            }
        }
    }
    log_exception(e) {
        this.log.error(e.message);
    }
    thumbnail_loaded(epoch, cip_name, jpg) {
        if (epoch != this.promise_epoch) {
            return;
        }
        this.log.info(`Thumbnail ${cip_name} loaded for project ${this.locator}`);
        const img = new Image();
        img.src = URL.createObjectURL(jpg);
        this.thumbnails.set(cip_name, img);
        this.application.thumbnails_updated();
    }
    project_load_error(e) {
        this.log.error(e);
        this.locator = null;
        this.application.project_load_completed(false);
    }
    save_project(locator) {
        locator = locator ? locator : this.locator;
        if (this.wasm_project === null || locator === null) {
            this.application.project_save_completed(false);
            return;
        }
        this.project_set.save_project(locator, this.wasm_project, this.project_saved.bind(this), this.project_save_error.bind(this));
    }
    project_saved() {
        this.modified = false;
        this.log.info(`Project ${this.locator} saved`);
        this.application.project_save_completed(true);
    }
    project_save_error(e) {
        this.log.error(e);
        this.application.project_save_completed(false);
    }
    /** Fetch the thumbnails (for a server project; does not work for 'local')
     *
     */
    get_cip_by_name(name) {
        if (this.wasm_project === null) {
            return null;
        }
        return this.wasm_project.cip(name);
    }
    cip_image_loaded(epoch, cip_name, jpg) {
        if (epoch == this.promise_epoch) {
            this.log.info(`Image ${cip_name} loaded for project ${this.locator}`);
            this.cip.set_cip_image_data(cip_name, jpg);
        }
    }
    set_cip(cip_name) {
        const wasm_cip = this.get_cip_by_name(cip_name);
        this.cip.set_cip(cip_name, wasm_cip);
        if (this.locator !== null) {
            const promise = this.project_set.promise_fetch_image(this.locator, cip_name);
            if (promise !== null)
                promise
                    .then((jpg) => {
                    this.cip_image_loaded(this.promise_epoch, cip_name, jpg);
                })
                    .catch(this.log_exception.bind(this));
        }
    }
}
