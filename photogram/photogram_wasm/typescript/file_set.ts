import * as storage from "./storage.js";
import * as utils from "./utils.js";

export enum FileKind {
  Unknown,
  Cdb,
  Camera,
  Project,
  Pms,
  Nps,
  Cips,
}

export interface File {
  kind(): FileKind;
  of_obj(obj: Object): File | null;
}

export class BaseFile implements File {
  file_kind: FileKind = FileKind.Unknown;
  kind(): FileKind {
    return this.file_kind;
  }
  of_obj(obj: Object) {
    return new UnknownFile(obj);
  }
  obj_is(_obj: Object): boolean {
    return true;
  }
}

export class UnknownFile extends BaseFile {
  obj: Object;
  constructor(obj: Object) {
    super();
    this.obj = obj;
  }
  static find_data_type(json: string): File {
    const obj = utils.parse_json(json) as any;
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

interface Body {
  name: string;
}
interface Lens {
  name: string;
}
export class CdbFile extends BaseFile {
  override file_kind: FileKind = FileKind.Cdb;
  bodies: Array<Body>;
  lenses: Array<Lens>;
  constructor(obj: Object) {
    super();
    this.bodies = (obj as CdbFile).bodies!;
    this.lenses = (obj as CdbFile).lenses!;
  }
  static obj_is(obj: Object): boolean {
    return obj.hasOwnProperty("bodies") && obj.hasOwnProperty("lenses");
  }
}

export class CameraFile extends BaseFile {
  override file_kind: FileKind = FileKind.Camera;
  body: string;
  lens: string;
  constructor(obj: Object) {
    super();
    this.body = (obj as CameraFile).body;
    this.lens = (obj as CameraFile).lens;
  }
  static obj_is(obj: Object): boolean {
    return (
      obj.hasOwnProperty("body") &&
      obj.hasOwnProperty("lens") &&
      obj.hasOwnProperty("position") &&
      obj.hasOwnProperty("direction")
    );
  }
}
export class ProjectFile extends BaseFile {
  override file_kind: FileKind = FileKind.Project;
  cdb: string;
  nps: string;
  cips: CipsFile[];
  constructor(obj: Object) {
    super();
    this.cdb = "some cdb";
    this.nps = "some nps";
    this.cips = [];
    for (const c of (obj as ProjectFile).cips) {
      if (CipsFile.obj_is(c)) {
        this.cips.push(new CipsFile(c));
      }
    }
  }
  static obj_is(obj: Object): boolean {
    return (
      obj.hasOwnProperty("cdb") &&
      obj.hasOwnProperty("nps") &&
      obj.hasOwnProperty("cips")
    );
  }
}

export class PmsFile extends BaseFile {
  override file_kind: FileKind = FileKind.Pms;
  constructor(_obj: Object) {
    super();
  }
  static obj_is(obj: Object): boolean {
    if (!utils.is_array(obj)) {
      return false;
    }
    const obj_a = obj as Array<Object>;
    if (obj_a.length == 0) {
      return false;
    }
    if (!utils.is_array(obj_a[0]!)) {
      return false;
    }
    const obj_a2 = obj_a[0] as Array<Object>;
    if (obj_a2.length != 3) {
      return false;
    }
    return (
      utils.is_string(obj_a2[0]!) &&
      utils.is_array(obj_a2[1]!) &&
      utils.is_float(obj_a2[2]!)
    );
  }
}

export class NpsFile extends BaseFile {
  override file_kind: FileKind = FileKind.Nps;
  points: [string, string, string[]][];
  constructor(obj: Object) {
    super();
    this.points = [];
    for (const p of obj as [string, string, string[]][]) {
      this.points.push(p);
    }
  }
  static obj_is(obj: Object): boolean {
    if (!utils.is_array(obj)) {
      return false;
    }
    const obj_a = obj as Array<Object>;
    if (obj_a.length == 0) {
      return false;
    }
    if (!utils.is_array(obj_a[0]!)) {
      return false;
    }
    const obj_a2 = obj_a[0] as Array<Object>;
    if (obj_a2.length != 3) {
      return false;
    }
    return (
      utils.is_string(obj_a2[0]!) &&
      utils.is_string(obj_a2[1]!) &&
      utils.is_array(obj_a2[2]!)
    );
  }
}

export class CipsFile extends BaseFile {
  override file_kind: FileKind = FileKind.Cips;
  constructor(_obj: Object) {
    super();
  }
  static obj_is(obj: Object): boolean {
    if (!utils.is_array(obj)) {
      return false;
    }
    const obj_a = obj as Array<Object>;
    if (obj_a.length == 0) {
      return false;
    }
    if (!utils.is_array(obj_a[0]!)) {
      return false;
    }
    const obj_a2 = obj_a[0] as Array<Object>;
    if (obj_a2.length != 3) {
      return false;
    }
    return (
      utils.is_string(obj_a2[0]!) &&
      utils.is_string(obj_a2[1]!) &&
      utils.is_array(obj_a2[2]!)
    );
  }
}

export class FileSet {
  storage: storage.LocalStorage;
  filenames_by_kind: Map<FileKind, Array<string>>;
  file_list_callback: () => void;
  constructor(storage: storage.LocalStorage, file_list_callback: () => void) {
    this.filenames_by_kind = new Map();
    this.storage = storage;
    this.file_list_callback = file_list_callback;
  }
  get_file_list() {
    this.storage.request_get_file_list(this.file_list_received.bind(this));
  }
  files_of_kind(kind: FileKind): Array<string> {
    const filenames = this.filenames_by_kind.get(kind);
    if (filenames === undefined) {
      return [];
    } else {
      return filenames;
    }
  }
  file_list_received(success: boolean): void {
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
        this.filenames_by_kind.get(file.kind())!.push(filename);
      }
      this.file_list_callback();
    }
  }
  load_file_as_str(filename: string): string | null {
    return this.storage.load_file(filename);
  }
  load_file_as_obj(filename: string, kind: FileKind): File | null {
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
  save_file(filename: string, _file: File, contents: string) {
    this.storage.save_file(filename, contents);
  }
}
