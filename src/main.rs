extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

extern crate catchit;

use catchit::{Engine, Object, CollisionShape, ObstacleKind, State, Extent};
use catchit::Scalar as CatchitScalar;

use piston::window::WindowSettings;
use piston::event::{ RenderArgs, UpdateArgs, Events, RenderEvent, UpdateEvent, MouseCursorEvent,
                     EventLoop, PressEvent };
use piston::input::{Button, Key};
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };
use opengl_graphics::glyph_cache::GlyphCache;
use graphics::character::CharacterCache;
use graphics::math::Scalar;

pub struct App {
    gl: GlGraphics,
    engine: Engine,
    last_state: Option<State>,
    field_border_y: Scalar,
    max_score: u32,
    text_height: f64,
    tries: u32,
    font_fira_bold: GlyphCache<'static>
}

const WIDTH: u16 = 1024;
const HEIGHT: u16 = 768;
const UPDATES_PER_SECOND: u64 = 60;
const FONT_SIZE: u32 = 22;
const HUD_SPACE: Scalar = 1.0 / 8.0;
const NEW_GAME_TEXT: &'static str = "Press SPACE for new game";

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::{ rectangle, ellipse, clear , Transformed, Text, Line };
        use graphics::math::Matrix2d;

        const BG:    [f32; 4] = [1.0, 204.0 / 255.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
        const WHITE: [f32; 4] = [0.8, 0.8, 0.8, 1.0];
        const RED:   [f32; 4] = [204.0 / 255.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 1.0);
        let s = self.engine.state();
        let font_fira_bold = &mut self.font_fira_bold;
        let text_height = self.text_height;
        let max_score = self.max_score;
        let tries = self.tries;
        let field_border_y = self.field_border_y;
        let last_state = &self.last_state;


        self.gl.draw(args.viewport(), |c, gl| {
            let draw_object = |obj: &Object, gl: &mut GlGraphics, color: [f32; 4]| {
                let transform = c.transform.trans(obj.pos[0] - obj.half_size, 
                                                  obj.pos[1] - obj.half_size)
                                           .scale(obj.half_size * 2.0, obj.half_size * 2.0);
                match obj.shape {
                    CollisionShape::Square => rectangle(color, square, transform, gl),
                    CollisionShape::Circle => ellipse(color, square, transform, gl),
                }
            };
            clear(BG, gl);

            let text = Text::colored(BLACK, FONT_SIZE);
            let text_matrix = |x: Scalar| -> Matrix2d {
                c.transform.trans(x, HEIGHT as Scalar - text_height / 2.0)
            };

            if let Some(ref s) = s.as_ref().or(last_state.as_ref()) {
                for obstacle in &s.obstacles {
                    let color = match obstacle.kind {
                        ObstacleKind::Deadly => BLACK,
                        ObstacleKind::InvisibiltySwitch => WHITE,
                    };
                    draw_object(&obstacle.object, gl, color);
                }

                draw_object(&s.prey, gl, RED);
                draw_object(&s.hunter, gl, RED);

                text.draw(&format!("Score: {}", s.score), font_fira_bold, 
                                                          &c.draw_state,
                                                          text_matrix(WIDTH as Scalar 
                                                                      * HUD_SPACE * 6.0),
                                                          gl);
            }

            if last_state.is_some() {
                let w = text_width(font_fira_bold, NEW_GAME_TEXT) / 2.0;
                text.draw(NEW_GAME_TEXT, font_fira_bold, &c.draw_state,
                                          c.transform.trans(WIDTH as Scalar / 2.0 - w,
                                                           HEIGHT as Scalar / 2.0),
                                          gl);
            }

            // Draw HUD
            ////////////
            let line = Line::new(BLACK, 1.0);

            line.draw([0.0, 0.0, WIDTH as Scalar, 0.0], &c.draw_state, 
                                                         c.transform.trans(0.0, field_border_y),
                                                         gl);

            text.draw(&format!("Best Score: {}", max_score), font_fira_bold, 
                                                         &c.draw_state,
                                                         text_matrix(WIDTH as Scalar 
                                                                     * HUD_SPACE * 1.0),
                                                         gl);

            text.draw(&format!("Tries: {}", tries),      font_fira_bold, 
                                                         &c.draw_state,
                                                         text_matrix(WIDTH as Scalar 
                                                                     * HUD_SPACE * 3.0),
                                                         gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if let Err(state) = self.engine.update(args.dt) {
            if state.score > self.max_score {
                self.max_score += state.score;
            }
            self.tries += 1;
            self.last_state = Some(state);
        }
    }
}

fn compute_field(width: u16, height: u16, text_height: Scalar) -> Extent {
    [width as CatchitScalar, height as CatchitScalar 
                                    - (text_height * 2.0)]
}

fn text_width(cache: &mut GlyphCache<'static>, text: &str) -> Scalar {
    let mut w = 0.0;
    for c in text.chars() {
        w += cache.character(FONT_SIZE, c).width();
    }
    w
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

    let mut app = {
        let gl = GlGraphics::new(OpenGL::_3_2);
        let mut glyphs = GlyphCache::from_bytes(include_bytes!("../res/FiraMono-Bold.ttf"))
                                                              .unwrap();
        let text_height = glyphs.character(FONT_SIZE, 'S').top();
        let field = compute_field(WIDTH, HEIGHT, text_height);

        // Create a new game and run it.
        App {
            gl: gl,
            engine: Engine::from_field(field),
            last_state: None,
            field_border_y: field[1],
            text_height: text_height,
            tries: 0,
            max_score: 0,
            font_fira_bold: glyphs,
        }
    };


    for e in window.events()
                   .max_fps(UPDATES_PER_SECOND)
                   .ups(UPDATES_PER_SECOND) {
        if let Some(pos) = e.mouse_cursor_args() {
            app.engine.set_hunter_pos(pos);
        }

        if let Some(Button::Keyboard(Key::Space)) = e.press_args() {
            app.last_state = None;
            app.engine.reset(compute_field(WIDTH, HEIGHT, app.text_height));
        }

        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}
