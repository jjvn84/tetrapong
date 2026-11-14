//! Central colour palette for the game. Keeping the colours in one place makes
//! it easy to retheme the arena without hunting through every plugin.

use crate::plugins::paddle::PaddleLocation;
use bevy::prelude::*;

pub const BACKGROUND: Color = Color::srgb_u8(11, 16, 32); // #0B1020
pub const BALL: Color = Color::srgb_u8(255, 247, 230); // #FFF7E6
pub const WALL: Color = Color::srgb_u8(71, 85, 105); // #475569

pub const HUD_PANEL: Color = Color::srgba_u8(30, 41, 59, 230); // #1E293B at ~90%

pub const PADDLE_LEFT: Color = Color::srgb_u8(56, 189, 248); // #38BDF8
pub const PADDLE_RIGHT: Color = Color::srgb_u8(244, 114, 182); // #F472B6
pub const PADDLE_BOTTOM: Color = Color::srgb_u8(163, 230, 53); // #A3E635
pub const PADDLE_TOP: Color = Color::srgb_u8(251, 191, 36); // #FBBF24

pub const POWERUP_BADGE: Color = Color::srgba_u8(15, 23, 42, 230); // #0F172A at ~90%
pub const POWERUP_GLYPH: Color = Color::srgb_u8(248, 250, 252); // #F8FAFC

pub const POWERUP_WIDTH: Color = Color::srgb_u8(96, 165, 250); // #60A5FA
pub const POWERUP_PADDLE_SPEED: Color = Color::srgb_u8(192, 132, 252); // #C084FC
pub const POWERUP_BALL_SPEED: Color = Color::srgb_u8(251, 146, 60); // #FB923C
pub const POWERUP_EARN_LIFE: Color = Color::srgb_u8(52, 211, 153); // #34D399
pub const POWERUP_LOSE_LIFE: Color = Color::srgb_u8(248, 113, 113); // #F87171

pub fn paddle_color(location: PaddleLocation) -> Color {
    match location {
        PaddleLocation::Left => PADDLE_LEFT,
        PaddleLocation::Right => PADDLE_RIGHT,
        PaddleLocation::Bottom => PADDLE_BOTTOM,
        PaddleLocation::Top => PADDLE_TOP,
    }
}
