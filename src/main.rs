extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

extern crate catchit;

use catchit::{Engine, Scalar, Object, CollisionShape, ObstacleKind};

use piston::window::WindowSettings;
use piston::event::{ RenderArgs, UpdateArgs, Events, RenderEvent, UpdateEvent, MouseCursorEvent,
                     EventLoop };
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use graphics::Context;

pub struct App {
    gl: GlGraphics,
    engine: Engine,
}

const WIDTH: u16 = 1024;
const HEIGHT: u16 = 768;
const UPDATES_PER_SECOND: u64 = 60;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::{ rectangle, ellipse, clear , Transformed };

        const BG:    [f32; 4] = [1.0, 204.0 / 255.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
        const WHITE: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
        const RED:   [f32; 4] = [204.0 / 255.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 1.0);
        let s = self.engine.state();

        self.gl.draw(args.viewport(), |c, gl| {
            let draw_object = |obj: &Object, gl: &mut GlGraphics, color: [f32; 4]| {
                let transform = c.transform.trans(obj.pos[0] - obj.size / 2.0, 
                                                  obj.pos[1] - obj.size / 2.0)
                                           .scale(obj.size, obj.size);
                match obj.shape {
                    CollisionShape::Square => rectangle(color, square, transform, gl),
                    CollisionShape::Circle => ellipse(color, square, transform, gl),
                }
            };
            clear(BG, gl);

            if let &Some(ref s) = s {
                for obstacle in &s.obstacles {
                    let color = match obstacle.kind {
                        ObstacleKind::Deadly => BLACK,
                        ObstacleKind::InvisibiltySwitch => WHITE,
                    };
                    draw_object(&obstacle.object, gl, color);
                }

                draw_object(&s.prey, gl, RED);
                draw_object(&s.hunter, gl, RED);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        
    }
}

fn main() {
    // Create an Glutin window.
    let window = Window::new(
        WindowSettings::new(
            "catchit",
            [WIDTH as u32, HEIGHT as u32]
        )
        .exit_on_esc(true)
        .vsync(true)
        .samples(4)
    );

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(OpenGL::_3_2),
        engine: Engine::from_field([WIDTH as Scalar, HEIGHT as Scalar]),
    };

    let mut tries = 0u32;
    let mut max_score = 0u32;

    for e in window.events()
                   .max_fps(UPDATES_PER_SECOND)
                   .ups(UPDATES_PER_SECOND) {
        if let Some(pos) = e.mouse_cursor_args() {
            app.engine.set_hunter_pos(pos);
        }

        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
