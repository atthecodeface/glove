import * as utils from "./utils.js";
import { UnknownFile } from "./file_kind.js";
export class FileSet {
    constructor(storage, file_list_callback) {
        this.filenames_by_kind = new Map();
        this.storage = storage;
        this.file_list_callback = file_list_callback;
    }
    get_file_list() {
        this.storage.request_get_file_list(this.file_list_received.bind(this));
    }
    has_file_of_kind(kind, filename) {
        const filenames = this.filenames_by_kind.get(kind);
        if (filenames === undefined) {
            return false;
        }
        const found = filenames.findIndex((f) => {
            return utils.strcmp(filename, f) == 0;
        });
        return found >= 0;
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
    save_file(filename, file) {
        this.storage.save_file(filename, file.to_json());
        this.file_list_received(true);
    }
}
