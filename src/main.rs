use nannou::prelude::*;

mod bezier;
mod bspline;
mod hermite;
mod model;
mod ui;

use model::{ControlPoint, Model, VisualizationMode, palette_color};

fn main() {
    nannou::app(model_init).event(event).simple_window(view).run();
}

fn model_init(_app: &App) -> Model {
    Model::new()
}

fn event(app: &App, model: &mut Model, event: Event) {
    // Space key: cycle visualization mode
    if let Event::WindowEvent { simple: Some(WindowEvent::KeyPressed(Key::Space)), .. } = event {
        model.mode = match model.mode {
            VisualizationMode::FullBezier => VisualizationMode::PiecewiseSpline,
            VisualizationMode::PiecewiseSpline => VisualizationMode::HermiteSpline,
            VisualizationMode::HermiteSpline => VisualizationMode::BSpline,
            VisualizationMode::BSpline => VisualizationMode::FullBezier,
        };
        return;
    }
    let _event = event;
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
        if app.keys.mods.shift()
            && model.selected_id.is_none()
            && model.selected_tangent_id.is_none()
        {
            let id = model.next_id;
            let default_tangent = if let Some(last) = model.points.last() {
                (mouse_pos - last.position) * 0.5
            } else {
                vec2(80.0, 0.0)
            };
            model.points.push(ControlPoint {
                id,
                position: mouse_pos,
                color: palette_color(id),
            });
            model.tangents.push(default_tangent);
            model.selected_id = Some(id);
            model.next_id += 1;
        }

        // In Hermite mode, check tangent handles before points
        if model.mode == VisualizationMode::HermiteSpline
            && model.selected_id.is_none()
            && model.selected_tangent_id.is_none()
        {
            for (i, point) in model.points.iter().enumerate() {
                if i < model.tangents.len() {
                    let handle_pos = point.position + model.tangents[i];
                    if mouse_pos.distance(handle_pos) < 15.0 {
                        model.selected_tangent_id = Some(point.id);
                        break;
                    }
                }
            }
        }

        // Click on an existing point to select it
        if model.selected_id.is_none() && model.selected_tangent_id.is_none() {
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
        model.selected_tangent_id = None;
    }

    // Drag the selected point
    if let Some(id) = model.selected_id {
        if let Some(point) = model.points.iter_mut().find(|p| p.id == id) {
            point.position = mouse_pos;
        }
    }

    // Drag the selected tangent handle
    if let Some(id) = model.selected_tangent_id {
        if let Some((i, _)) = model.points.iter().enumerate().find(|(_, p)| p.id == id) {
            if i < model.tangents.len() {
                model.tangents[i] = mouse_pos - model.points[i].position;
            }
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let positions: Vec<Vec2> = model.points.iter().map(|p| p.position).collect();
    let win = app.window_rect();

    match model.mode {
        VisualizationMode::FullBezier => {
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
            if positions.len() > 1 {
                let curve = bezier::sample_curve(&positions, 100);
                draw.polyline().weight(3.0).points(curve).color(STEELBLUE);

                // Playhead
                let playhead = bezier::de_casteljau(&positions, model.current_t);
                draw.ellipse().xy(playhead).radius(7.0).color(BLACK);
                draw.ellipse().xy(playhead).radius(5.0).color(WHITE);
            }

            // Control points
            for point in &model.points {
                draw.ellipse().xy(point.position).radius(5.0).color(point.color);
            }

            ui::draw_influence_graph(&draw, win, &model.points, model.current_t, 0, 1);
        }

        VisualizationMode::PiecewiseSpline => {
            let ranges = bezier::piecewise_segment_ranges(positions.len());
            let total_segs = ranges.len();

            // Per-segment: dim control polygon + colored curve
            for (seg_i, &(s, e)) in ranges.iter().enumerate() {
                let seg_color = palette_color(seg_i);
                let seg_pts = &model.points[s..e];
                let seg_pos = &positions[s..e];

                // Dim control polygon
                for w in seg_pts.windows(2) {
                    draw.line()
                        .start(w[0].position)
                        .end(w[1].position)
                        .color(rgba(
                            seg_color.red * 0.35,
                            seg_color.green * 0.35,
                            seg_color.blue * 0.35,
                            0.6f32,
                        ));
                }

                // Segment curve
                if seg_pos.len() > 1 {
                    let curve = bezier::sample_curve(seg_pos, 80);
                    draw.polyline().weight(3.0).points(curve).color(seg_color);
                }
            }

            // Junction rings at shared points between segments
            for &(_, e) in ranges.iter().take(total_segs.saturating_sub(1)) {
                draw.ellipse()
                    .xy(positions[e - 1])
                    .radius(8.0)
                    .no_fill()
                    .stroke_weight(1.5)
                    .stroke(rgba(1.0f32, 1.0, 1.0, 0.6));
            }

            // Control points on top
            for point in &model.points {
                draw.ellipse().xy(point.position).radius(5.0).color(point.color);
            }

            // Playhead
            if positions.len() > 1 {
                let playhead = bezier::evaluate_piecewise(&positions, model.current_t);
                draw.ellipse().xy(playhead).radius(7.0).color(BLACK);
                draw.ellipse().xy(playhead).radius(5.0).color(WHITE);
            }

            // Influence graph for the active segment
            if !ranges.is_empty() {
                let (seg_idx, local_t) = bezier::global_to_local_t(&ranges, model.current_t);
                let (s, e) = ranges[seg_idx];
                ui::draw_influence_graph(
                    &draw, win, &model.points[s..e], local_t, seg_idx, total_segs,
                );
            }
        }

        VisualizationMode::HermiteSpline => {
            let n = positions.len();

            // Tangent handles
            for (i, point) in model.points.iter().enumerate() {
                if i < model.tangents.len() {
                    let handle_pos = point.position + model.tangents[i];
                    let neg_handle = point.position - model.tangents[i];
                    draw.line()
                        .start(neg_handle)
                        .end(handle_pos)
                        .weight(1.0)
                        .color(rgba(1.0f32, 1.0, 0.4, 0.3));
                    // Draggable handle
                    draw.ellipse()
                        .xy(handle_pos)
                        .radius(4.0)
                        .color(rgba(1.0, 1.0, 0.4, 0.8));
                    // Opposite indicator
                    draw.ellipse()
                        .xy(neg_handle)
                        .radius(2.5)
                        .no_fill()
                        .stroke_weight(1.0)
                        .stroke(rgba(1.0f32, 1.0, 0.4, 0.4));
                }
            }

            // Hermite curve
            if n >= 2 && model.tangents.len() >= n {
                let curve = hermite::sample_curve(&positions, &model.tangents, 100);
                draw.polyline()
                    .weight(3.0)
                    .points(curve)
                    .color(rgb(1.0f32, 0.85, 0.3));

                // Playhead
                let playhead =
                    hermite::evaluate_piecewise(&positions, &model.tangents, model.current_t);
                draw.ellipse().xy(playhead).radius(7.0).color(BLACK);
                draw.ellipse().xy(playhead).radius(5.0).color(WHITE);
            }

            // Control points on top
            for point in &model.points {
                draw.ellipse().xy(point.position).radius(5.0).color(point.color);
            }

            // Hermite influence graph for the active segment
            if n >= 2 {
                let n_segs = n - 1;
                let (seg_idx, local_t) = hermite::global_to_local_t(n, model.current_t);
                ui::draw_hermite_influence_graph(
                    &draw,
                    win,
                    seg_idx,
                    n_segs,
                    local_t,
                    model.points[seg_idx].color,
                    model.points[seg_idx + 1].color,
                );
            }
        }

        VisualizationMode::BSpline => {
            // Control polygon (dim)
            if model.points.len() > 1 {
                for w in model.points.windows(2) {
                    draw.line()
                        .start(w[0].position)
                        .end(w[1].position)
                        .color(rgba(0.3f32, 0.7, 1.0, 0.3));
                }
            }

            // B-spline curve (needs >= 4 control points for cubic)
            if positions.len() >= 4 {
                let curve = bspline::sample_curve(&positions, 200);
                draw.polyline()
                    .weight(3.0)
                    .points(curve)
                    .color(rgb(0.3f32, 0.7, 1.0));

                // Playhead
                let playhead = bspline::evaluate(&positions, model.current_t);
                draw.ellipse().xy(playhead).radius(7.0).color(BLACK);
                draw.ellipse().xy(playhead).radius(5.0).color(WHITE);
            }

            // Control points
            for point in &model.points {
                draw.ellipse().xy(point.position).radius(5.0).color(point.color);
            }

            // B-spline basis function graph
            ui::draw_bspline_influence_graph(&draw, win, &model.points, model.current_t);
        }
    }

    // Build per-segment position slices for the derivative graphs
    let graph_segments: Vec<Vec<Vec2>> = match model.mode {
        VisualizationMode::FullBezier => {
            if positions.len() >= 2 { vec![positions.clone()] } else { vec![] }
        }
        VisualizationMode::PiecewiseSpline => {
            bezier::piecewise_segment_ranges(positions.len())
                .into_iter()
                .map(|(s, e)| positions[s..e].to_vec())
                .collect()
        }
        VisualizationMode::HermiteSpline => {
            if positions.len() >= 2 && model.tangents.len() >= positions.len() {
                (0..positions.len() - 1)
                    .map(|i| {
                        let bez = hermite::hermite_to_bezier(
                            positions[i],
                            model.tangents[i],
                            positions[i + 1],
                            model.tangents[i + 1],
                        );
                        bez.to_vec()
                    })
                    .collect()
            } else {
                vec![]
            }
        }
        VisualizationMode::BSpline => {
            bspline::spans_to_bezier(&positions)
        }
    };
    ui::draw_velocity_graph(&draw, win, &graph_segments, model.current_t);
    ui::draw_acceleration_graph(&draw, win, &graph_segments, model.current_t);
    ui::draw_g1_graph(&draw, win, &graph_segments, model.current_t);
    ui::draw_g2_graph(&draw, win, &graph_segments, model.current_t);
    ui::draw_continuity_labels(&draw, win, &graph_segments);
    ui::draw_slider(&draw, win, model.current_t);
    draw.to_frame(app, &frame).unwrap();
}
