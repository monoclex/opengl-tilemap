# Fast OpenGL Tilemap Implementation

This is an impementation of an OpenGL tilemap as a single quad. The code is
lovingly documented, so you should be able to figure out what it's doing.

There is a [brief but accompanying blog post here][blog]

## Code Structure

```
/assets
  /tilemap.png -- The image of the tilemap
/src
  /main.rs              -- Main source code entry
  /vertex_shader.vert   -- Vertex shader source
  /fragment_shader.frag -- Fragment shader source
/Cargo.lock -- Rust stuff
/Cargo.toml -- Rust stuff
/.gitignore -- Git stuff
```

Dive in at `/src/main.rs`, then read the vertex and fragment shaders. Good luck!

[blog]: sirjosh3917.com/posts/implementing-fast-opengl-tilemap
