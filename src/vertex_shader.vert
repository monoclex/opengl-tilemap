#version 400

// the position of this vertex
in vec2 position;

// the `fragPosition` vertex
//
// the `smooth` modifier dictates that this value will be *linearly
// interpolated* for the fragment shader. because 3 vertices make a triangle,
// the linear interpolation will happen between all three fragment values. in
// essence, this will allow the fragment shader to get the position of the
// specific pixel it's coloring in.
smooth out vec2 fragPosition;

// the `origin` vertex
//
// the `flat` modifier dictates that the fragment shader, when it runs across
// every pixel, will get a constant value. unlike `smooth` which lineaarly
// interpolates for three vertices, `flat` does not linearly interpolate and
// instead fixes the value to the value given by one of the vertex shaders.
//
// when we draw the shape, we tell opengl to use the first vertex should be the
// "provoking vertex". this means that for the entire shape, the fragment
// shader will get the value provided by the provoking vertex only.
//
// in our case, we set the *first* vertex to be the provoking vertex (vertex
// `3`), which is the bottom-left corner of the square.
flat out vec2 origin;

// we use this for zoom (not explained here, as this isn't related to tilemap)
uniform float zoom;

void main() {
  // this has an effect for the provoking vertex only - it will set the `origin`
  // to the bottom left corner of the triangle, because the bottom left corner
  // of the triangle is the first vertex and we told opengl to set the provoking
  // vertex to the first vertex.
  origin = position;

  // this will get linearly interpolated across the entire fragment that is
  // being drawn
  fragPosition = position;

  gl_Position = vec4(position, 0.0, zoom);
}