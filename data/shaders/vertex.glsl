#version 330 core
in vec3 position;
out vec3 color;

// mat4 frustum() {
//   u_near = -u_near;
//   u_far = -u_far;
//   float e = u_near * 2, f = u_right - u_left, g = u_top - u_bottom,
//         h = u_far - u_near;
//   float a = (u_right + u_left) / f, b = (u_top + u_bottom) / g,
//         c = (u_far + u_near) / h, d = e * u_far / h;
//   return mat4(e / f, 0, a, 0, 0, e / g, b, 0, 0, s0, c, d, 0, 0, -1, 0);
// }

void main() {
  // gl_Position = frustum() * vec4(position, 1.0);
  gl_Position = vec4(position / 2, 3 - position.z);
  color = position / 2;
}