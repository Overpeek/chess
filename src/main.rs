use board::{Board, BoardPos};
use main_game_loop::{AnyEngine, Event, GameLoop, Runnable, WinitEvent};
use piece::{Piece, Side};
use rand::prelude::{IteratorRandom, SliceRandom, ThreadRng};
use ron::ser::PrettyConfig;
use srs2dge::{
    batch::{quad::QuadMesh, BatchRenderer, Idx},
    glam::{Mat4, Vec2, Vec4},
    glium::{
        backend::Facade,
        program, uniform,
        uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
        Blend, DrawParameters, Frame, Program, Surface,
    },
    packer::{
        texture::{TextureAtlasMap, TextureAtlasMapBuilder},
        TexturePosition,
    },
    program::{color_2d_program, texture_2d_program, DefaultVertex},
    BuildEngine, Engine,
};
use std::{fs::File, process::exit};
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    window::WindowBuilder,
};

//

mod board;
mod piece;

//

static_res::static_res! { "res/*.png" }

macro_rules! load_png {
    ($path:expr) => {
        image::load_from_memory($path).unwrap().to_rgba8()
    };
}

//

struct App {
    color_batcher: BatchRenderer<DefaultVertex, QuadMesh>,
    tex_batcher: BatchRenderer<DefaultVertex, QuadMesh>,
    circle_batcher: BatchRenderer<DefaultVertex, QuadMesh>,
    color_program: Program,
    tex_program: Program,
    circle_program: Program,
    texture: TextureAtlasMap<(Side, Piece)>,

    turn: Side,

    circle_quads: [Idx; 64],
    piece_quads: [Idx; 64],
    hand_quad: Idx,
    board: Board,

    cursor: Option<(BoardPos, Vec2)>,
    moving: Option<(BoardPos, Side, Piece)>,
}

//

impl App {
    fn update_batch(&mut self) {
        // log::debug!("board = {:?}", self.board);
        for (side, piece, pos) in self.board.iter() {
            let i = pos.to_usize();
            let idx = self.piece_quads[i];

            // log::debug!("{side:?} {piece:?} at {pos}");

            let quad = self.tex_batcher.get_mut(idx);
            quad.size = Vec2::new(0.25, 0.25);
            quad.pos = Vec2::new(
                pos.file as f32 * 0.25 - 1.25,
                -(pos.rank as f32 * 0.25 - 1.0),
            );
            quad.col = Vec4::new(1.0, 1.0, 1.0, 1.0);
            quad.tex = *self.texture.get(&(side, piece)).unwrap();
            quad.tex.top_left.y = 1.0 - quad.tex.top_left.y;
            quad.tex.bottom_right.y = 1.0 - quad.tex.bottom_right.y;
        }

        if let Some((pos, side, piece)) = self.moving.as_ref() {
            // piece picked up
            let quad = self.tex_batcher.get_mut(self.piece_quads[pos.to_usize()]);
            quad.pos -= 0.02;
            quad.size += 0.04;

            // piece valid moves
            for piece in BoardPos::iter() {
                let idx = self.circle_quads[piece.to_usize()];
                if (self.circle_batcher.get(idx).col.w).abs() >= std::f32::EPSILON {
                    self.circle_batcher.get_mut(idx).col.w = 0.0;
                }
            }
            for piece in piece.moves(&self.board, *pos, *side) {
                if self.board.get_piece(&piece).is_some() {
                    // if it is a capture
                    // draw a frame around it
                    let idx = self.circle_quads[piece.to_usize()];
                    if self.circle_batcher.get(idx).col.w >= std::f32::EPSILON - 1.0 {
                        self.circle_batcher.get_mut(idx).col.w = -1.0;
                    }
                } else {
                    // if it is just a move
                    // draw a dot in it
                    let idx = self.circle_quads[piece.to_usize()];
                    if self.circle_batcher.get(idx).col.w <= 1.0 - std::f32::EPSILON {
                        self.circle_batcher.get_mut(idx).col.w = 1.0;
                    }
                }
            }
        } else {
            for piece in BoardPos::iter() {
                let idx = self.circle_quads[piece.to_usize()];
                if (self.circle_batcher.get(idx).col.w).abs() >= std::f32::EPSILON {
                    self.circle_batcher.get_mut(idx).col.w = 0.0;
                }
            }
        }
    }

    pub fn circle_program<F>(facade: &F) -> Program
    where
        F: Facade,
    {
        program!(facade,
            140 => {
                vertex: "#version 140
                in vec2 vi_position;
                in vec4 vi_color;
                in vec2 vi_uv;

                uniform mat4 mat;

                out vec4 fi_color;
                out vec2 fi_uv;

                void main() {
                    gl_Position = mat * vec4(vi_position, 0.0, 1.0) * vec4(1.0, -1.0, 1.0, 1.0);
                    fi_color = vi_color;
                    fi_uv = vi_uv;
                }",
                fragment: "#version 140
                in vec4 fi_color;
                in vec2 fi_uv;

                out vec4 o_color;

                void main() {
                    vec2 uv = fi_uv - 0.5;
                    o_color = fi_color;

                    if (o_color.a >= 0.0) {
                        o_color.a *= smoothstep(0.03, 0.023, uv.x * uv.x + uv.y * uv.y) * 0.5;
                    } else {
                        o_color.a = smoothstep(0.32, 0.35, uv.x * uv.x + uv.y * uv.y) * 0.5;
                    }
                }",
                outputs_srgb: true
            }
        )
        .unwrap_or_else(|err| panic!("Default program failed to compile: {}", err))
    }
}

impl Runnable<Engine> for App {
    fn init(gl: &mut GameLoop<Engine>) -> Self {
        let color_program = color_2d_program(&gl.engine);
        let tex_program = texture_2d_program(&gl.engine);
        let circle_program = Self::circle_program(&gl.engine);

        let texture = TextureAtlasMapBuilder::new()
            .with((Side::Black, Piece::Rook), load_png!(res::rook_b_png))
            .with((Side::Black, Piece::Knight), load_png!(res::knight_b_png))
            .with((Side::Black, Piece::Bishop), load_png!(res::bishop_b_png))
            .with((Side::Black, Piece::Queen), load_png!(res::queen_b_png))
            .with((Side::Black, Piece::King), load_png!(res::king_b_png))
            .with((Side::Black, Piece::Pawn), load_png!(res::pawn_b_png))
            .with((Side::White, Piece::Rook), load_png!(res::rook_w_png))
            .with((Side::White, Piece::Knight), load_png!(res::knight_w_png))
            .with((Side::White, Piece::Bishop), load_png!(res::bishop_w_png))
            .with((Side::White, Piece::Queen), load_png!(res::queen_w_png))
            .with((Side::White, Piece::King), load_png!(res::king_w_png))
            .with((Side::White, Piece::Pawn), load_png!(res::pawn_w_png))
            .build(&gl.engine);

        let mut file = File::options()
            // .create(true)
            // .append(false)
            .create(true)
            .write(true)
            .open("atlas.ron")
            .unwrap();
        ron::ser::to_writer_pretty(&mut file, &texture.convert(), PrettyConfig::default()).unwrap();
        drop(file);

        let mut color_batcher = BatchRenderer::new(&gl.engine);
        let mut tex_batcher = BatchRenderer::new(&gl.engine);
        let mut circle_batcher = BatchRenderer::new(&gl.engine);

        (0..64)
            .map(|i| {
                let u = i % 8;
                let v = i / 8;
                let x = u as f32 * 0.25 - 1.0;
                let y = -(v as f32 * 0.25 - 0.75);
                let p = Vec2::new(x, y);
                let c = if (u + v) % 2 == 0 {
                    Vec4::new(0.8, 0.4, 0.4, 1.0)
                } else {
                    Vec4::new(0.5, 0.1, 0.1, 1.0)
                };
                (p, c)
            })
            .for_each(|(pos, col)| {
                color_batcher.push_with(QuadMesh {
                    pos,
                    size: Vec2::new(0.25, 0.25),
                    col,
                    tex: TexturePosition::default(),
                });
            });

        let circle_quads = (0..64)
            .map(|i| {
                let u = i % 8;
                let v = i / 8;
                let x = u as f32 * 0.25 - 1.0;
                let y = -(v as f32 * 0.25 - 0.75);
                Vec2::new(x, y)
            })
            .map(|pos| {
                circle_batcher.push_with(QuadMesh {
                    pos,
                    size: Vec2::new(0.25, 0.25),
                    col: Vec4::new(0.0, 0.0, 0.0, 1.0),
                    tex: TexturePosition::default(),
                })
            })
            .collect::<Vec<Idx>>()
            .try_into()
            .unwrap();

        let piece_quads = (0..64)
            .map(|_| {
                tex_batcher.push_with(QuadMesh {
                    pos: Vec2::new(0.0, 0.0),
                    size: Vec2::new(0.25, 0.25),
                    col: Vec4::new(0.0, 0.0, 0.0, 0.0),
                    tex: TexturePosition::default(),
                })
            })
            .collect::<Vec<Idx>>()
            .try_into()
            .unwrap();

        let hand_quad = tex_batcher.push_with(QuadMesh {
            pos: Vec2::new(0.0, 0.0),
            size: Vec2::new(0.25, 0.25),
            col: Vec4::new(0.0, 0.0, 0.0, 0.0),
            tex: TexturePosition::default(),
        });

        let board = Board::starting();
        log::debug!("{board:?}");

        let mut res = Self {
            color_batcher,
            tex_batcher,
            circle_batcher,
            color_program,
            tex_program,
            circle_program,
            texture,

            turn: Side::White,

            circle_quads,
            piece_quads,
            hand_quad,
            board,

            cursor: None,
            moving: None,
        };

        res.update_batch();

        res
    }

    fn update(&mut self, _: &mut GameLoop<Engine>) {
        if self.turn == Side::Black {
            let mut rng = rand::thread_rng();
            let (to_move, moves) = self
                .board
                .iter()
                .filter(|(side, _, _)| *side == Side::Black) // filter AI pieces
                .map(|(side, piece, pos)| {
                    (pos, piece.moves(&self.board, pos, side).collect::<Vec<_>>())
                })
                .filter(|(_, moves)| !moves.is_empty()) // filter pieces that can move
                .choose(&mut rng)
                .unwrap_or_else(|| {
                    log::info!("AI lost");
                    exit(0);
                });
            let move_to = moves.choose(&mut rng).unwrap();

            let piece_quad = self
                .tex_batcher
                .get_mut(self.piece_quads[to_move.to_usize()]);
            piece_quad.col.w = 0.0;
            let (side, piece) = self.board.remove_piece(&to_move).unwrap();
            self.board.set_piece(side, piece, *move_to);
            self.update_batch();
            self.turn = Side::White;
        }
    }

    fn event(&mut self, gl: &mut GameLoop<Engine>, event: &Event) {
        if let Event::WinitEvent(WinitEvent::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        }) = event
        {
            gl.stop();
        }

        if let Event::WinitEvent(WinitEvent::WindowEvent {
            event: WindowEvent::CursorLeft { .. },
            ..
        }) = event
        {
            self.cursor = None;
        }

        if let Event::WinitEvent(WinitEvent::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        }) = event
        {
            // screen pos
            let x = position.x as f32;
            let y = position.y as f32;

            // map to world pos
            let x = x / gl.size.0 * gl.aspect * 2.4 - gl.aspect * 1.2;
            let y = -(y / gl.size.1 * 2.4 - 1.2);

            // map to board pos
            let x = ((x + 1.0) * 4.0).floor() as i32 + 1;
            let y = ((y + 1.0) * 4.0).floor() as i32 + 1;

            let pos = match BoardPos::new(x, y) {
                Some(ok) => ok,
                None => {
                    // log::debug!("BoardPos out of bounds");
                    return;
                }
            };
            // log::debug!("{pos} = {pos:?}");

            self.cursor = Some((pos, Vec2::new(position.x as f32, position.y as f32)));
        }

        if let Event::WinitEvent(WinitEvent::WindowEvent {
            event:
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                    ..
                },
            ..
        }) = event
        {
            if self.turn == Side::Black {
                log::debug!("It is AI:s turn");
            }
            if let Some((pos, _)) = self.cursor.as_ref() {
                if let Some((old_pos, side, piece)) = self.moving.take() {
                    if old_pos != *pos
                        && !piece
                            .moves(&self.board, old_pos, side)
                            .any(|possible_pos| &possible_pos == pos)
                    {
                        log::debug!("invalid move");
                        self.moving = Some((old_pos, side, piece));
                        return;
                    }

                    if old_pos == *pos {
                        log::debug!("move cancelled");
                        self.board.set_piece(side, piece, *pos);
                        self.update_batch();
                        return;
                    }

                    self.turn = self.turn.other();
                    log::debug!("drop {pos} from {old_pos}");
                    let piece_quad = self
                        .tex_batcher
                        .get_mut(self.piece_quads[old_pos.to_usize()]);
                    piece_quad.col.w = 0.0;
                    self.board.remove_piece(&old_pos);
                    self.board.set_piece(side, piece, *pos);
                    self.update_batch();
                } else if let Some((side, piece)) = self.board.get_piece(pos) {
                    if side != self.turn {
                        log::debug!("wrong player");
                        return;
                    }

                    log::debug!("pick {pos}");
                    let piece_quad = self.tex_batcher.get_mut(self.piece_quads[pos.to_usize()]);
                    piece_quad.col.w = 0.0;

                    self.moving = Some((*pos, side, piece));
                    self.update_batch();
                }
            }
        }
    }

    fn draw(&mut self, gl: &mut GameLoop<Engine>, frame: &mut Frame, _: f32) {
        frame.clear_color(0.01, 0.01, 0.01, 1.0);

        let ubo = uniform! {
            mat: Mat4::orthographic_rh_gl(-gl.aspect * 1.2, gl.aspect * 1.2, -1.2, 1.2, 0.0, 100.0).to_cols_array_2d()
        };

        let (vbo, ibo) = self.color_batcher.draw(&gl.engine);
        frame
            .draw(
                vbo,
                ibo,
                &self.color_program,
                &ubo,
                &DrawParameters {
                    primitive_restart_index: true,
                    ..Default::default()
                },
            )
            .unwrap();

        let (vbo, ibo) = self.circle_batcher.draw(&gl.engine);
        frame
            .draw(
                vbo,
                ibo,
                &self.circle_program,
                &ubo,
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    primitive_restart_index: true,
                    ..Default::default()
                },
            )
            .unwrap();

        let ubo = uniform! {
            mat: Mat4::orthographic_rh_gl(-gl.aspect * 1.2, gl.aspect * 1.2, -1.2, 1.2, 0.0, 100.0).to_cols_array_2d(),
            sprite: self.texture
            .sampled()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .minify_filter(MinifySamplerFilter::Nearest)
        };

        let (vbo, ibo) = self.tex_batcher.draw(&gl.engine);
        frame
            .draw(
                vbo,
                ibo,
                &self.tex_program,
                &ubo,
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    primitive_restart_index: true,
                    ..Default::default()
                },
            )
            .unwrap();
    }
}

//

fn main() {
    env_logger::init();

    WindowBuilder::new()
        .with_title("Chess")
        .build_engine()
        .build_game_loop()
        .run::<App>();
}
