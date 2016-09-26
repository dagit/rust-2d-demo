#[macro_use]
extern crate glium;
extern crate glutin;
extern crate immi;
extern crate rusttype;
extern crate image;

use rusttype::Font;
use glium::texture::texture2d::Texture2d;

use std::borrow::Cow;
use std::marker::PhantomData;
use std::io::Cursor;

pub struct DemoDrawer<'a> {
    // Just here for the lifetime parameter
    font_proxy: PhantomData<&'a Font<'a>>,

    display:    &'a glium::Display,
    ui_shader:  &'a glium::Program,
    frame:      glium::Frame,
}

impl<'a> immi::Draw for DemoDrawer<'a> {
    type ImageResource = Texture2d;
    type TextStyle     = Font<'a>;

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
            tex: texture,
        };

        self.frame.draw(&vertex_buffer, &indices, &self.ui_shader,
                        &uniforms, &Default::default()).unwrap();
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
    fn draw_glyph(&mut self, font: &Font<'a>, c: char, matrix: &immi::Matrix)
    {
        // TODO: this information should be font value instead of
        // directly passing the font.
        let scale     = rusttype::Scale::uniform(24.0);
        let v_metrics = font.v_metrics(scale);
        let offset    = rusttype::point(0.0, v_metrics.ascent);

        let glyphs: Vec<rusttype::PositionedGlyph> = font
            .layout(&c.to_string(), rusttype::Scale::uniform(24.0), offset)
            .collect();

        for glyph in glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                let (width, height) = (bb.width() as u32, bb.height() as u32);
                let mut pixels = vec![0; (height * width) as usize];
                glyph.draw(|x,y,v| {
                    let v = ((v * 255.0) + 0.5).floor().max(0.0).min(255.0) as u8;
                    pixels[(y * width + x) as usize] = v;
                });
                let raw = glium::texture::RawImage2d {
                    data:   Cow::Borrowed(pixels.as_slice()),
                    width:  width,
                    height: height,
                    format: glium::texture::ClientFormat::U8
                };
                let tex = glium::texture::texture2d::Texture2d::new(self.display, raw).unwrap();
                self.draw_image(&tex, matrix);
            }
        }
    }

    /// Returns the height of a line of text in EMs.
    ///
    /// This value is usually somewhere around `1.2`.
    /// TODO: get this from the font info
    fn line_height(&self, _: &Font<'a>) -> f32 { 1.2 }
    
    fn kerning(&self, font: &Font<'a>, a: char, b: char) -> f32
    {
        let scale = rusttype::Scale::uniform(24.0);
        font.pair_kerning(scale, a, b)
    }
    
    fn glyph_infos(&self, font: &Font<'a>, c: char) -> immi::GlyphInfos {
        let scale     = rusttype::Scale::uniform(24.0);
        let glyph     = font.glyph(c).unwrap().scaled(scale);
        let h_metrics = glyph.h_metrics();
        let v_metrics = font.v_metrics(scale);
        //let factor = font.info.scale_for_pixel_height(scale.y) * (scale.x / scale.y);
        // TODO: this might be totally wrong. Width/height should probably be the
        // bounding box width/height.
        immi::GlyphInfos { width: h_metrics.advance_width,
                          height: v_metrics.ascent - v_metrics.descent,
                          x_offset: 0.0,
                          y_offset: 1.0,
                          x_advance: 1.0 }
    }
    
}

fn main() {
    use glium::{DisplayBuild, Surface};

    let display      = glium::glutin::WindowBuilder::new()
        .with_vsync()
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

    let font_data    = include_bytes!("../assets/font.ttf");
    let collection   = rusttype::FontCollection::from_bytes(font_data as &[u8]);
    let font         = collection.into_font().unwrap();

    struct UiState<'a> {
        #[allow(dead_code)]
        immi_state: immi::UiState,
        
        #[allow(dead_code)]
        background: Texture2d,
        
        #[allow(dead_code)]
        font:       Font<'a>,
    }

    let image   = image::load(Cursor::new(&include_bytes!("../bee-trixel.png")[..]),
                             image::PNG).unwrap().to_rgba();
    let id      = image.dimensions();
    let image   = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), id);
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let mut ui_state : UiState = UiState {
        immi_state: Default::default(),
        background: texture,
        font:       font,
    };

    fn draw_ui<'a>(ctxt: &immi::DrawContext<DemoDrawer<'a>>, ui_state: &mut UiState<'a>)
    {
        // This doesn't render correctly, not sure why
        immi::widgets::image::draw(ctxt, &ui_state.background, &immi::Alignment::top());
        //immi::widgets::label::flow(ctxt, &ui_state.font, &"Hello, World!", &immi::HorizontalAlignment::Left);
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
        let ui_context = ui_context.draw(1024.0, 768.0, &mut drawer, None, false, false);
        
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
