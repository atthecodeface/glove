import { WebglShaderSrc } from "./web_gl";

export class ImageShader implements WebglShaderSrc {
  id: string = "image";
  extra_uniforms = [];

  vertex: string = `#version 300 es
  uniform mat4 projection;
  uniform mat4 view;
  uniform mat4 model;

  // These are implicit
  // in highp int gl_VertexID;
  // in highp int gl_InstanceID;
  // out highp vec4 gl_Position;
  // out highp float gl_PointSize;

  in vec3 coord;
  in vec2 texcoord_in;

  out vec2 vTextureCoord;
  void main() {

    vTextureCoord = texcoord_in;
    gl_Position = model * vec4(coord, 1);
    }
`;

  fragment: string = `#version 300 es
precision mediump float;
uniform vec4 color;
uniform sampler2D uSampler;

in vec2 vTextureCoord;

out vec4 FragColor; // must be the only output declaration; is not implicit!

void main() {
  FragColor = texture(uSampler, vTextureCoord);
}
`;
}
