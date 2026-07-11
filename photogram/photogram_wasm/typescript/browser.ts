import { HtmlElement, Table } from "./html.js";
import * as file_kind from "./file_kind.js";
import * as file_set from "./file_set.js";
import { Logger } from "./log.js";

import { Application, ApplicationTab } from "./application.js";

export class Browser implements ApplicationTab {
  application: Application;
  file_set: file_set.FileSet;
  browser: HtmlElement;
  log: Logger;

  constructor(
    application: Application,
    log: Logger,
    file_set: file_set.FileSet,
    browser: HtmlElement,
  ) {
    this.application = application;
    this.log = log;
    this.file_set = file_set;
    this.browser = browser;
    application.add_tab(this, null);
  }

  tab_deselected(): void {}
  tab_selected(): void {
    this.repopulate();
  }

  tab_name(): string {
    return "browser";
  }
  tab_text(): string {
    return "Browser";
  }

  file_link(f: string): HtmlElement {
    // href is the file to fetch; donwload indicates it is to be downloaded to *that* filename
    const link = HtmlElement.new_ele("a", {}, [
      ["href", f],
      ["download", f],
    ]);

    link.ele.addEventListener("click", (event: PointerEvent) => {
      const link = event.target as HTMLAnchorElement;
      const data = this.file_set.load_file_as_str(f);
      link.href = "";
      if (data !== null) {
        link.href = "data:application/json," + encodeURIComponent(data) + "";
      }
    });
    link.add_content(f);
    return link;
  }

  repopulate() {
    this.browser.clear();
    this.browser.add_label("upload").add_content("Upload (Json)");
    this.browser.add_input_files(".json", true, this.upload_files.bind(this), {
      id: "upload",
    });
    this.add_table_of_cdb();
    this.add_table_of_projects();
    this.add_table_of_named_point_sets();
    this.add_table_of_cip();
  }

  add_table_of_cdb() {
    const heading = HtmlElement.new_ele("h1", {
      classes: "browser_ft_heading",
    });
    heading.add_content("Camera Database");

    const table = new Table({ classes: "cdb" });
    table.add_headings(["Filename", "Bodies", "Lenses"]);

    for (const filename of this.file_set.files_of_kind(
      file_kind.FileKind.Cdb,
    )) {
      let obj = this.file_set.load_file_as_obj(
        filename,
        file_kind.FileKind.Cdb,
      );
      if (obj === null) {
        continue;
      }
      let cdb = obj as file_kind.CdbFile;
      if (cdb === null) {
        continue;
      }

      var bodies_html = HtmlElement.new_ele("div");
      for (let i = 0; i < cdb.num_bodies(); i++) {
        bodies_html.add_span(cdb.body_name(i)!);
        bodies_html.add_ele("br");
      }

      var lenses_html = HtmlElement.new_ele("div");
      for (let i = 0; i < cdb.num_lenses(); i++) {
        lenses_html.add_span(cdb.lens_name(i)!);
        lenses_html.add_ele("br");
      }

      const link = this.file_link(filename);
      table.add_body([link, bodies_html, lenses_html]);
    }
    this.browser.add_content(heading);
    this.browser.add_content(table.as_html());
  }

  add_table_of_projects() {
    const heading = HtmlElement.new_ele("h1", {
      classes: "browser_ft_heading",
    });
    heading.add_content("Projects");

    const table = new Table({ classes: "proj" });
    table.add_headings(["Filename", "Cdb", "Nps", "Number CIP"]);

    for (const filename of this.file_set.files_of_kind(
      file_kind.FileKind.Project,
    )) {
      let obj = this.file_set.load_file_as_obj(
        filename,
        file_kind.FileKind.Project,
      );
      if (obj === null) {
        continue;
      }
      let project = obj as file_kind.ProjectFile;
      if (project === null) {
        continue;
      }
      const link = this.file_link(filename);
      table.add_body([
        link,
        "project.project.cdb",
        "project.project.nps",
        project.project.ncips().toString(),
      ]);
    }
    this.browser.add_content(heading);
    this.browser.add_content(table.as_html());
  }

  add_table_of_named_point_sets() {
    const heading = HtmlElement.new_ele("h1", {
      classes: "browser_ft_heading",
    });
    heading.add_content("Named point sets");

    const table = new Table({ classes: "nps" });
    table.add_headings(["Filename", "Number of points"]);

    for (const filename of this.file_set.files_of_kind(
      file_kind.FileKind.Nps,
    )) {
      let obj = this.file_set.load_file_as_obj(
        filename,
        file_kind.FileKind.Nps,
      );
      if (obj === null) {
        continue;
      }
      let nps = obj as file_kind.NpsFile;
      const link = this.file_link(filename);
      table.add_body([link, nps.num_points().toString()]);
    }
    this.browser.add_content(heading);
    this.browser.add_content(table.as_html());
  }

  add_table_of_cip() {
    const heading = HtmlElement.new_ele("h1", {
      classes: "browser_ft_heading",
    });
    heading.add_content("Camera/Image/Point mapping set");

    const table = new Table({ classes: "cip" });
    table.add_headings([
      "Filename",
      "Image",
      "Body",
      "Lens",
      "Number point-mappings",
    ]);

    for (const filename of this.file_set.files_of_kind(
      file_kind.FileKind.Cip,
    )) {
      let obj = this.file_set.load_file_as_obj(
        filename,
        file_kind.FileKind.Cip,
      );
      if (obj === null) {
        continue;
      }
      let cip = obj as file_kind.CipFile;

      const link = this.file_link(filename);
      table.add_body([
        link,
        cip.image(),
        cip.camera_body(),
        cip.camera_lens(),
        cip.num_mappings().toString(),
      ]);
    }
    this.browser.add_content(heading);
    this.browser.add_content(table.as_html());
  }

  /**
   * Upload files, invoked by a button from the HTML page itself
   *
   *  e.g. <input type="file" id="upload" name="upload" accept=".json" multiple>/>
   *
   * with a change event listener of  this.upload_files(<elemennt>.files);
   *
   * @param files
   */
  upload_files(files: FileList) {
    for (const file of files) {
      file.text().then((file_contents) => {
        this.upload_file_contents(file.name, file_contents);
      });
    }
  }

  upload_file_contents(filename: string, contents: string) {
    let file = file_kind.UnknownFile.find_data_type(contents);
    if (file.kind() !== file_kind.FileKind.Unknown) {
      this.log.info("upload", `Uploaded ${filename} of type ${file.kind()}`);
      this.file_set.save_file(filename, file);
      this.repopulate();
    } else {
      this.log.warning("upload", `Could not determine type of ${filename}`);
    }
  }
}
