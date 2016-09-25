extern crate glium;
extern crate glium_text;
extern crate glutin;
extern crate cgmath;

fn main() {
    use glium::DisplayBuild;
    use glium::Surface;
    use std::io::Cursor;

    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();
    let system  = glium_text::TextSystem::new(&display);
    let font    = glium_text::FontTexture::new(&display, &include_bytes!("../assets/font.ttf")[..],70).unwrap();
    let text    = glium_text::TextDisplay::new(&system, &font, "Hello, world!");
    let text_width = text.get_width();
    println!("Text width: {:?}", text_width);
    
    'main: loop {
        let (w, h) = display.get_framebuffer_dimensions();

        let matrix:[[f32; 4]; 4] = cgmath::Matrix4::new(
            2.0 / text_width, 0.0,                                        0.0, 0.0,
            0.0,              2.0 * (w as f32) / (h as f32) / text_width, 0.0, 0.0,
            0.0,              0.0,                                        1.0, 0.0,
            -1.0,            -1.0,                                        0.0, 1.0f32,
        ).into();

        // drawing
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        glium_text::draw(&text, &system, &mut target, matrix, (1.0, 1.0, 0.0, 1.0));
        target.finish().unwrap();
    
        for ev in display.poll_events () {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}
