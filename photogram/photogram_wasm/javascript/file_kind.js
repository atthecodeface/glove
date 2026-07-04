import { WasmProject, WasmCipDesc, WasmCameraDatabase, WasmPointMappingSet, WasmNamedPointSet, } from "../pkg/photogram_wasm.js";
import * as utils from "./utils.js";
export var FileKind;
(function (FileKind) {
    FileKind[FileKind["Unknown"] = 0] = "Unknown";
    FileKind[FileKind["Cdb"] = 1] = "Cdb";
    FileKind[FileKind["Project"] = 2] = "Project";
    FileKind[FileKind["Pms"] = 3] = "Pms";
    FileKind[FileKind["Nps"] = 4] = "Nps";
    FileKind[FileKind["Cip"] = 5] = "Cip";
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
    to_json() {
        throw new Error("No means to generate Json for " + this.file_kind.toString());
    }
}
export class UnknownFile extends BaseFile {
    constructor(obj) {
        super();
        this.obj = obj;
    }
    static find_data_type(json) {
        {
            let p = ProjectFile.of_json(json);
            if (p !== null) {
                return p;
            }
        }
        {
            let c = CipFile.of_json(json);
            if (c !== null) {
                return c;
            }
        }
        {
            let c = CdbFile.of_json(json);
            if (c !== null) {
                return c;
            }
        }
        {
            let c = PmsFile.of_json(json);
            if (c !== null) {
                return c;
            }
        }
        {
            let c = NpsFile.of_json(json);
            if (c !== null) {
                return c;
            }
        }
        const obj = utils.parse_json(json);
        return new UnknownFile(obj);
    }
}
export class CdbFile extends BaseFile {
    constructor(cdb) {
        super();
        this.file_kind = FileKind.Cdb;
        this.cdb = cdb;
    }
    static of_json(json) {
        try {
            return new CdbFile(WasmCameraDatabase.of_json(json));
        }
        catch (e) {
            return null;
        }
    }
    to_json() {
        return this.cdb.to_json(true);
    }
    num_bodies() {
        return this.cdb.num_bodies();
    }
    num_lenses() {
        return this.cdb.num_lenses();
    }
    body_name(n) {
        return this.cdb.body_name(n);
    }
    lens_name(n) {
        return this.cdb.lens_name(n);
    }
}
export class ProjectFile extends BaseFile {
    constructor(project) {
        super();
        this.file_kind = FileKind.Project;
        this.project = project;
    }
    static of_json(json) {
        try {
            return new ProjectFile(WasmProject.of_json(json));
        }
        catch (e) {
            return null;
        }
    }
    to_json() {
        return this.project.to_json(true);
    }
}
export class PmsFile extends BaseFile {
    constructor(num_points) {
        super();
        this.file_kind = FileKind.Pms;
        this.num_points = num_points;
    }
    static of_json(json) {
        try {
            return new PmsFile(WasmPointMappingSet.try_json(json));
        }
        catch (e) {
            return null;
        }
    }
    npoints() {
        return this.num_points;
    }
}
export class NpsFile extends BaseFile {
    constructor(nps) {
        super();
        this.file_kind = FileKind.Nps;
        this.nps = nps;
    }
    static of_json(json) {
        try {
            const nps = new WasmNamedPointSet();
            nps.read_json(json);
            return new NpsFile(nps);
        }
        catch (e) {
            return null;
        }
    }
    num_points() {
        return this.nps.num_points();
    }
}
export class CipFile extends BaseFile {
    constructor(cip) {
        super();
        this.file_kind = FileKind.Cip;
        this.cip = cip;
    }
    static of_json(json) {
        try {
            return new CipFile(WasmCipDesc.try_json(json));
        }
        catch (e) {
            return null;
        }
    }
    image() {
        return this.cip.image;
    }
    camera_body() {
        return this.cip.camera_body;
    }
    camera_lens() {
        return this.cip.camera_lens;
    }
    num_mappings() {
        return this.cip.num_mappings;
    }
}
