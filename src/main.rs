mod chess;
mod computer_player;
mod human_player;

use chess::*;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::*;
use sdl2::gfx::primitives::DrawRenderer;
use std::collections::HashMap;
use std::time::Duration;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let font_context = ttf::init().unwrap();
    let font = font_context.load_font("LiberationMono-Regular.ttf", 18).unwrap();

    let window = video_subsystem.window("Chess", 960, 640)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let textures = [
        (Piece::WHITE_PAWN, "white_pawn.png"),
        (Piece::BLACK_PAWN, "black_pawn.png"),
        (Piece::WHITE_KNIGHT, "white_knight.png"),
        (Piece::BLACK_KNIGHT, "black_knight.png"),
        (Piece::WHITE_BISHOP, "white_bishop.png"),
        (Piece::BLACK_BISHOP, "black_bishop.png"),
        (Piece::WHITE_ROOK, "white_rook.png"),
        (Piece::BLACK_ROOK, "black_rook.png"),
        (Piece::WHITE_QUEEN, "white_queen.png"),
        (Piece::BLACK_QUEEN, "black_queen.png"),
        (Piece::WHITE_KING, "white_king.png"),
        (Piece::BLACK_KING, "black_king.png"),
    ].into_iter().map(|(piece, filename)| {
        let image = image::open(filename).unwrap();
        let image = image.into_rgba8();
        let mut texture = texture_creator.create_texture_static(
            PixelFormatEnum::ARGB8888, 
            image.width(), 
            image.height()
        ).unwrap();
        texture.update(None, &image, (image.width()*4) as usize).unwrap();
        texture.set_blend_mode(render::BlendMode::Blend);
        (piece, texture)
    }).collect::<HashMap<_, _>>(); 

    let mut board = Board::new();

    let mut mouse_click: Option<(bool, i32, i32)> = None;
    let mut mouse_pos: (i32, i32) = (0, 0);

    let mut checkmate: Option<PieceColor> = None;

    let mut log = Vec::new();

    let player_color = if let Ok(button) = sdl2::messagebox::show_message_box(
        messagebox::MessageBoxFlag::INFORMATION, 
        &[
            messagebox::ButtonData {
                flags: messagebox::MessageBoxButtonFlag::NOTHING,
                button_id: 0,
                text: "White",
            },
            messagebox::ButtonData {
                flags: messagebox::MessageBoxButtonFlag::NOTHING,
                button_id: 1,
                text: "Black",
            }
        ], 
        "Choose a color", 
        "Choose the color you want to play as.", 
        None, 
        None
    ) {
        match button {
            messagebox::ClickedButton::CloseButton => PieceColor::White,
            messagebox::ClickedButton::CustomButton(button_data) => match button_data.button_id {
                0 => PieceColor::White,
                1 => PieceColor::Black,
                _ => unreachable!()
            },
        }
    } else { PieceColor::White }; 

    let mut human_player = human_player::HumanPlayer::new(player_color);

    let mut computer_player = computer_player::ComputerPlayer::new(!player_color);
    
    canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
    canvas.set_draw_color(Color::RGB(127, 127, 127));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut start;
    'running: loop {
        start = std::time::Instant::now();
        mouse_click = None;
        canvas.set_draw_color(Color::RGB(127, 127, 127));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::MouseButtonDown { timestamp, window_id, which, mouse_btn, clicks, x, y } => {
                    if mouse_btn == mouse::MouseButton::Left {
                        mouse_click = Some((true, x, y))
                    }
                },
                Event::MouseButtonUp { timestamp, window_id, which, mouse_btn, clicks, x, y } => {
                    if mouse_btn == mouse::MouseButton::Left {
                        mouse_click = Some((false, x, y))
                    }
                }
                Event::MouseMotion { timestamp, window_id, which, mousestate, x, y, xrel, yrel } => {
                    mouse_pos = (x, y);
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        {
            if board.side_to_move() == player_color {
                if !human_player.in_turn() {
                    human_player.begin_turn(&board);
                    if human_player.move_count() == 0 && human_player.in_check() {
                        checkmate.replace(player_color);
                    }
                }
                if let Some((down, x, y)) = mouse_click {
                    let (rank, file) = (y / 80, x / 80);
                    if (0..8).contains(&rank) && (0..8).contains(&file) {
                        let piece_pos = if player_color.is_white() {
                            Position::new((7-rank) as u8, file as u8)
                        } else {
                            Position::new(rank as u8, file as u8)
                        };
                        if down {
                            human_player.set_start_position(piece_pos);
                        } else if !down {
                            if human_player.set_target_position(piece_pos) {
                                if human_player.needs_promotion_choice() {
                                    unimplemented!();
                                }
                                human_player.finish_turn(&mut board);
                            } else {
                                human_player.cancel_move();
                            }
                        }
                    } else if !down {
                        human_player.cancel_move();
                    }
                }
            } else {
                let start = std::time::Instant::now();
                if let Some(_move) = computer_player.play(&mut board) {
                    log.push(_move);
                } else {
                    if computer_player.move_count() == 0 && computer_player.in_check() {
                        checkmate.replace(!player_color);
                    }
                }
                
                println!("Computer move took {} ms", start.elapsed().as_millis());
            }
        }

        for rank in (0u8..8) {
            for file in (0u8..8) {
                let square_rect = Rect::new((file as i32) * 80, (rank as i32) * 80, 80, 80);
                let piece_rect = Rect::new(((file as i32) * 80) + 10, ((rank as i32) * 80) + 10, 60, 60);
                let piece_pos: Position = if player_color.is_white() {
                    (7-rank, file).into()
                } else {
                    (rank, file).into()
                };
                
                if (piece_pos.rank() + piece_pos.file()) % 2 == 0 {
                    canvas.set_draw_color(Color::RGB(255, 213, 128));
                } else {
                    canvas.set_draw_color(Color::RGB(179, 119, 0));
                }
                canvas.fill_rect(square_rect).unwrap();
                let _ = canvas.string(piece_rect.x as i16, piece_rect.y as i16, &format!("{}", piece_pos), Color::RGBA(191, 191, 191, 127));
                
                if board.side_to_move() == player_color {
                    if let Some(old_piece_pos) = human_player.start_position() {
                        if human_player.can_move_to(piece_pos) {
                            let center = square_rect.center();
                            
                            canvas.filled_circle(center.x as i16, center.y as i16, 30, Color::RGBA(0, 255, 0, 127)).unwrap();
                        }
                    }
                }

                if square_rect.contains_point(mouse_pos) {
                    canvas.set_draw_color(Color::RGBA(255, 255, 255, 63));
                    canvas.fill_rect(square_rect).unwrap();
                }

                if let Some(piece) = board.get(piece_pos) {
                    if board.side_to_move() != player_color || human_player.start_position().is_none_or(|pos| pos != piece_pos) {
                        let texture = textures.get(&piece).unwrap();
                        canvas.copy(texture, None, piece_rect).unwrap();
                    }
                }
            }
        }

        if board.side_to_move() == player_color {
            if let Some(piece_pos) = human_player.start_position() {
                if let Some(piece) = board.get(piece_pos) {
                    let piece_rect = Rect::new(mouse_pos.0 - 30, mouse_pos.1 - 30, 60, 60);
                    let texture = textures.get(&piece).unwrap();
                    canvas.copy(texture, None, piece_rect).unwrap();
                }
            }
        }

        {
            let (max_width, max_height) = (300, 200);
            canvas.set_draw_color(Color::BLACK);
            canvas.fill_rect(Rect::new(640, 0, max_width+20, max_height+20)).unwrap();
            
            let mut line = 0i16;
            for move_result in log.iter().rev().take(20).rev() {
                let string = match move_result.kind() {
                    MoveKind::Basic => {
                        let (or, of) = move_result.src().into_chars();
                        let (nr, nf) = move_result.dst().into_chars();
                        format!("{} {}{} -> {}{}", "", or, of, nr, nf)
                    },
                    _ => {"".into()}
                };

                canvas.string(645, 5 + (line*10), &string, Color::WHITE).unwrap();
                line += 1;
            }
        }

        canvas.present();

        if let Some(color) = checkmate {
            if let Ok(button) = sdl2::messagebox::show_message_box(
                messagebox::MessageBoxFlag::INFORMATION, 
                &[
                    messagebox::ButtonData {
                        flags: messagebox::MessageBoxButtonFlag::RETURNKEY_DEFAULT,
                        button_id: 0,
                        text: "OK",
                    }
                ], 
                "Checkmate", 
                if color.is_white() {
                    "White wins!"
                } else {
                    "Black wins!"
                }, 
                None, 
                None
            ) {
                match button {
                    messagebox::ClickedButton::CloseButton |
                    messagebox::ClickedButton::CustomButton(messagebox::ButtonData {
                        flags: _,
                        button_id: 0,
                        text: _,
                    }) => {
                        board.reset();
                        checkmate.take();
                    },
                    _ => unreachable!()
                }
            };
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60).saturating_sub(std::time::Instant::now().saturating_duration_since(start)));
    }
}
