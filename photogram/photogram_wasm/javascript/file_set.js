import * as utils from "./utils.js";
export var FileKind;
(function (FileKind) {
    FileKind[FileKind["Unknown"] = 0] = "Unknown";
    FileKind[FileKind["Cdb"] = 1] = "Cdb";
    FileKind[FileKind["Camera"] = 2] = "Camera";
    FileKind[FileKind["Project"] = 3] = "Project";
    FileKind[FileKind["Pms"] = 4] = "Pms";
    FileKind[FileKind["Nps"] = 5] = "Nps";
    FileKind[FileKind["Cips"] = 6] = "Cips";
})(FileKind || (FileKind = {}));
export class BaseFile {
    constructor() {
        this.file_kind = FileKind.Unknown;
    }
    kind() {
        return this.file_kind;
    }
    of_obj(obj) {
        return new UnknownFile(obj);
    }
    obj_is(_obj) {
        return true;
    }
}
export class UnknownFile extends BaseFile {
    constructor(obj) {
        super();
        this.obj = obj;
    }
    static find_data_type(json) {
        const obj = utils.parse_json(json);
        if (CdbFile.obj_is(obj)) {
            return new CdbFile(obj);
        }
        if (CameraFile.obj_is(obj)) {
            return new CameraFile(obj);
        }
        if (ProjectFile.obj_is(obj)) {
            return new ProjectFile(obj);
        }
        if (PmsFile.obj_is(obj)) {
            return new PmsFile(obj);
        }
        if (NpsFile.obj_is(obj)) {
            return new NpsFile(obj);
        }
        if (CipsFile.obj_is(obj)) {
            return new CipsFile(obj);
        }
        return new UnknownFile(obj);
    }
}
export class CdbFile extends BaseFile {
    constructor(obj) {
        super();
        this.file_kind = FileKind.Cdb;
        this.bodies = obj.bodies;
        this.lenses = obj.lenses;
    }
    static obj_is(obj) {
        return obj.hasOwnProperty("bodies") && obj.hasOwnProperty("lenses");
    }
}
export class CameraFile extends BaseFile {
    constructor(obj) {
        super();
        this.file_kind = FileKind.Camera;
        this.body = obj.body;
        this.lens = obj.lens;
    }
    static obj_is(obj) {
        return (obj.hasOwnProperty("body") &&
            obj.hasOwnProperty("lens") &&
            obj.hasOwnProperty("position") &&
            obj.hasOwnProperty("direction"));
    }
}
export class ProjectFile extends BaseFile {
    constructor(obj) {
        super();
        this.file_kind = FileKind.Project;
        this.cdb = "some cdb";
        this.nps = "some nps";
        this.cips = [];
        for (const c of obj.cips) {
            if (CipsFile.obj_is(c)) {
                this.cips.push(new CipsFile(c));
            }
        }
    }
    static obj_is(obj) {
        return (obj.hasOwnProperty("cdb") &&
            obj.hasOwnProperty("nps") &&
            obj.hasOwnProperty("cips"));
    }
}
export class PmsFile extends BaseFile {
    constructor(_obj) {
        super();
        this.file_kind = FileKind.Pms;
    }
    static obj_is(obj) {
        if (!utils.is_array(obj)) {
            return false;
        }
        const obj_a = obj;
        if (obj_a.length == 0) {
            return false;
        }
        if (!utils.is_array(obj_a[0])) {
            return false;
        }
        const obj_a2 = obj_a[0];
        if (obj_a2.length != 3) {
            return false;
        }
        return (utils.is_string(obj_a2[0]) &&
            utils.is_array(obj_a2[1]) &&
            utils.is_float(obj_a2[2]));
    }
}
export class NpsFile extends BaseFile {
    constructor(obj) {
        super();
        this.file_kind = FileKind.Nps;
        this.points = [];
        for (const p of obj) {
            this.points.push(p);
        }
    }
    static obj_is(obj) {
        console.log("Is nps?", obj);
        if (!utils.is_array(obj)) {
            return false;
        }
        const obj_a = obj;
        if (obj_a.length == 0) {
            return false;
        }
        if (!utils.is_array(obj_a[0])) {
            return false;
        }
        const obj_a2 = obj_a[0];
        if (obj_a2.length != 3) {
            return false;
        }
        console.log("Got this far!");
        return (utils.is_string(obj_a2[0]) &&
            utils.is_string(obj_a2[1]) &&
            utils.is_array(obj_a2[2]));
    }
}
export class CipsFile extends BaseFile {
    constructor(_obj) {
        super();
        this.file_kind = FileKind.Cips;
    }
    static obj_is(obj) {
        if (!utils.is_array(obj)) {
            return false;
        }
        const obj_a = obj;
        if (obj_a.length == 0) {
            return false;
        }
        if (!utils.is_array(obj_a[0])) {
            return false;
        }
        const obj_a2 = obj_a[0];
        if (obj_a2.length != 3) {
            return false;
        }
        return (utils.is_string(obj_a2[0]) &&
            utils.is_string(obj_a2[1]) &&
            utils.is_array(obj_a2[2]));
    }
}
export class FileSet {
    constructor(storage, file_list_callback) {
        this.filenames_by_kind = new Map();
        this.storage = storage;
        this.file_list_callback = file_list_callback;
    }
    get_file_list() {
        this.storage.request_get_file_list(this.file_list_received.bind(this));
    }
    files_of_kind(kind) {
        const filenames = this.filenames_by_kind.get(kind);
        if (filenames === undefined) {
            return [];
        }
        else {
            return filenames;
        }
    }
    file_list_received(success) {
        if (success) {
            this.filenames_by_kind.clear();
            for (const f of this.storage.dir().files_of_type("json")) {
                const filename = f + ".json";
                const json = this.storage.load_file(filename);
                if (json === null) {
                    continue;
                }
                const file = UnknownFile.find_data_type(json);
                const file_kind = file.kind();
                if (!this.filenames_by_kind.has(file_kind)) {
                    this.filenames_by_kind.set(file_kind, []);
                }
                this.filenames_by_kind.get(file.kind()).push(filename);
            }
            this.file_list_callback();
        }
    }
    load_file_as_str(filename) {
        return this.storage.load_file(filename);
    }
    load_file_as_obj(filename, kind) {
        const json = this.storage.load_file(filename);
        if (json === null) {
            return null;
        }
        const file = UnknownFile.find_data_type(json);
        if (file.kind() !== kind) {
            return null;
        }
        return file;
    }
    save_file(filename, _file, contents) {
        this.storage.save_file(filename, contents);
    }
}
