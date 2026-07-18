import { WebglShaderSrc } from "./web_gl";

const star_color_and_point_size = `
  // A polynomial fitting 4bit to red has a reasonable polynomial of red = 5.8x + 256x^2 (clamp to 255)
  // A polynomial fitting 4bit to green has a reasonable polynomial of green = 18x + 167x^2 (clamp to 255)
  // A polynomial fitting 4bit to blue has a reasonable polynomial of blue = 111 + 16x
  float red = clamp(t_f * (5.8/255.0 + 255.0/255.0*t_f),0.0,1.0);
  float green = clamp(t_f * (18.0/255.0 + 167.0/255.0*t_f),0.0,1.0);
  float blue = clamp(t_f * 16.0/255.0 + 111.0/255.0,0.0,1.0);

  float brightness = clamp(1.0 - m_f/16.0, 0.5, 1.0);
  star_color = vec3(brightness*red, brightness*green, brightness*blue);
  gl_PointSize = clamp(4.0 - 0.5 * m_f, 1.0, 4.0);
`;

export class StarCalibrationShader implements WebglShaderSrc {
  id: string = "star_calibration";
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

  in vec4 star;

  out vec3 star_color;
  void main() {

    float m_f = star.z;
    float t_f = star.w;

    gl_Position = projection * view * model * vec4(star.x, star.y, 0, 1);

    // Calculate 'star_color' and 'gl_PointSize'
    ${star_color_and_point_size}
    gl_PointSize = 4.0;
  }
`;

  fragment: string = `#version 300 es
  precision mediump float;
  in vec3 star_color;
  uniform vec4 color;

  out vec4 FragColor; // must be the only output declaration; is not implicit!

  // These are implicit
  // in highp vec4 gl_FragCoord;
  // in bool gl_FrontFacing;
  // out highp float gl_FragDepth;
  // in mediump vec2 gl_PointCoord;

  void main() {
  FragColor.r = color.r * star_color.r;
  FragColor.g = color.g * star_color.g;
  FragColor.b = color.b * star_color.b;
  FragColor.a = color.a;

  }
  `;
}

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
    gl_Position = projection * view * model * vec4(coord, 1);
    }
`;

  fragment: string = `#version 300 es
precision mediump float;
uniform vec4 color;
uniform sampler2D uSampler;

in vec2 vTextureCoord;

out vec4 FragColor; // must be the only output declaration; is not implicit!

void main() {
  vec4 img_color = texture(uSampler, vTextureCoord);
  FragColor.r = color.r * img_color.r;
  FragColor.g = color.g * img_color.g;
  FragColor.b = color.b * img_color.b;
  FragColor.a = color.a;
}
`;
}

/** Shader for grid lines over an image, and other effects, using instanced drawing
 *
 * Note that the maximum line width in WebGL2 implementations is usually 1, so this cannot generate thicker lines
 *
 */
export class ImageOverlayShader implements WebglShaderSrc {
  id: string = "image_overlay";
  extra_uniforms = ["args"];

  vertex: string = `#version 300 es
  uniform mat4 projection;
  uniform mat4 view;
  uniform mat4 model;
  uniform vec4 args;

  // implicit in highp int gl_InstanceID;

  in vec2 position;

  out vec3 line_color;
  void main() {
    float dx = args.x * (float(gl_InstanceID) + args.z);
    float dy = args.y * (float(gl_InstanceID) + args.z);
    float brightness = args.w;

    vec4 pos = vec4(position.x + dx, position.y + dy, 0, 1);
    if ((pos.x > 1.0) || (pos.x < -1.0) || (pos.y < -1.0) || (pos.y > 1.0)) {
        pos.z = 2.0;
    }
    gl_Position = projection * view * model * pos;
    line_color = vec3(brightness, brightness, brightness);
  }
`;

  fragment: string = `#version 300 es
  precision mediump float;
  in vec3 line_color;
  uniform vec4 color;

  out vec4 FragColor;
  void main() {
  FragColor.r = color.r * line_color.r;
  FragColor.g = color.g * line_color.g;
  FragColor.b = color.b * line_color.b;
  FragColor.a = color.a;

  }
  `;
}
