use bevy::{color::palettes::tailwind, prelude::*};

pub const FONT_SIZE: f32 = 16.0;

pub const FONT_COLOR: Srgba = tailwind::VIOLET_50;
pub const FONT_COLOR_EMPHASIS: Srgba = tailwind::VIOLET_300;
pub const FONT_COLOR_MUTED: Srgba =
    Srgba::new(FONT_COLOR.red, FONT_COLOR.green, FONT_COLOR.blue, 0.6);

pub const NORMAL_BUTTON: Srgba = tailwind::VIOLET_500;
pub const HOVERED_BUTTON: Srgba = tailwind::VIOLET_600;
pub const PRESSED_BUTTON: Srgba = tailwind::VIOLET_700;

pub const CONTAINER_BG: Srgba = Srgba::new(0., 0., 0., 0.8);
