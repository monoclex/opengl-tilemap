#version 400

// the bottom-left corner of the triangle
flat in vec2 origin;

// the position of the pixel the fragment shader is being ran on
smooth in vec2 fragPosition;

out vec4 color;

// the `tilemap` - this is the image that holds the images of the tiles we're
// drawing
uniform sampler2D tilemap;

// the `indices` worldmap - this is an image, where every pixel corresponds to a
// position in the tilemap. by doing so, we can store world data
uniform sampler2D indices;

void main() {
  // our `tilemap` is a 2x2 tilemap
  vec2 tileSize = vec2(2.0, 2.0);

  // the size of our map is `1000` by `1000`. we could easily make this bigger
  // or smaller to effectively downsample the map, because of how the math below
  // works out
  vec2 mapSize = vec2(1000.0, 1000.0);

  // first, get the relative position
  //
  // so, note that the `fragPosition` is the position this fragment shader is
  // executing for, and that `origin` is the bottom left corner. we can
  // visualize this like so:
  //
  //     +
  //    /|
  //   /P| P = ( 0.25,  0.0)
  //  /  |
  // O---+ O = (-0.5 , -0.5)
  //
  // |---|
  //   L   L : 1.0
  //
  // because our origin and position could be placed anywhere, we want to get a
  // coordinate that is within and relative to the square. to do this, we
  // subtract the fragPosition P from the origin O to obtain a "normalized"
  // vector.
  //
  // P - O = (0.75, 0.5)
  //
  // however, this position - even though it is relative within the square - is
  // not normalized. for example, for a triangle of width 3 and an origin (0,
  // 0), there exists a P such that P - O = (3, 3) because the triangle is of
  // size 3. we do not want this, so we normalize the vector by dividing it by
  // L, the length of the triangle to obtain R:
  //
  // R = (P - O) / L = (0.75, 0.5)
  //
  // in the above case, it does not make a difference. since we know that
  // the size of our vertices will be spread `1` apart, we can hardcode this
  // here. (TODO: somehow get the total size of the triangle?)
  //
  // now that we have R, we have a coordinate such that x and y position ranges
  // from 0.0 to 1.0 inclusive. this makes it easier to map it to other places.
  vec2 quadSize = vec2(1.0, 1.0);
  vec2 relativePos = (fragPosition - origin) / quadSize;

  // now what we will do is index into the worldmap to get the value of the
  // block at this position. effectively, what we are doing can be visualized
  // below:
  //
  // +---+---+---+---+
  // |0,0|0,0|0,0|0,0|
  // +---+---+---+---+
  // |0,0|1,0|1,0|0,0|
  // +---+---+-R-+---+
  // |0,0|1,0|1,0|0,0|
  // +---+---+---+---+
  // |0,0|0,0|0,0|0,0|
  // +---+---+---+---+
  //
  // the above represents an image, `indices` (the world map). every open cell
  // is an RGBA value, with only the red and green values visualized as R,G.
  // then, we sample the pixel at `R` within this texture.
  //
  // the (R, G) value tells us the position on the tilemap that we want our
  // block at - so it would be bad if this pixel was linearly interpolated.
  // thus, we tell OpenGL to use nearest neighbor sampling. that way, it will
  // just take the value of one of the cells `R` is nearest to - in our case, it
  // would probably take `1,0`.
  //
  // the sampled coordinate, S
  // S = `1,0`
  //
  // now when we sample a coordinate, we actually get RGBA colors from 0.0
  // to 1.0, rather than from 0.0 to 255.0, so we want to multiply by 255.0 to
  // cancel it out.
  //
  // S = (1.0 / 255.0, 0.0 / 255.0)
  // I = (1.0 / 255.0, 0.0 / 255.0) * 255.0 = (1.0, 0.0)
  //
  // now we have an index in whole numbers that we can use
  vec2 index = texture(indices, relativePos).xy * 255.0;

  // now, for a refresher for what we have:
  //
  // R : a coordinate from 0.0 to 1.0 that represents where in the square we're
  // shading
  //
  // I : an index into the texture atlas, whose x and y values represent
  // somewhere in the texture atlas we want to sample
  //
  // now our goal is to get the color of the block we're sampling, and set that
  // to the color of this pixel. we already have the index of the texture we
  // want, but now we need the *offset* of this pixel - how far away we are from
  // the origin of the block we sampled.
  //
  // first, we need to get the "origin of the block". to do this, first we'll
  // interpret our relative position (from 0.0 to 1.0) onto a scale as big as
  // the entire map (4x4), like so:
  //
  // let R = (0.8, 0.6)
  //     M = R * mapSize
  //
  // +---+
  // |  R|
  // +---+
  //
  // -> transforms into ->
  //
  // +---+---+---+---+
  // |   |   |   |   |
  // +---+---+---+---+
  // |   |   |   |M  | M = (3.2, 2.4)
  // +---+---+---+---+
  // |   |   |   |   |
  // +---+---+---+---+
  // |   |   |   |   |
  // +---+---+---+---+
  vec2 mapPos = relativePos * mapSize;

  // now, to get the block origin coordinate ("origin of the block"), O_b, we
  // can simply floor the vector to get the exact origin coordinate
  //
  // O_b = floor(M) = (3, 2)
  vec2 base = floor(mapPos);

  // then, to get the offset between the block origin coordinate and the
  // relative map position, we subtract the two
  //
  // +-----+
  // | M   | M   = (3.2, 2.4)
  // |     |
  // O-----+ O_b = (  3,   2)
  //
  // delta = M - O_b = (0.2, 0.4)
  vec2 offset = mapPos - base;

  // finally, we can add the index and offset together to get the position to
  // sample the tile at:
  //
  // S = I          + delta
  //   = (1  , 0  ) + (0.2, 0.4)
  //   = (1.2, 0.4)
  //
  // however, `S` is not normalized  within the tilemap. a 3x3 tilemap would be
  // as follows:
  //
  //  1           1
  // 0+---+---+---+1
  //  |   |   |   |
  //  +---+---+---+
  //  |   |   |   |
  //  +---+---+---+
  //  |   |   |   |
  // 0+---+---+---+1
  //  0           0
  //
  // `S` would fall out of range of the tilemap - so we must put it into range
  // of the tilemap. by dividing S by the tileSize, we normalize it into that
  // range.
  //
  // S_n = S           / (3, 3)
  //     = (1.2, 0.4 ) / (3, 3)
  //     = (0.4, 0.13)
  //
  //  1           1
  // 0+---+---+---+1
  //  |   |   |   |
  //  +---+---+---+
  //  |   |   |   |
  //  +---+---+---+
  //  |   |X  |   |  X = S_n = (0.4, 0.13)
  // 0+---+---+---+1
  //  0           0
  //
  // and thus, we sample the texture!
  color = texture(tilemap, (index + offset) / tileSize);
}