import { WasmProject } from "../pkg/photogram_wasm.js";

import { Logger } from "./log.js";
import { FileKind, ProjectFile } from "./file_kind.js";
import * as file_set from "./file_set.js";
import * as utils from "./utils.js";

interface Projects {
  filenames(): string[];
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
}

class LocalProjects implements Projects {
  files: file_set.FileSet;
  constructor(files: file_set.FileSet) {
    this.files = files;
  }

  filenames(): string[] {
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
}

class ServerProjects implements Projects {
  project_names: string[];
  constructor() {
    this.project_names = [];
    this.get_projects();
  }

  filenames(): string[] {
    return this.project_names;
  }

  filename_of_locator(locator: string): string | null {
    if (!locator.startsWith("server:")) {
      return null;
    }
    const filename = locator.slice(7);
    return filename;
  }

  load_project(
    filename: string,
    callback: (project: WasmProject) => void,
    error_callback: (error: string) => void,
  ): void {
    fetch("/project/" + filename + "?load")
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
    /*      this.files.save_file(
        "proj",
        "server_bkp_" + locator[1],
        project.to_json(true),
      );
      window.log.add_log(
        5,
        "project",
        "save",
        `Saved local backup to project server_bkp_${locator[1]}`,
      );
      */
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
    fetch("/project/" + filename + "?save", put_data).then((response) => {
      if (!response.ok) {
        error_callback(
          `Failed to save server project ${name}: ${response.status}`,
        );
      } else {
        callback();
      }
    });
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

  server_projects_json(json: Object) {
    if (utils.is_array(json)) {
      for (const name of json as Array<any>) {
        if (utils.is_string(name)) {
          this.project_names.push(name);
        }
      }
    }
  }
}

class ProjectKinds {
  local: LocalProjects;
  server: ServerProjects;
  constructor(files: file_set.FileSet) {
    this.local = new LocalProjects(files);
    this.server = new ServerProjects();
  }

  decode_locator(locator: string): [Projects, string] | null {
    const local = this.local.filename_of_locator(locator);
    console.log(locator, local);
    if (local !== null) {
      return [this.local, local];
    } else {
      const server = this.server.filename_of_locator(locator);
      if (server !== null) {
        return [this.server, server];
      }
    }
    return null;
  }

  local_filename(): string[] {
    return this.local.filenames();
  }
  server_filename(): string[] {
    return this.server.filenames();
  }

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
      /*       this.files.save_file(
        "proj",
        "server_bkp_" + locator[1],
        project.to_json(true),
      );
      window.log.add_log(
        5,
        "project",
        "save",
        `Saved local backup to project server_bkp_${locator[1]}`,
      );
 */
    }
  }
}

export class ProjectSet {
  files: file_set.FileSet;
  callback: () => void;
  projects: ProjectKinds;
  log: Logger;

  constructor(log: Logger, files: file_set.FileSet, callback: () => void) {
    this.files = files;
    this.log = log;
    this.callback = callback;
    this.projects = new ProjectKinds(files);
  }

  /**
   * Load the project from its locator (local or server) as a Json String -
   * invoking the callback on the data when loaded
   */
  load_project(
    locator: string,
    callback: (project: WasmProject) => void,
    error_callback: (error: string) => void,
  ) {
    this.projects.load_project(locator, callback, error_callback);
  }

  save_project(
    locator: string,
    project: WasmProject,
    callback: () => void,
    error_callback: (error: string) => void,
  ): void {
    this.projects.save_project(locator, project, callback, error_callback);
  }
}

/*
update_thumbnails() {
    const me = this;
    const i = document.getElementById("thumbnails");
    if (i && this.server_project) {
        html.clear(i);
        for (const n in this.server_project.thumbnails) {
            if (this.server_project.thumbnails[n]) {
                const a = html.add_ele(i, "a");
                a.addEventListener('click', function(e) {me.select_cip_of_project(n);});
                const img = html.add_ele(a, "img");
                img.src = URL.createObjectURL(this.server_project.thumbnails[n]);
            }
        }
    }
}
*/
