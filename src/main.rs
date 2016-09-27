#[macro_use]
extern crate glium;
extern crate glutin;
extern crate immi;
extern crate rusttype;
extern crate image;

use rusttype::Font;
use glium::texture::texture2d::Texture2d;

use std::marker::PhantomData;
use std::io::Cursor;

pub struct DemoDrawer<'a> {
    // Just here for the lifetime parameter
    font_proxy: PhantomData<&'a Font<'a>>,

    display:    &'a glium::Display,
    ui_shader:  &'a glium::Program,
    frame:      glium::Frame,
}

pub struct TextInfo<'a> {
    font: Font<'a>,
    size: f32,
}


pub struct UiState<'a> {
    #[allow(dead_code)]
    immi_state: immi::UiState,
    background: Texture2d,
    textinfo:   TextInfo<'a>,
}


impl<'a> immi::Draw for DemoDrawer<'a> {
    type ImageResource = Texture2d;
    type TextStyle     = TextInfo<'a>;

    fn draw_triangle(&mut self,
                     texture:   &Texture2d,
                     matrix:    &immi::Matrix,
                     uv_coords: [[f32; 2]; 3])
    {
        use glium::Surface;

        #[derive(Copy, Clone)]
        struct Vertex {
            position:  [f32; 2],
            uv_coords: [f32; 2],
        }
        implement_vertex!(Vertex, position, uv_coords);

        let vertex1 = Vertex { position: [-1.0,  1.0], uv_coords: uv_coords[0] };
        let vertex2 = Vertex { position: [-1.0, -1.0], uv_coords: uv_coords[1] };
        let vertex3 = Vertex { position: [ 1.0,  1.0], uv_coords: uv_coords[2] };
        let shape = vec![vertex1, vertex2, vertex3];

        let vertex_buffer = glium::VertexBuffer::new(self.display, &shape).unwrap();
        let indices       = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let uniforms      = uniform! {
            matrix: Into::<[[f32; 4]; 4]>::into(*matrix),
            tex:    texture,
        };

        let blend = glium::Blend::alpha_blending();
        let draw_params = glium::DrawParameters { blend: blend, ..Default::default() };
        self.frame.draw(&vertex_buffer, &indices, &self.ui_shader,
                        &uniforms, &draw_params).unwrap();
    }

    /// Given an image, this functions returns its width divided by its height.
    fn get_image_width_per_height(&mut self, texture: &Texture2d) -> f32
    {
        // TODO: is this unwrap safe?
        texture.get_width() as f32 / texture.get_height().unwrap() as f32
    }

    /// Does the same as `draw_image`, but draws a glyph of a text
    /// instead: Draws an image that covers the whole surface (from
    /// `-1.0` to `1.0` both horizontally and vertically), but
    /// multiplied by the matrix.
    ///
    /// This function should not try to preseve the aspect ratio of the
    /// image. This is handled by the caller.
    fn draw_glyph(&mut self, textinfo: &TextInfo<'a>, c: char, matrix: &immi::Matrix)
    {
        let font      = &textinfo.font;
        let scale     = rusttype::Scale::uniform(textinfo.size);
        let v_metrics = font.v_metrics(scale);
        let offset    = rusttype::point(0.0, v_metrics.ascent);

        let glyphs: Vec<rusttype::PositionedGlyph> = font
            .layout(&c.to_string(), scale, offset)
            .collect();

        fn index(x: u32, y: u32, width: u32, depth: u32) -> usize {
            (y * width * depth + x * depth) as usize
        }
        
        for glyph in glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                let (width, height) = (bb.width() as u32, bb.height() as u32);
                let depth = 4;
                // This is wasteful and we can do better. We really
                // only need to set one channel and then use a special
                // shader that knows to look at that one channel and
                // treat it as a greyscale value.
                let size = height as usize * width as usize * depth as usize;
                let mut pixels = vec![0; size];
                glyph.draw(|x,y,v| {
                    let v = ((v * 255.0) + 0.5).floor().max(0.0).min(255.0) as u8;
                    for i in 0..depth {
                        pixels[index(x,y,width,depth) + i as usize] = v;
                    }
                });
                let raw = glium::texture::RawImage2d::from_raw_rgba(pixels, (width, height));
                let tex = glium::texture::texture2d::Texture2d::new(self.display, raw).unwrap();

                // I'm not sure why, but we need to invert the y-axis
                // here.  From the docs I would have thought we didn't
                // need to do that.
                let inverty = immi::Matrix([
                    [1.0,  0.0],
                    [0.0, -1.0],
                    [0.0,  0.0],
                    ]
                );
                let invertedmat = inverty * *matrix;
                self.draw_image(&tex, &invertedmat);
            }
        }
    }

    /// Returns the height of a line of text in EMs.
    ///
    /// This value is usually somewhere around `1.2`.
    /// TODO: get this from the font info
    fn line_height(&self, _: &TextInfo<'a>) -> f32 { 1.2 }

    #[allow(unused_variables)]
    fn kerning(&self, textinfo: &TextInfo<'a>, a: char, b: char) -> f32
    {
        // For some reason all my attempts to calculate the kerning have been wrong
        // so I just return 0.0 here. Which is also not great, but works.
        // let scale = rusttype::Scale::uniform(textinfo.size);
        // textinfo.font.pair_kerning(scale, a, b);
        0.0
        
    }
    
    fn glyph_infos(&self, textinfo: &TextInfo<'a>, c: char) -> immi::GlyphInfos {
        let font      = &textinfo.font;
        let scale     = rusttype::Scale::uniform(textinfo.size);
        let v_metrics = font.v_metrics(scale);
        let offset    = rusttype::point(0.0, v_metrics.ascent);
        let glyphs: Vec<rusttype::PositionedGlyph> =
            font.layout(&c.to_string(), scale, offset)
            .collect();
        let ems: Vec<rusttype::PositionedGlyph> =
            font.layout(&'M'.to_string(), scale, offset)
            .collect();
        let glyph        = &glyphs[0];
        let em           = &ems[0];
        let h_metrics    = glyph.clone().into_unpositioned().h_metrics();
        let em_h_metrics = em.clone().into_unpositioned().h_metrics();
        if let Some(glyphbb) = glyph.pixel_bounding_box() {
            if let Some(embb) = em.pixel_bounding_box() {
                // This doesn't seem to be quite right, but it's close
                // in some cases.
                return immi::GlyphInfos { width: glyphbb.width() as f32 / embb.width() as f32,
                                          height: glyphbb.height() as f32 / embb.height() as f32,
                                          x_offset: 0.0,
                                          y_offset: 1.0,
                                          x_advance: h_metrics.advance_width / em_h_metrics.advance_width}
            } else {
                //println!("No embb");
            }
        } else {
            //println!("no glyphbb for '{}'", c);
        }

        // This is the default value if we couldn't calculate something
        // more reasonable
        immi::GlyphInfos { width: 1.0, height: 1.0, x_offset: 0.0,
                          y_offset: 1.0, x_advance: 1.0}

    }
    
}

fn main() {
    use glium::{DisplayBuild, Surface};

    let window_width  = 1024.0;
    let window_height = 768.0;
    let display      = glium::glutin::WindowBuilder::new()
        .with_vsync()
        .with_dimensions(window_width as u32, window_height as u32)
        .with_srgb(Some(true))
        .build_glium()
        .unwrap();
    let ui_shader    = program!(
        &display,
        140 => {
            vertex: r#"
                #version 140
                
                in  vec2 position;
                in  vec2 uv_coords;
                out vec2 v_uv_coords;
                
                uniform mat4 matrix;
                
                void main() {
                    v_uv_coords = uv_coords;
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                }
            "#,
            fragment: r#"
                #version 140
                
                in  vec2 v_uv_coords;
                out vec4 color;
                
                uniform sampler2D tex;
                
                void main() {
                    color = texture(tex, v_uv_coords);
                }
            "#
        }).unwrap();

    //let font_data    = include_bytes!("../assets/font.ttf");
    let font_data    = include_bytes!("../assets/Arial Unicode.ttf");
    let collection   = rusttype::FontCollection::from_bytes(font_data as &[u8]);
    let font         = collection.into_font().unwrap();

    let image   = image::load(Cursor::new(&include_bytes!("../assets/bee-trixel.png")[..]),
                             image::PNG).unwrap().to_rgba();
    let id      = image.dimensions();
    let image   = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), id);
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    // let dpi_factor = display.get_window().unwrap().hidpi_factor();
    // println!("dpi_factor is {}", dpi_factor);
    
    let mut ui_state : UiState = UiState {
        immi_state: Default::default(),
        background: texture,
        textinfo:   TextInfo {
            font: font,
            size: 1000.0,
        }
    };

    fn draw_ui<'a>(ctxt: &immi::DrawContext<DemoDrawer<'a>>, ui_state: &mut UiState<'a>)
    {

        immi::widgets::image::draw(ctxt, &ui_state.background, &immi::Alignment::top());
        // This doesn't render correctly, not sure why
        // immi::widgets::label::flow(ctxt, &ui_state.textinfo, &"Hello, World!",
        //                         &immi::HorizontalAlignment::Left);
        immi::widgets::label::contain(ctxt, &ui_state.textinfo, &"Hello, World!",
                                   &immi::Alignment::center());

    }

    'main: loop {

        // Create a new frame and clear the screen before we do
        // anything else.
        //
        // Note: We create a new DemoDrawer value every iteration
        // because it owns `frame` and we need `frame` to get dropped
        // at the end. We can't call `finish()` directly because it
        // requires a move.  So instead we call `set_finish()` and let
        // Drop force the buffer swap.
        let mut drawer = DemoDrawer {
            font_proxy: PhantomData::default(),
            display:    &display,
            ui_shader:  &ui_shader,
            frame:      display.draw(),
        };
        drawer.frame.clear_color(0.0, 0.0, 1.0, 1.0);
        let ui_context = immi::draw();
        let ui_context = ui_context.draw(window_width, window_height, &mut drawer, None, false, false);
        
        draw_ui(&ui_context, &mut ui_state);
        ui_context.draw().frame.set_finish().unwrap();
        
        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => break 'main,
                _ => {
                }
            }
        }
    }
}
