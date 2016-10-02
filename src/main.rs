#[macro_use]
extern crate glium;
extern crate glium_sdl2;
extern crate sdl2;
extern crate image;
extern crate rand;
extern crate time;

#[macro_use]
extern crate imgui;

use glium::texture::texture2d::Texture2d;

use std::io::Cursor;

fn print_context_info(display: &glium_sdl2::SDL2Facade)
{
    use glium::{Api, Profile, Version};

    let version       = *display.get_opengl_version();
    let api           = match version {
        Version(Api::Gl  , _, _) => "OpenGL",
        Version(Api::GlEs, _, _) => "OpenGL ES",
    };

    println!("{} context version: {}", api, display.get_opengl_version_string());
    print!("{} context flags:", api);

    if display.is_forward_compatible() {
        print!(" forward-compatible");
    }
    if display.is_debug() {
        print!(" debug");
    }
    if display.is_robust() {
        print!(" robustness");
    }
    print!("\n");

    if version >= Version(Api::Gl, 3, 2) {
        println!("{} profile mask: {}", api,
                 match display.get_opengl_profile (){
                     Some(Profile::Core)          => "core",
                     Some(Profile::Compatibility) => "compatibility",
                     None                         => "unknown",
                 });
    }

    println!("{} robustness strategy: {}", api,
             if display.is_context_loss_possible() {
                 "lose"
             } else {
                 "none"
             });

    println!("{} context renderer: {}", api, display.get_opengl_renderer_string());
    println!("{} context vendor: {}", api, display.get_opengl_vendor_string());
}

use sdl2::audio::{AudioCallback, AudioSpecDesired};

struct Squarewave {
    phase_inc: f32,
    phase:     f32,
    volume:    f32,
}

impl AudioCallback for Squarewave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]){
        // Generate a square wave
        for x in out.iter_mut() {
            *x = match self.phase {
                0.0 ... 0.5 =>  self.volume,
                _           => -self.volume,
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn start_squarewave(sdl_context: &sdl2::Sdl) -> sdl2::audio::AudioDevice<Squarewave>
{

    // TODO: we need a way to signal exit sharing a channel with
    // the main thread would be a nice abstraction.

    let audio_subsystem = sdl_context.audio().unwrap();

    let audio_spec = AudioSpecDesired {
        freq:     Some(44100),
        channels: Some(1), // mono
        samples:  None,    // default sample rate
    };

    let device = audio_subsystem.open_playback(None, &audio_spec, |spec|{
        // Show the spec we got
        println!("{:?}", spec);
        Squarewave {
            phase_inc: 440.0 / spec.freq as f32,
            phase:     0.0,
            volume:    0.05,
        }
    }).unwrap();

    // Start playback
    device.resume();
    device
}

#[allow(unused_variables)]
fn main() {
    use glium_sdl2::DisplayBuild;
    use sdl2::video::GLProfile;

    let sdl_context     = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_attr         = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);

    let window_width  = 1024;
    let window_height = 768;
    let display       = video_subsystem.window("Rust 2D Demo", window_width, window_height)
        .resizable()
        .build_glium()
        .unwrap();
    print_context_info(&display);

    let shader    = program!(
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

    let image   = image::load(Cursor::new(&include_bytes!("../assets/bee-trixel.png")[..]),
                             image::PNG).unwrap().to_rgba();
    let id      = image.dimensions();
    let image   = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), id);
    let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    // For reasons I don't understand, if we don't name the result of
    // start_squarewave then it will never start playing. Perhaps it
    // gets dropped immediately which would cause the thread to get
    // killed.
    let x = start_squarewave(&sdl_context);

    let mut support = Support::init(display, sdl_context);
    'main: loop {
        // Check if we should exit before we do anything else
        let active = support.update_events();
        if !active { break 'main }
        support.render((0.0, 0.0, 1.0, 1.0), display_info,
                       |frame: &mut glium::Frame, display: &mut glium_sdl2::Display| {
                           draw_scene(frame, display, &texture, &shader);
                       });
    }
}

fn draw_scene(frame: &mut glium::Frame,
              display: &mut glium_sdl2::Display,
              texture: &Texture2d,
              program: &glium::Program)
{
    use glium::Surface;
    #[derive(Copy, Clone)]
    struct Vertex {
        position:  [f32; 2],
        uv_coords: [f32; 2],
    }

    implement_vertex!(Vertex, position, uv_coords);

    let vertex1 = Vertex { position: [-0.5, -0.5],  uv_coords: [0.0, 0.0] };
    let vertex2 = Vertex { position: [ 0.0,  0.5],  uv_coords: [0.0, 1.0] };
    let vertex3 = Vertex { position: [ 0.5, -0.25], uv_coords: [1.0, 0.0] };
    let shape = vec![vertex1, vertex2, vertex3];

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
    let uniforms = uniform! {
        matrix: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ],
        tex: texture.sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
    };

    let blend = glium::Blend::alpha_blending();
    let draw_params = glium::DrawParameters { blend: blend, ..Default::default() };
    frame.draw(&vertex_buffer, &indices, program, &uniforms, &draw_params).unwrap();
}

fn display_info<'a>(ui: &Ui<'a>) {
    ui.window(im_str!("Debug Info"))
        .size((400.0, 100.0), ImGuiSetCond_Always)
        .position((0.0, 0.0), ImGuiSetCond_Always)
        .collapsible(false)
        .movable(false)
        .resizable(false)
        .title_bar(true)
        .show_borders(false)
        .build(|| {
            let mouse_pos = ui.imgui().mouse_pos();
            ui.text(im_str!("Mouse Position: ({:.1},{:.1})", mouse_pos.0, mouse_pos.1));
            ui.text(im_str!("Frame rate: {:.1}", ui.framerate()));
        })
}

// Everything here for imgui is taken basically verbatim from the
// example in the imgui-rs repo
use imgui::*;
use imgui::glium_renderer::Renderer;
use std::time::Instant;


struct Support {
    display:       glium_sdl2::Display,
    event_pump:    sdl2::EventPump,
    imgui:         ImGui,
    renderer:      Renderer,
    last_frame:    Instant,
    mouse_pos:     (i32, i32),
    mouse_pressed: (bool, bool, bool),
    mouse_wheel:   f32,
}

impl Support {
    pub fn init(display: glium_sdl2::Display, sdl_context: sdl2::Sdl) -> Support {
        let mut imgui      = ImGui::init();
        let renderer       = Renderer::init(&mut imgui, &display).unwrap();
        let event_pump     = sdl_context.event_pump().unwrap();

        imgui.set_imgui_key(ImGuiKey::Tab,        0);
        imgui.set_imgui_key(ImGuiKey::LeftArrow,  1);
        imgui.set_imgui_key(ImGuiKey::RightArrow, 2);
        imgui.set_imgui_key(ImGuiKey::UpArrow,    3);
        imgui.set_imgui_key(ImGuiKey::DownArrow,  4);
        imgui.set_imgui_key(ImGuiKey::PageUp,     5);
        imgui.set_imgui_key(ImGuiKey::PageDown,   6);
        imgui.set_imgui_key(ImGuiKey::Home,       7);
        imgui.set_imgui_key(ImGuiKey::End,        8);
        imgui.set_imgui_key(ImGuiKey::Delete,     9);
        imgui.set_imgui_key(ImGuiKey::Backspace, 10);
        imgui.set_imgui_key(ImGuiKey::Enter,     11);
        imgui.set_imgui_key(ImGuiKey::Escape,    12);
        imgui.set_imgui_key(ImGuiKey::A,         13);
        imgui.set_imgui_key(ImGuiKey::C,         14);
        imgui.set_imgui_key(ImGuiKey::V,         15);
        imgui.set_imgui_key(ImGuiKey::X,         16);
        imgui.set_imgui_key(ImGuiKey::Y,         17);
        imgui.set_imgui_key(ImGuiKey::Z,         18);

        Support {
            display:       display,
            event_pump:    event_pump,
            imgui:         imgui,
            renderer:      renderer,
            last_frame:    Instant::now(),
            mouse_pos:     (0, 0),
            mouse_pressed: (false, false, false),
            mouse_wheel:   0.0,
        }
    }

    pub fn update_mouse(&mut self) {
        let scale = self.imgui.display_framebuffer_scale();
        self.imgui.set_mouse_pos(self.mouse_pos.0 as f32 / scale.0, self.mouse_pos.1 as f32 / scale.1);
        self.imgui.set_mouse_down(&[self.mouse_pressed.0, self.mouse_pressed.1, self.mouse_pressed.2, false, false]);
        self.imgui.set_mouse_wheel(self.mouse_wheel / scale.1);
        self.mouse_wheel = 0.0;
    }

    pub fn render<F: FnMut(&Ui), G: FnMut(&mut glium::Frame, &mut glium_sdl2::Display)>
        (&mut self, clear_color: (f32, f32, f32, f32), mut run_ui: F, mut draw_scene: G)
    {
        use glium::Surface;
        let now     = Instant::now();
        let delta   = now - self.last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        self.last_frame = now;

        self.update_mouse();

        let mut frame   = self.display.draw();
        frame.clear_color(clear_color.0, clear_color.1, clear_color.2, clear_color.3);
        draw_scene(&mut frame, &mut self.display);

        let window      = self.display.window();

        let size_pixels = window.drawable_size();
        // TODO: it might be wrong to use the size_pixels twice, the
        // code this is based off of also had a `size_points`, but
        // I'm not sure what tht means or how to calculate it.
        let ui = self.imgui.frame(size_pixels, size_pixels, delta_s);

        run_ui(&ui);

        self.renderer.render(&mut frame, ui).unwrap();

        frame.finish().unwrap();
    }

    pub fn update_events(&mut self) -> bool {
        let set_key_state = |imgui: &mut ImGui, scancode: Option<sdl2::keyboard::Scancode>, pressed: bool| {
            use sdl2::keyboard::Scancode;
            match scancode {
                Some(Scancode::Tab)                             => imgui.set_key(0,  pressed),
                Some(Scancode::Left)                            => imgui.set_key(1,  pressed),
                Some(Scancode::Right)                           => imgui.set_key(2,  pressed),
                Some(Scancode::Up)                              => imgui.set_key(3,  pressed),
                Some(Scancode::Down)                            => imgui.set_key(4,  pressed),
                Some(Scancode::PageUp)                          => imgui.set_key(5,  pressed),
                Some(Scancode::PageDown)                        => imgui.set_key(6,  pressed),
                Some(Scancode::Home)                            => imgui.set_key(7,  pressed),
                Some(Scancode::End)                             => imgui.set_key(8,  pressed),
                Some(Scancode::Delete)                          => imgui.set_key(9,  pressed),
                Some(Scancode::Backspace)                       => imgui.set_key(10, pressed),
                Some(Scancode::Return)                          => imgui.set_key(11, pressed),
                Some(Scancode::Escape)                          => imgui.set_key(12, pressed),
                Some(Scancode::A)                               => imgui.set_key(13, pressed),
                Some(Scancode::C)                               => imgui.set_key(14, pressed),
                Some(Scancode::V)                               => imgui.set_key(15, pressed),
                Some(Scancode::X)                               => imgui.set_key(16, pressed),
                Some(Scancode::Y)                               => imgui.set_key(17, pressed),
                Some(Scancode::Z)                               => imgui.set_key(18, pressed),
                Some(Scancode::LCtrl)  | Some(Scancode::RCtrl)  => imgui.set_key_ctrl(pressed),
                Some(Scancode::LShift) | Some(Scancode::RShift) => imgui.set_key_shift(pressed),
                Some(Scancode::LAlt)   | Some(Scancode::RAlt)   => imgui.set_key_alt(pressed),
                Some(Scancode::LGui)   | Some(Scancode::RGui)   => imgui.set_key_super(pressed),
                _ => {}
            };
        };

        for event in self.event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit   { .. }           => return false,
                Event::KeyDown{ scancode, .. } => {
                    set_key_state(&mut self.imgui, scancode, true);
                },
                Event::KeyUp  { scancode, .. } => {
                    set_key_state(&mut self.imgui, scancode, false);
                },
                Event::MouseMotion{ x,y, .. }  => self.mouse_pos = (x,y),
                Event::MouseButtonDown{ mouse_btn, .. } => {
                    use sdl2::mouse::Mouse;
                    match mouse_btn {
                        Mouse::Left   => self.mouse_pressed.0 = true,
                        Mouse::Right  => self.mouse_pressed.1 = true,
                        Mouse::Middle => self.mouse_pressed.2 = true,
                        _             => ()
                    }
                },
                Event::MouseButtonUp{ mouse_btn, .. } => {
                    use sdl2::mouse::Mouse;
                    match mouse_btn {
                        Mouse::Left   => self.mouse_pressed.0 = false,
                        Mouse::Right  => self.mouse_pressed.0 = false,
                        Mouse::Middle => self.mouse_pressed.0 = false,
                        _             => ()
                    }
                },
                Event::MouseWheel{ y,   .. } => self.mouse_wheel = y as f32,
                Event::TextInput{ text, .. } => {
                    for c in text.chars() {
                        self.imgui.add_input_character(c);
                    }
                },
                _                  => (),
            }
        }
        true
    }

}
