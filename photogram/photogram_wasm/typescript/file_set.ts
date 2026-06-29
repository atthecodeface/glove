import * as storage from "./storage.js";
import * as utils from "./utils.js";
import { File, UnknownFile, FileKind } from "./file_kind.js";

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

  has_file_of_kind(kind: FileKind, filename: string): boolean {
    const filenames = this.filenames_by_kind.get(kind);
    if (filenames === undefined) {
      return false;
    }
    const found = filenames.findIndex((f) => {
      return utils.strcmp(filename, f) == 0;
    });

    return found >= 0;
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

  save_file(filename: string, file: File) {
    this.storage.save_file(filename, file.to_json());
  }
}
