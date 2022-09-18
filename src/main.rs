use std::time::Instant;

use eframe::egui;
use egui::{containers::*, *};

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "DAQmx test",
        options,
        Box::new(|_cc| Box::new(App::default())),
    );

    println!("does this happen?");
}

struct App {
    lastNow: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            lastNow: Instant::now(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = Instant::now();
        let deltaTime = (now - self.lastNow).as_secs_f32();
        self.lastNow = now;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("DAQmx test");
            strings_ui(ui);
        });
    }
}

fn strings_ui(ui: &mut Ui) {
    let color = if ui.visuals().dark_mode {
        Color32::from_additive_luminance(196)
    } else {
        Color32::from_black_alpha(240)
    };

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();
        let time = ui.input().time;

        let desired_size = ui.available_width() * vec2(1.0, 0.35);
        let (_id, rect) = ui.allocate_space(desired_size);

        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0), rect);

        let mut shapes = vec![];

        for &mode in &[2, 3, 5] {
            let mode = mode as f64;
            let n = 1200;
            let speed = 1.5;

            let points: Vec<Pos2> = (0..=n)
                .map(|i| {
                    let t = i as f64 / (n as f64);
                    let amp = (time * speed * mode).sin() / mode;
                    let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                    to_screen * pos2(t as f32, y as f32)
                })
                .collect();

            let thickness = 10.0 / mode as f32;
            shapes.push(epaint::Shape::line(points, Stroke::new(thickness, color)));
        }

        ui.painter().extend(shapes);
    });
}
