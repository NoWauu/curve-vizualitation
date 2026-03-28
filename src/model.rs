use nannou::prelude::*;

/// Color palette — each control point gets a unique color cycling through this list.
pub const PALETTE: [Rgb<f32>; 8] = [
    Rgb { red: 1.0, green: 0.45, blue: 0.45, standard: std::marker::PhantomData }, // coral
    Rgb { red: 0.45, green: 1.0, blue: 0.55, standard: std::marker::PhantomData }, // mint
    Rgb { red: 0.4,  green: 0.8,  blue: 1.0,  standard: std::marker::PhantomData }, // sky
    Rgb { red: 1.0,  green: 1.0,  blue: 0.4,  standard: std::marker::PhantomData }, // yellow
    Rgb { red: 1.0,  green: 0.5,  blue: 1.0,  standard: std::marker::PhantomData }, // violet
    Rgb { red: 1.0,  green: 0.7,  blue: 0.3,  standard: std::marker::PhantomData }, // amber
    Rgb { red: 0.35, green: 1.0,  blue: 0.9,  standard: std::marker::PhantomData }, // cyan
    Rgb { red: 0.7,  green: 0.45, blue: 1.0,  standard: std::marker::PhantomData }, // purple
];

pub fn palette_color(index: usize) -> Rgb<f32> {
    PALETTE[index % PALETTE.len()]
}

pub struct ControlPoint {
    pub id: usize,
    pub position: Vec2,
    pub color: Rgb,
}

pub struct Model {
    pub points: Vec<ControlPoint>,
    pub selected_id: Option<usize>,
    pub next_id: usize,
    pub current_t: f32,
    pub dragging_slider: bool,
}

impl Model {
    pub fn new() -> Self {
        Model {
            points: vec![
                ControlPoint { id: 0, position: vec2(100.0, 100.0), color: palette_color(0) },
                ControlPoint { id: 1, position: vec2(-100.0, -100.0), color: palette_color(1) },
                ControlPoint { id: 2, position: vec2(50.0, -50.0), color: palette_color(2) },
            ],
            selected_id: None,
            next_id: 3,
            current_t: 0.0,
            dragging_slider: false,
        }
    }
}
