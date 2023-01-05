#version 330 core
in vec3 color;
void main() {
  //   gl_FragColor = vec4(0.5, 0.5, 0.5, 1);
  gl_FragColor = vec4(color + 0.5, 1);
}