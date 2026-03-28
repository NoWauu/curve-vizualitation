use nannou::geom::Point2;
use nannou::prelude::*;

mod models;

struct Point {
    position: Point2,
    color: Rgb,
}

struct Model {
    points: Vec<Point>,
}

fn main() {
    nannou::app(model).event(event).simple_window(view).run();
}

fn model(_app: &App) -> Model {
    Model { points: vec![
        Point { position: Point2::new(100.0, 100.0), color: WHITE.into_format() },
        Point { position: Point2::new(-100.0, -100.0), color: WHITE.into_format() },
        Point { position: Point2::new(50.0, -50.0), color: WHITE.into_format() },
    ] }
}

fn event(_app: &App, _model: &mut Model, _event: Event) {
    // Make the points interactive: check if the mouse is close to any point and allow dragging
    let mouse_pos = _app.mouse.position();
    for point in &mut _model.points {
        // If the mouse is close enough and the mouse button is pressed, select the point for dragging
        if mouse_pos.distance(point.position) < 10.0 {
            // Change the color of the point
            point.color = RED.into_format();
            if _app.mouse.buttons.left().is_down() {
                // Update the position of the selected point to follow the mouse
                point.position = mouse_pos;
            }
        } else {
            // Reset the color if the mouse is not close
            point.color = WHITE.into_format();
        }
    }
}

fn view(app: &App, _model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    

    // Draw points from the model
    for point in &_model.points {
        draw.ellipse().x_y(point.position.x, point.position.y).radius(5.0).color(point.color);
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


    draw.to_frame(app, &frame).unwrap();
}