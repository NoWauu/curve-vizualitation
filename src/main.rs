use nannou::prelude::*;

mod bezier;
mod model;
mod ui;

use model::{ControlPoint, Model, palette_color};

fn main() {
    nannou::app(model_init).event(event).simple_window(view).run();
}

fn model_init(_app: &App) -> Model {
    Model::new()
}

fn event(app: &App, model: &mut Model, _event: Event) {
    let mouse_pos = app.mouse.position();
    let win = app.window_rect();

    if app.mouse.buttons.left().is_down() {
        // Slider takes priority — start or continue a drag
        if model.dragging_slider || ui::hits_slider(win, model.current_t, mouse_pos) {
            model.dragging_slider = true;
            model.current_t = ui::t_from_mouse_x(win, mouse_pos.x);
            return; // don't interact with control points while on the slider
        }

        // Shift+click: add a new point and grab it immediately
        if app.keys.mods.shift() && model.selected_id.is_none() {
            let id = model.next_id;
            model.points.push(ControlPoint {
                id,
                position: mouse_pos,
                color: palette_color(id),
            });
            model.selected_id = Some(id);
            model.next_id += 1;
        }

        // Click on an existing point to select it
        if model.selected_id.is_none() {
            for point in &model.points {
                if mouse_pos.distance(point.position) < 15.0 {
                    model.selected_id = Some(point.id);
                    break;
                }
            }
        }
    } else {
        model.dragging_slider = false;
        model.selected_id = None;
    }

    // Drag the selected point
    if let Some(id) = model.selected_id {
        if let Some(point) = model.points.iter_mut().find(|p| p.id == id) {
            point.position = mouse_pos;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    // Control polygon
    if model.points.len() > 1 {
        for w in model.points.windows(2) {
            draw.line()
                .start(w[0].position)
                .end(w[1].position)
                .color(rgba(0.8f32, 0.2, 0.2, 0.5));
        }
    }

    // Bezier curve
    let positions: Vec<Vec2> = model.points.iter().map(|p| p.position).collect();
    if positions.len() > 1 {
        let curve = bezier::sample_curve(&positions, 100);
        draw.polyline().weight(3.0).points(curve).color(STEELBLUE);

        // Playhead: point on the curve at current_t
        let playhead = bezier::de_casteljau(&positions, model.current_t);
        draw.ellipse()
            .xy(playhead)
            .radius(7.0)
            .color(BLACK);
        draw.ellipse()
            .xy(playhead)
            .radius(5.0)
            .color(WHITE);
    }

    // Control points (drawn on top of curve)
    for point in &model.points {
        draw.ellipse()
            .xy(point.position)
            .radius(5.0)
            .color(point.color);
    }

    ui::draw_influence_graph(&draw, app.window_rect(), &model.points, model.current_t);
    ui::draw_slider(&draw, app.window_rect(), model.current_t);

    draw.to_frame(app, &frame).unwrap();
}
