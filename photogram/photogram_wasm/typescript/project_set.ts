import { WasmProject } from "../pkg/photogram_wasm.js";

import { Logger } from "./log.js";
import { FileKind, ProjectFile } from "./file_kind.js";
import * as file_set from "./file_set.js";
import * as utils from "./utils.js";

interface ProjectSource {
  project_filenames(): string[];
  filename_of_locator(locator: string): string | null;
  load_project(
    filename: string,
    callback: (project: WasmProject) => void,
    error_callback: (error: string) => void,
  ): void;

  save_project(
    filename: string,
    project: WasmProject,
    callback: () => void,
    error_callback: (error: string) => void,
  ): void;

  promise_fetch_thumbnail(
    filename: string,
    cip_name: string,
    width: number,
  ): Promise<Blob> | null;

  promise_fetch_image(filename: string, cip_name: string): Promise<Blob> | null;
}

class LocalProjects implements ProjectSource {
  files: file_set.FileSet;
  constructor(files: file_set.FileSet) {
    this.files = files;
  }

  project_filenames(): string[] {
    return this.files.files_of_kind(FileKind.Project);
  }

  filename_of_locator(locator: string): string | null {
    if (!locator.startsWith("local:")) {
      return null;
    }
    const filename = locator.slice(6);
    if (!this.files.has_file_of_kind(FileKind.Project, filename)) {
      return null;
    }
    return filename;
  }

  load_project(
    filename: string,
    callback: (project: WasmProject) => void,
    error_callback: (error: string) => void,
  ): void {
    const p = this.files.load_file_as_obj(filename, FileKind.Project);
    if (p === null) {
      error_callback(`File ${filename} not found in browser storage`);
      return;
    }
    const project = p as ProjectFile;
    callback(project.project);
  }

  save_project(
    filename: string,
    project: WasmProject,
    callback: () => void,
    _error_callback: (error: string) => void,
  ): void {
    const p = new ProjectFile(project);
    this.files.save_file(filename, p);
    callback();
  }

  promise_fetch_thumbnail(
    _filename: string,
    _cip_name: string,
    _width: number,
  ): Promise<Blob> | null {
    return null;
  }

  promise_fetch_image(
    _filename: string,
    _cip_name: string,
  ): Promise<Blob> | null {
    return null;
  }
}

class ServerProjects implements ProjectSource {
  project_names: string[];
  constructor() {
    this.project_names = [];
    this.get_projects();
  }

  project_filenames(): string[] {
    return this.project_names;
  }

  filename_of_locator(locator: string): string | null {
    if (!locator.startsWith("server:")) {
      return null;
    }
    const filename = locator.slice(7);
    return filename;
  }

  fetch_for_project(
    project: string,
    action: string,
    init?: RequestInit,
  ): Promise<Response> {
    return fetch("/project/" + project + "?" + action, init);
  }

  get_projects() {
    this.project_names = [];
    fetch("/project?list")
      .then((response) => {
        if (!response.ok) {
          throw new Error(
            `Failed to fetch server projects: ${response.status}`,
          );
        }
        return response.json();
      })
      .then(this.server_projects_json.bind(this))
      .catch((err) => console.error(`Fetch problem: ${err.message}`));
  }

  load_project(
    filename: string,
    callback: (project: WasmProject) => void,
    error_callback: (error: string) => void,
  ): void {
    this.fetch_for_project(filename, "load")
      .then((response) => {
        if (response.ok) {
          return response.text();
        } else {
          error_callback(
            `Failed to fetch server project ${name}: ${response.status}`,
          );
          return null;
        }
      })
      .then((json) => {
        if (json !== null) {
          try {
            const project = WasmProject.of_json(json);
            callback(project);
          } catch (e) {
            error_callback((e as any).toString());
          }
        }
      });
  }

  save_project(
    filename: string,
    project: WasmProject,
    callback: () => void,
    error_callback: (error: string) => void,
  ): void {
    const put_data: RequestInit = {
      cache: "no-store",
      credentials: "same-origin",
      headers: {
        "Content-Type": "application/json",
      },
      method: "PUT",
      mode: "same-origin", // cors?
      body: project.to_json(false),
    };
    this.fetch_for_project(filename, "save", put_data).then((response) => {
      if (!response.ok) {
        error_callback(
          `Failed to save server project ${name}: ${response.status}`,
        );
      } else {
        callback();
      }
    });
  }

  server_projects_json(json: Object) {
    if (utils.is_array(json)) {
      for (const name of json as Array<any>) {
        if (utils.is_string(name)) {
          this.project_names.push(name);
        }
      }
    }
  }

  promise_fetch_thumbnail(
    filename: string,
    cip_name: string,
    width: number,
  ): Promise<Blob> | null {
    return this.fetch_for_project(
      filename,
      `thumbnail&cip=${cip_name}&width=${width}`,
    )
      .then((response) => {
        if (!response.ok) {
          throw new Error(
            `Failed to fetch thumbnail for ${filename} ${cip_name}: ${response.status}`,
          );
        }
        return response.arrayBuffer();
      })
      .then((data) => {
        return new Blob([data], { type: "image/jpeg" });
      });
  }

  promise_fetch_image(
    filename: string,
    cip_name: string,
  ): Promise<Blob> | null {
    return this.fetch_for_project(filename, `image&cip=${cip_name}`)
      .then((response) => {
        if (!response.ok) {
          throw new Error(
            `Failed to fetch image for ${filename} ${cip_name}: ${response.status}`,
          );
        }
        return response.arrayBuffer();
      })
      .then((data) => {
        return new Blob([data], { type: "image/jpeg" });
      });
  }
}

/** A set of in-browser (local) and server project sources
 *
 * Each of the project sources must supply the 'Projects' interface
 *
 * Currently a single server is supported; this could be more than one
 */
export class ProjectSet {
  project_sources: ProjectSource[];
  files: file_set.FileSet;
  callback: () => void;
  log: Logger;

  constructor(log: Logger, files: file_set.FileSet, callback: () => void) {
    this.files = files;
    this.log = log;
    this.callback = callback;
    this.project_sources = [];
    this.project_sources.push(new LocalProjects(files));
    this.project_sources.push(new ServerProjects());
  }

  decode_locator(locator: string): [ProjectSource, string] | null {
    for (const p of this.project_sources) {
      const f = p.filename_of_locator(locator);
      if (f !== null) {
        return [p, f];
      }
    }
    return null;
  }

  /**
   * Load the project from its locator (local or server) as a Json String -
   * invoking the callback on the data when loaded
   */
  load_project(
    locator: string,
    callback: (project: WasmProject) => void,
    error_callback: (error: string) => void,
  ): void {
    const decode = this.decode_locator(locator);
    if (decode === null) {
      error_callback(
        `Unknown source (not local or server) in filename ${locator}`,
      );
    } else {
      decode[0].load_project(decode[1], callback, error_callback);
    }
  }

  save_project(
    locator: string,
    project: WasmProject,
    callback: () => void,
    error_callback: (error: string) => void,
  ): void {
    const decode = this.decode_locator(locator);
    if (decode === null) {
      error_callback(
        `Unknown source (not local or server) in filename ${locator}`,
      );
    } else {
      decode[0].save_project(decode[1], project, callback, error_callback);
    }
  }

  promise_fetch_thumbnail(
    locator: string,
    cip_name: string,
    width: number,
  ): Promise<Blob> | null {
    const decode = this.decode_locator(locator);
    if (decode === null) {
      return null;
    } else {
      return decode[0].promise_fetch_thumbnail(decode[1], cip_name, width);
    }
  }

  promise_fetch_image(locator: string, cip_name: string): Promise<Blob> | null {
    const decode = this.decode_locator(locator);
    if (decode === null) {
      return null;
    } else {
      return decode[0].promise_fetch_image(decode[1], cip_name);
    }
  }
}
