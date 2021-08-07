use std::f32::consts::FRAC_1_SQRT_2;
use std::f32::consts::PI;
use std::f32::consts::TAU;

use macroquad::prelude::*;
use ringtactoe::Board;
use ringtactoe::Cell;
use ringtactoe::Win;

const RADIUS: f32 = 300.0;
const CENTER_RADIUS: f32 = 100.0;
const LINE_THICKNESS: f32 = 4.0;
const WIN_LINE_THICKNESS: f32 = LINE_THICKNESS * 2.0;
const CELL_COLOR: Color = WHITE;
// gap between inner circle and ring
const GAP: f32 = 5.0;
const RING_INNER_RADIUS: f32 = CENTER_RADIUS + GAP;
const RING_THICKNESS: f32 = RADIUS - RING_INNER_RADIUS;

// l = r * angle
// angle = l / r
const INNER_GAP_ANGLE: f32 = GAP / (CENTER_RADIUS + GAP);
const OUTER_GAP_ANGLE: f32 = GAP / RADIUS;
const LINE_INNER_RADIUS: f32 = RING_INNER_RADIUS + RING_THICKNESS / 2.0 - WIN_LINE_THICKNESS / 2.0;
const LINE_OUTER_RADIUS: f32 = RING_INNER_RADIUS + RING_THICKNESS / 2.0 + WIN_LINE_THICKNESS / 2.0;
const LINE_INNER_GAP_ANGLE: f32 = GAP / LINE_INNER_RADIUS;
const LINE_OUTER_GAP_ANGLE: f32 = GAP / LINE_OUTER_RADIUS;

const RING_BG: Color = LIME;

const MOVEMENT_THRESHOLD: f32 = 5.0;

fn draw_cell(x: f32, y: f32, rotation: f32, radius: f32, cell: Cell) {
    match cell {
        Cell::None => {}
        Cell::X => {
            // The messy working out for all this nonsense is in `working.heic`.

            let cos = rotation.cos();
            let sin = rotation.sin();

            let off1 = radius * (sin + cos) * FRAC_1_SQRT_2;
            let off2 = radius * (sin - cos) * FRAC_1_SQRT_2;

            draw_line(
                x - off1,
                y - off2,
                x + off1,
                y + off2,
                LINE_THICKNESS,
                CELL_COLOR,
            );

            draw_line(
                x + off2,
                y - off1,
                x - off2,
                y + off1,
                LINE_THICKNESS,
                CELL_COLOR,
            );
        }
        Cell::O => {
            draw_poly_lines(x, y, 100, radius, rotation, LINE_THICKNESS, CELL_COLOR);
        }
    }
}

/// `angle` is the middle of the arc.
fn draw_arc(
    angle: f32,
    inner_arc: f32,
    outer_arc: f32,
    inner_radius: f32,
    outer_radius: f32,
    color: Color,
) {
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    let inner_start = angle - inner_arc / 2.0;
    let outer_start = angle - outer_arc / 2.0;

    let points: Vec<_> = (0..=50)
        .flat_map(|i| {
            let portion = i as f32 / 50.0;

            let inner_angle = inner_start + inner_arc * portion;
            let outer_angle = outer_start + outer_arc * portion;

            [
                vec2(
                    center_x + outer_angle.cos() * outer_radius,
                    center_y + outer_angle.sin() * outer_radius,
                ),
                vec2(
                    center_x + inner_angle.cos() * inner_radius,
                    center_y + inner_angle.sin() * inner_radius,
                ),
            ]
        })
        .collect();

    // TODO: replace this with `array_windows` when it's stabilised.
    for window in points.windows(3) {
        let (a, b, c) = match window {
            [a, b, c] => (a, b, c),
            _ => unreachable!(),
        };

        draw_triangle(*a, *b, *c, color);
    }
}

fn draw_board(board: &Board, rotation: f32) {
    let cell_radius = f32::min(
        LINE_INNER_RADIUS * (TAU / board.ring.len() as f32 - LINE_INNER_GAP_ANGLE) / 2.0 - GAP,
        CENTER_RADIUS * 2.0 / 3.0,
    );

    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    // First, just draw the cell in the middle.
    draw_poly(center_x, center_y, 100, CENTER_RADIUS, 0.0, RING_BG);
    draw_cell(center_x, center_y, 0.0, cell_radius, board.center);

    // Drawing the ring around the outside is a bit more complicated, since macroquad doesn't provide any way of drawing arcs or anything.
    // So instead, we just have to draw all the individual triangles ourselves.
    for (i, cell) in board.ring.into_iter().enumerate() {
        let num_cells = board.ring.len() as f32;
        let angle = rotation + i as f32 / num_cells * TAU;
        let arc = TAU / num_cells;
        let inner_arc = arc - INNER_GAP_ANGLE;
        let outer_arc = arc - OUTER_GAP_ANGLE;

        draw_arc(
            angle,
            inner_arc,
            outer_arc,
            CENTER_RADIUS + GAP,
            RADIUS,
            RING_BG,
        );

        const RADIUS_DIFF: f32 = RADIUS - (CENTER_RADIUS + GAP);
        const CELL_POS_RADIUS: f32 = CENTER_RADIUS + GAP + (RADIUS_DIFF / 2.0);

        draw_cell(
            center_x + CELL_POS_RADIUS * angle.cos(),
            center_y + CELL_POS_RADIUS * angle.sin(),
            angle,
            cell_radius,
            cell,
        );
    }

    for win in board.wins() {
        match win {
            Win::Center { index } => {
                let num_cells = board.ring.len() as f32;

                let angle = rotation + index as f32 / num_cells * TAU;

                let x_off = RADIUS * angle.cos();
                let y_off = RADIUS * angle.sin();

                draw_line(
                    center_x - x_off,
                    center_y - y_off,
                    center_x + x_off,
                    center_y + y_off,
                    WIN_LINE_THICKNESS,
                    RED,
                );
            }
            Win::Ring { index } => {
                let num_cells = board.ring.len() as f32;

                let angle = rotation + (index + 1) as f32 / num_cells * TAU;
                let inner_arc = TAU / num_cells * 3.0 - LINE_INNER_GAP_ANGLE;
                let outer_arc = TAU / num_cells * 3.0 - LINE_OUTER_GAP_ANGLE;

                draw_arc(
                    angle,
                    inner_arc,
                    outer_arc,
                    LINE_INNER_RADIUS,
                    LINE_OUTER_RADIUS,
                    RED,
                );
            }
        }
    }
}

#[macroquad::main("Ring-Tac-Toe")]
async fn main() {
    let mut board = Board::new(8);

    let mut turn = Cell::X;

    let mut rotation = 0.0;
    let mut velocity = 0.0;

    // Becomes `Some` when the the ring is grabbed.
    let mut last_mouse_angle = None;

    let mut mouse_movement = 0.0;
    let mut last_mouse_pos = (0.0, 0.0);

    loop {
        draw_board(&board, rotation);

        let (mut x, mut y) = mouse_position();
        x -= screen_width() / 2.0;
        y -= screen_height() / 2.0;

        // inverse tan is also called arctan, apparently
        let mut angle = f32::atan(y / x);
        // `atan` returns angles in the first quadrant for positive values and angles in the fourth quadrant for negative values.
        // Fix the angle to be in the correct quadrant.
        if x < 0.0 {
            // If `angle` is negative, it will end up as PI - angle, in the second quadrant, which is correct.
            // If `angle` is positive, it will end up as PI + angle, in the third quadrant, which is correct.
            angle += PI;
        }

        if let Some(last_angle) = last_mouse_angle {
            let diff = angle - last_angle;
            if is_mouse_button_released(MouseButton::Left) {
                last_mouse_angle = None;

                // If the mouse was barely moved, we consider it a click.
                if mouse_movement < MOVEMENT_THRESHOLD {
                    // We already know they were clicking the ring, since `last_mouse_angle` was `Some`.

                    // Undo the offset of the ring's rotation
                    angle -= rotation;

                    // Put all of the angles in the 0..TAU range.
                    while angle < 0.0 {
                        angle += TAU;
                    }

                    angle %= TAU;

                    // Figure out which index in the ring the angle corresponds to.
                    let i = f32::round(angle / TAU * board.ring.len() as f32) as u8;

                    // Set the cell.
                    board.ring.set(i, turn);

                    turn = match turn {
                        Cell::X => Cell::O,
                        Cell::O => Cell::X,
                        Cell::None => unreachable!(),
                    }
                } else {
                    // This was a drag, so give the ring the velocity that mouse had when it let go.
                    velocity = diff / get_frame_time();
                }
            } else {
                // The mouse was already pressed, so we need to update the rotation to apply the change in mouse position.
                rotation += diff;
                last_mouse_angle = Some(angle);
            }
        } else {
            if is_mouse_button_pressed(MouseButton::Left) {
                mouse_movement = 0.0;
                last_mouse_pos = (x, y);

                let dist_from_center = f32::sqrt(x.powi(2) + y.powi(2));
                // The click was within the ring, so mark it as grabbed.
                if dist_from_center > CENTER_RADIUS + GAP && dist_from_center < RADIUS {
                    last_mouse_angle = Some(angle);
                }
            } else if is_mouse_button_released(MouseButton::Left) {
                // If the mouse was barely moved, we consider it a click.
                if mouse_movement < MOVEMENT_THRESHOLD {
                    // If this was a click on the ring, `last_mouse_angle` would have been `Some`, so this can only have been a click in the center.
                    let dist_from_center = f32::sqrt(x.powi(2) + y.powi(2));
                    if dist_from_center < CENTER_RADIUS {
                        // They clicked the center.
                        board.center = turn;

                        turn = match turn {
                            Cell::X => Cell::O,
                            Cell::O => Cell::X,
                            Cell::None => unreachable!(),
                        }
                    }
                }
            }
        }

        mouse_movement +=
            f32::sqrt((x - last_mouse_pos.0).powi(2) + (y - last_mouse_pos.1).powi(2));
        last_mouse_pos = (x, y);

        // Don't just use an `else` block so that this also triggers on the frame that the mouse is released and this has just been set to `None`.
        if last_mouse_angle.is_none() {
            rotation += velocity * get_frame_time();
            // Simulate friction
            velocity *= 0.9;
        }

        next_frame().await;
    }
}
