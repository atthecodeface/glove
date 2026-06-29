import {
  WasmProject,
  WasmCipDesc,
  WasmCameraDatabase,
  WasmPointMappingSet,
  WasmNamedPointSet,
} from "../pkg/photogram_wasm.js";

import * as utils from "./utils.js";

export enum FileKind {
  Unknown,
  Cdb,
  Project,
  Pms,
  Nps,
  Cip,
}

export interface File {
  kind(): FileKind;
  of_obj(obj: Object): File | null;
  to_json(): string;
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
  to_json(): string {
    throw new Error(
      "No means to generate Json for " + this.file_kind.toString(),
    );
  }
}

export class UnknownFile extends BaseFile {
  obj: Object;
  constructor(obj: Object) {
    super();
    this.obj = obj;
  }
  static find_data_type(json: string): File {
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
    const obj = utils.parse_json(json) as any;
    return new UnknownFile(obj);
  }
}

export class CdbFile extends BaseFile {
  override file_kind: FileKind = FileKind.Cdb;
  cdb: WasmCameraDatabase;
  constructor(cdb: WasmCameraDatabase) {
    super();
    this.cdb = cdb;
  }
  static of_json(json: string): CdbFile | null {
    try {
      return new CdbFile(WasmCameraDatabase.of_json(json));
    } catch (e) {
      return null;
    }
  }
  override to_json(): string {
    return this.cdb.to_json(true);
  }
  num_bodies(): number {
    return this.cdb.num_bodies();
  }
  num_lenses(): number {
    return this.cdb.num_lenses();
  }
  body_name(n: number): string | undefined {
    return this.cdb.body_name(n);
  }
  lens_name(n: number): string | undefined {
    return this.cdb.lens_name(n);
  }
}

export class ProjectFile extends BaseFile {
  override file_kind: FileKind = FileKind.Project;
  project: WasmProject;
  constructor(project: WasmProject) {
    super();
    this.project = project;
  }
  static of_json(json: string): ProjectFile | null {
    try {
      return new ProjectFile(WasmProject.of_json(json));
    } catch (e) {
      return null;
    }
  }
  override to_json(): string {
    return this.project.to_json(true);
  }
}

export class PmsFile extends BaseFile {
  override file_kind: FileKind = FileKind.Pms;
  num_points: number;
  constructor(num_points: number) {
    super();
    this.num_points = num_points;
  }
  static of_json(json: string): PmsFile | null {
    try {
      return new PmsFile(WasmPointMappingSet.try_json(json));
    } catch (e) {
      return null;
    }
  }
  npoints(): number {
    return this.num_points;
  }
}

export class NpsFile extends BaseFile {
  override file_kind: FileKind = FileKind.Nps;
  nps: WasmNamedPointSet;
  constructor(nps: WasmNamedPointSet) {
    super();
    this.nps = nps;
  }
  static of_json(json: string): NpsFile | null {
    try {
      const nps = new WasmNamedPointSet();
      nps.read_json(json);
      return new NpsFile(nps);
    } catch (e) {
      return null;
    }
  }
  num_points(): number {
    return this.nps.num_points();
  }
}

export class CipFile extends BaseFile {
  override file_kind: FileKind = FileKind.Cip;
  cip: WasmCipDesc;
  constructor(cip: WasmCipDesc) {
    super();
    this.cip = cip;
  }
  static of_json(json: string): CipFile | null {
    try {
      return new CipFile(WasmCipDesc.try_json(json));
    } catch (e) {
      return null;
    }
  }
  image(): string {
    return this.cip.image;
  }
  camera_body(): string {
    return this.cip.camera_body;
  }
  camera_lens(): string {
    return this.cip.camera_lens;
  }
  num_mappings(): number {
    return this.cip.num_mappings;
  }
}
