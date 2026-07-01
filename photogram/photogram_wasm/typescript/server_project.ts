//a Imports
import { Log } from "./log.js";
import * as utils from "./utils.js";

//a ServerProject
export class ServerProject {
  //fp constructor
  constructor(uri, project) {
    this.uri = uri;
    this.project = project;
    this.thumbnails = {};
    this.meshes = {};
    this.interestings = {};
  }

  //mp fetch_thumbnail
  async fetch_thumbnail(cip, width) {
    this.thumbnails[cip] = null;
    console.log(`fetch(${this.uri}?thumbnail&cip=${cip}&width=${width})`);
    return fetch(`${this.uri}?thumbnail&cip=${cip}&width=${width}`)
      .then((response) => {
        if (!response.ok) {
          throw new Error(`Failed to fetch thumbnail: ${response.status}`);
        }
        return response.arrayBuffer();
      })
      .then((data) => {
        const blob = new Blob([data], { type: "image/jpeg" });
        return blob;
      });
  }

  //mp fetch_mesh
  async fetch_mesh(cip) {
    this.meshes[cip] = [];
    console.log(`fetch(${this.uri}?mesh&cip=${cip})`);
    return fetch(`${this.uri}?mesh&cip=${cip}`).then((response) => {
      if (!response.ok) {
        throw new Error(`Failed to fetch mesh: ${response.status}`);
      }
      return response.json();
    });
  }

  //mp fetch_interesting
  async fetch_interesting(cip) {
    this.interestings[cip] = [];
    console.log(`fetch(${this.uri}?interesting&cip=${cip})`);
    return fetch(`${this.uri}?interesting&cip=${cip}`).then((response) => {
      if (!response.ok) {
        throw new Error(
          `Failed to fetch interesting points: ${response.status}`,
        );
      }
      return response.json();
    });
  }

  //mp fetch_thumbnails
  fetch_thumbnails(width, callback) {
    this.thumbnails = {};
    const me = this;
    let promises = [];
    for (let i = 0; i < this.project.ncips(); i++) {
      const cip_name = this.project.cip_name(i);
      promises.push(
        this.fetch_thumbnail(cip_name, width)
          .then((blob) => {
            me.thumbnails[cip_name] = blob;
          })
          .catch((err) => console.error(`Fetch problem: ${err.message}`)),
      );
    }
    Promise.all(promises).then(() => {
      callback(me);
    });
  }

  //mp issue_fetch_interestings
  issue_fetch_interestings(cip, callback) {
    if (!cip) {
      return;
    }
    if (this.interestings[cip]) {
      callback(this);
      return;
    }
    const me = this;
    let promises = [];
    promises.push(
      this.fetch_interesting(cip)
        .then((m) => {
          me.interestings[cip] = m;
        })
        .catch((err) => console.error(`Fetch problem: ${err.message}`)),
    );
    Promise.all(promises).then(() => {
      callback(me);
    });
  }

  //mp issue_fetch_mesh
  issue_fetch_mesh(cip, callback) {
    if (!cip) {
      return;
    }
    if (this.meshes[cip]) {
      callback(this);
      return;
    }
    const me = this;
    let promises = [];
    promises.push(
      this.fetch_mesh(cip)
        .then((m) => {
          me.meshes[cip] = m;
        })
        .catch((err) => console.error(`Fetch problem: ${err.message}`)),
    );
    Promise.all(promises).then(() => {
      callback(me);
    });
  }

  //mp get_mesh
  get_mesh(cip) {
    return this.meshes[cip];
  }

  //mp get_interestings
  get_interestings(cip) {
    return this.interestings[cip];
  }

  //mp clear_meshes
  clear_meshes() {
    this.meshes = {};
  }

  //mp image_uri
  image_uri(cip) {
    return `${this.uri}?image&cip=${cip}`;
  }

  //zz All Done
}
