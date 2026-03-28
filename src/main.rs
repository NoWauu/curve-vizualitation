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
    selected_id: Option<usize>
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
        selected_id: None, }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {
    // Make the points interactive: check if the mouse is close to any point and allow dragging
    let mouse_pos = _app.mouse.position();

    // Press/Select
    if _app.mouse.buttons.left().is_down() {
        if _app.keys.mods.shift() {
            let new_point = Point {id: 3, position: mouse_pos, color: WHITE.into_format() };
            _model.points.push(new_point);
        }

        for point in &mut _model.points {
            if mouse_pos.distance(point.position) < 10.0 {
                _model.selected_id = Some(point.id);
            }
        }
    } else {
        // Release
        _model.selected_id = None;
    }

    // Drag
    if let Some(id) = _model.selected_id {
        _model.points[id].position = mouse_pos;
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

    // Run a lerp function on each point with its neighbor
    for i in 1.._model.points.len() {
        draw_point(&draw, &lerp(&_model.points[i-1], &_model.points[i], 0.5));
    }
    
    draw_bezier_curve(&draw, &_model);

    draw.to_frame(app, &frame).unwrap();
}

fn lerp(p0: &Point, p1: &Point, t: f32) -> Point {
    let position = (1.0-t)*p0.position + t*p1.position;
    Point { id: 2, position, color: BLUE.into_format() }
}

fn draw_point(draw: &Draw, point: &Point) {
    draw.ellipse().x_y(point.position.x, point.position.y).radius(5.0).color(point.color);
}

fn get_bezier_point(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    // De Casteljau's Algorithm:
    // 1. Lerp between P0 and P1
    let a = p0.lerp(p1, t);
    // 2. Lerp between P1 and P2
    let b = p1.lerp(p2, t);
    // 3. Final Lerp between the results
    a.lerp(b, t)
}

fn draw_bezier_curve(draw: &Draw, model: &Model) {
    let steps = 100;
    let mut curve_points = Vec::new();

    // 2. Sample the curve at regular intervals of t
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let point = get_bezier_point(
            model.points[0].position, 
            model.points[2].position, // Note: usually P1 is the 'pull' point
            model.points[1].position, 
            t
        );
        curve_points.push(point);
    }

    // 3. Draw the resulting polyline
    draw.polyline()
        .weight(3.0)
        .points(curve_points)
        .color(STEELBLUE);
}