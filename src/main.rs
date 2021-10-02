#![feature(array_map)]

use std::{
    fs::File,
    io::BufReader,
    sync::{atomic::AtomicUsize, Arc},
    time::Instant,
};

extern crate glium;

use glium::{
    draw_parameters::ProvokingVertex,
    glutin::{
        self,
        dpi::{PhysicalPosition, PhysicalSize},
        event::{DeviceEvent, Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
        ContextBuilder,
    },
    implement_vertex,
    texture::RawImage2d,
    uniform,
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
    Display, DrawParameters, Program, Surface, Texture2d, VertexBuffer,
};
use image::ImageFormat;

fn main() {
    // the windowing boilerplate will not be covered in depth
    // this is because the glium tutorial book already goes over it
    //
    // https://github.com/glium/glium/tree/master/book
    //
    // regardless, comments may still describe sections

    // window boilerplate
    let (event_loop, display) = setup_window_boilerplate();

    fn setup_window_boilerplate() -> (EventLoop<()>, Display) {
        let event_loop = EventLoop::new();
        let window_builder = WindowBuilder::new().with_inner_size(PhysicalSize::new(800, 600));
        let context_builder = ContextBuilder::new();
        let display = Display::new(window_builder, context_builder, &event_loop).unwrap();

        (event_loop, display)
    }

    // load the tilemap texture
    let tilemap = load_tilemap(&display);
    let tilemap = Texture2d::new(&display, tilemap).unwrap();

    fn load_tilemap(display: &Display) -> RawImage2d<u8> {
        // non-opengl part, just loading the image
        let bytes = include_bytes!("../assets/tilemap.png");
        let img = image::load_from_memory_with_format(bytes, ImageFormat::Png).unwrap();

        // we want the pixels as a series of (r, g, b) bytes, with each value being a `u8`
        let rgb8_img = img.to_rgb8();

        // now we read the raw png data into an image. however, we need to call
        // the `_reversed` method because the PNG data is not the way OpenGL
        // expects
        //
        // our image data is a bunch of bytes representing the pixels. however,
        // the PNG treats the first pixel as being in the upper left:
        //
        // [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]
        //  ^^^^^^^^ ^^^^^^^^ ^^^^^^^^ ^^^^^^^<<
        //  up left  up right btm left btm right  <- PNG interpretation
        //
        // OpenGL texture coordinates are different - the data starts getting
        // filled in "at the bottom"
        //
        // [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]
        //  ^^^^^^^^ ^^^^^^^^ ^^^^^^^^ ^^^^^^^<
        //  btm left btm rght up left  up right   <- OpenGL interpretation
        //
        // as a result, we need to call the `_reversed` method to vertically
        // flip the image. that means it'll flip the data around so that it
        // lines up with the PNG interpretation
        //
        // PNG interpretation:
        //
        // [0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3]
        //  ^^^^^^^^ ^^^^^^^^ ^^^^^^^^ ^^^^^^^<<
        //  up left  up right btm left btm right  <- PNG interpretation
        //
        // reversed image data interpretation:
        //
        //    width of img      width of img
        //  /--------------\  /--------------\
        // [2, 2, 2, 3, 3, 3, 0, 0, 0, 1, 1, 1]
        //  ^^^^^^^^ ^^^^^^^^ ^^^^^^^^ ^^^^^^^<
        //  btm left btm rght up left  up right   <- OpenGL interpretation
        //
        let dimensions = rgb8_img.dimensions();
        RawImage2d::from_raw_rgb_reversed(&rgb8_img.to_vec(), dimensions)
    }

    // now, we generate some world data
    const WORLD_WIDTH: u32 = 1000;
    const WORLD_HEIGHT: u32 = 1000;

    let worlddata = generate_worlddata();
    let world = RawImage2d::from_raw_rgb(worlddata, (WORLD_WIDTH, WORLD_HEIGHT));
    let world = Texture2d::new(&display, world).unwrap();

    fn generate_worlddata() -> Vec<u8> {
        const BYTES_PER_RGB_VALUE: usize = 3;
        let mut data = vec![0; BYTES_PER_RGB_VALUE * (WORLD_WIDTH * WORLD_HEIGHT) as usize];

        // remember, we're dealing with UV coordinates - the coordinates OpenGL
        // uses for textures
        //
        //   1     1
        // 0 A-----B 1
        //   |     |
        //   |     |
        // 0 C-----D 1
        //   0     0
        //
        // A = (1, 0)
        // B = (1, 1)
        // C = (0, 0)
        // D = (0, 1)
        //
        // however, those conceptual values *aren't the ones we're using here*.
        // this is becauase our fragment shader **turns these values into indices**,
        // meaning that `(2, 0)` on a `4x4` tilemap would correspond to `(0.5, 0)`
        //
        // we are **just providing indices** for the fragment shader to pick up on.
        // read the fragment shader for more info
        type Xy = (u8, u8);
        const BOTTOM_LEFT: Xy = (0, 0);
        const BOTTOM_RIGHT: Xy = (0, 1);
        const UPPER_LEFT: Xy = (1, 0);
        const UPPER_RIGHT: Xy = (1, 1);

        // arbitrary pattern
        let pattern = [BOTTOM_LEFT, BOTTOM_RIGHT, UPPER_LEFT, UPPER_RIGHT];

        // now for how we'll set the data:
        //
        // every pixel in our worlddata is a triplet of RGB values
        // and we're filling in data *starting from the bottom*. see the above
        // function about `_reversed` for more on that
        //
        // we're dealing with *single bytes at a time* - so we want to iterate
        // on triplets of RGB values
        let pixels = data.chunks_mut(BYTES_PER_RGB_VALUE);
        //
        // then, we'll just set the R and the G values to our X and Y values for the tilemap
        for (pixel, (x, y)) in pixels.zip(pattern.iter().cycle()) {
            pixel[0] = *x;
            pixel[1] = *y;
            pixel[2] = 0; // we don't care about the third pixel value
        }

        data
    }

    let vertices = gen_vertices();
    let vertices = VertexBuffer::new(&display, &vertices).unwrap();

    // once you've read `gen_vertices`, come back and read this:
    //
    // now, we set up the indices for the triangles. recall the quad structure:
    //
    // 0-1
    // |/|
    // 3-2
    //
    // here, i've selected (3, 1, 2) to be the first triangle
    //
    //   1
    //  /|
    // 3-2
    //
    // and (3, 0, 1) to be the second triangle
    //
    // 0-1
    // |/
    // 3
    //
    // importantly, notice that the `3` vertex is first. this is because we
    // need a frame of reference coordinate for the beginning of the quad. more
    // info in the shaders. NOTE: the first vertex *doesn't always matter*, but
    // we *told* OpenGL it does down in the draw call. ctrl + f "provoking vertex"
    let quad_indices = glium::index::IndexBuffer::<u8>::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &[
            3, 1, 2, // first triangle
            3, 0, 1, // second triangle
        ],
    )
    .unwrap();

    fn gen_vertices() -> Vec<Vertex> {
        // the `MAP_WIDTH` and `MAP_HEIGHT` are for how big the quad is, in
        // OpenGL coordinate space
        //
        // these correspond *directly* to `quadSize` in `fragment_shader.frag`
        const MAP_WIDTH: f32 = 1.0;
        const MAP_HEIGHT: f32 = 1.0;

        // this is just a chosen offset to place the triangle at
        // we want it centered, so i chose -0.5
        const MAP_X: f32 = -0.5;
        const MAP_Y: f32 = -0.5;

        // our triangle vertices are set up like so:
        //
        //                 _ +1
        //                 |
        //      0-----1    |
        //      |    /|    |
        //      |   / |    |
        //      |  /  |    | OpenGL Y Axis
        //      | /   |    |
        //      |/    |    |
        //      3-----2    |
        //                 |
        //                 |
        // |-OpenGL X Axis-| -1
        // -1             +1
        //
        //
        // notice how the vertex ordering is completely arbitrary. it's up to
        // you to order them the way you want
        //
        let vertices = vec![
            // 0
            Vertex {
                position: [MAP_X, MAP_Y + MAP_HEIGHT],
            },
            // 1
            Vertex {
                position: [MAP_X + MAP_WIDTH, MAP_Y + MAP_HEIGHT],
            },
            // 2
            Vertex {
                position: [MAP_X + MAP_WIDTH, MAP_Y],
            },
            // 3
            Vertex {
                position: [MAP_X, MAP_Y],
            },
        ];

        vertices
    }

    // load our GPU program
    // i wouldn't look into `vertex_shader` or `fragment_shader` yet, only
    // once you've finished reading the draw call
    let vertex_shader_src = include_str!("./vertex_shader.vert");
    let fragment_shader_src = include_str!("./fragment_shader.frag");
    let program =
        Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    // keep a timer for how long simulation has been running
    let start = Instant::now();

    // for every frame
    event_loop.run(move |event, _, control_flow| {
        let time = 1.0 / (Instant::now() - start).as_secs_f32().powi(2);

        // get a handle to draw with
        let mut target = display.draw();

        // clear the frame
        target.clear_color(0.0, 0.0, 0.0, 0.0);

        //
        // set up our draw call for the tilemap
        //

        // the drawing parameters
        let draw_params = DrawParameters {
            // set the provoking vertex to the first vertex.
            //
            // this is so that the bottom left corner of our triangle is
            // considered the provoking vertex.
            //
            // for more information on why we're doing that, check the vertex
            // shader
            provoking_vertex: ProvokingVertex::FirstVertex,
            ..Default::default()
        };

        // sample our tilemap with nearest neighbor sampling when zoomed in, and
        // linear when zoomed out
        //
        // this is because when zoomed in, using nearest neighbor makes the blocks
        // look super crisp. when zoomed out, linear interpolation allows it to
        // get approximately the right color.
        let tilemap_sampler = (tilemap.sampled())
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::LinearMipmapLinear);

        // now this sampler is *important* to *stay nearest neighbor*.
        //
        // why? because we the world map has the precise values of the blocks we
        // want. interpolating them would just screw up those values.
        let worldmap_sampler = (world.sampled())
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest);

        // make the actual draw call
        target
            .draw(
                &vertices,
                &quad_indices,
                &program,
                &uniform! {
                    zoom: time,
                    tilemap: tilemap_sampler,
                    indices: worldmap_sampler,
                },
                &draw_params,
            )
            .unwrap();

        target.finish().unwrap();

        // you may want to implement VSync, omitted here for brevity
        //
        // https://github.com/glium/glium/blob/master/book/tuto-01-getting-started.md#creating-a-window

        // close event handling for window
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
            return;
        }
    });
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);
