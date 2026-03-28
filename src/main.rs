use nannou::geom::Point2;
use nannou::prelude::*;

mod models;

struct Point {
    id: usize,
    position: Point2,
    color: Rgb,
}

struct Model {
    points: Vec<Point>,
    selected_id: Option<usize>,
    next_id: usize,
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    Model { 
        points: vec![
            Point { id: 0, position: Point2::new(100.0, 100.0), color: WHITE.into_format() },
            Point { id: 1, position: Point2::new(-100.0, -100.0), color: WHITE.into_format() },
            Point { id: 2 ,position: Point2::new(50.0, -50.0), color: WHITE.into_format() },
        ],
        selected_id: None,
        next_id: 3 }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {
    let mouse_pos = _app.mouse.position();

    if _app.mouse.buttons.left().is_down() {
        // --- ADD NEW POINT ---
        // Use a "just_pressed" style check or a boolean to avoid 
        // spawning 60 points per second while holding shift!
        if _app.keys.mods.shift() && _model.selected_id.is_none() {
            let new_point = Point {
                id: _model.next_id,
                position: mouse_pos,
                color: WHITE.into_format(),
            };
            _model.points.push(new_point);
            _model.selected_id = Some(_model.next_id); // Grab it immediately
            _model.next_id += 1; // Increment for the next one
        }

        // --- SELECT POINT ---
        if _model.selected_id.is_none() {
            for point in &_model.points {
                if mouse_pos.distance(point.position) < 15.0 {
                    _model.selected_id = Some(point.id);
                    break; 
                }
            }
        }
    } else {
        _model.selected_id = None;
    }

    // --- DRAG POINT SAFELY ---
    if let Some(id) = _model.selected_id {
        // Find the point that has this ID
        if let Some(point) = _model.points.iter_mut().find(|p| p.id == id) {
            point.position = mouse_pos;
        }
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    

    // Draw points from the model
    for point in &_model.points {
        draw_point(&draw, &point);
    }

    // Draw a line between the points to visualize the connections
    if _model.points.len() > 1 {
        for i in 0.._model.points.len() - 1 {
            draw.line()
                .start(_model.points[i].position)
                .end(_model.points[i + 1].position)
                .color(RED);
        }
    }
    
    let mut curve_points = Vec::new();
    let resolution = 100; // How many segments make up the curve

    // Extract just the Vec2 positions from your Point structs
    let control_positions: Vec<Vec2> = _model.points.iter().map(|p| p.position).collect();

    if control_positions.len() > 1 {
        for i in 0..=resolution {
            let t = i as f32 / resolution as f32;
            let p = calculate_bezier_recursive(&control_positions, t);
            curve_points.push(p);
        }

        // Draw the "fluid" curve
        draw.polyline()
            .weight(3.0)
            .points(curve_points)
            .color(STEELBLUE);
    }

    draw.to_frame(app, &frame).unwrap();
}

fn lerp(p0: Vec2, p1: Vec2, t: f32) -> Vec2 {
    (1.0-t)*p0 + t*p1
}

fn draw_point(draw: &Draw, point: &Point) {
    draw.ellipse().x_y(point.position.x, point.position.y).radius(5.0).color(point.color);
}

fn calculate_bezier_recursive(points: &[Vec2], t: f32) -> Vec2 {
    // Base case: If we only have one point, that's our position at time t
    if points.len() == 1 {
        return points[0];
    }

    // Create a new list of points by lerping between each neighbor
    let mut new_points = Vec::with_capacity(points.len() - 1);
    for i in 0..points.len() - 1 {
        let p0 = points[i];
        let p1 = points[i + 1];
        // Standard Linear Interpolation: (1-t)P0 + tP1
        new_points.push(lerp(p0, p1, t));
    }

    // Recurse with the smaller list until we reach a single point
    calculate_bezier_recursive(&new_points, t)
}