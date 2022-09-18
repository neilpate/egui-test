use std::thread;
use std::thread::spawn;
use std::time;
use std::time::Instant;

use daqmx::{channels::VoltageChannelBuilder, tasks::Task};
use eframe::egui;
use egui::{containers::*, *};

use daqmx::channels::*;
use daqmx::tasks::*;
use daqmx::types::*;

use std::sync;

use std::sync::mpsc::{self, Receiver, Sender};

fn main() {
    let options = eframe::NativeOptions::default();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || analogue_input(tx));

    let mut app = App::default();
    app.receiver = Some(rx);

    eframe::run_native("DAQmx test", options, Box::new(|_cc| Box::new(app)));
}

fn analogue_input(tx: Sender<f32>) {
    let mut task = Task::new("buffered").unwrap();
    let ch1 = VoltageChannelBuilder::new("Sim-PCIe-6320/ai0").unwrap();
    task.create_channel(ch1).unwrap();
    task.configure_sample_clock_timing(
        None,
        1000.0,
        ClockEdge::Rising,
        SampleMode::ContinuousSamples,
        10000,
    )
    .unwrap();

    let mut buffer = [0.0; 100];

    task.start().unwrap();

    loop {
        task.read(
            Timeout::Seconds(1.0),
            DataFillMode::GroupByChannel,
            Some(100),
            &mut buffer[..],
        )
        .unwrap();

        let mut avg = 0.0;

        for sample in buffer {
            avg = avg + sample;
        }
        avg = avg / 100.0;

        tx.send(avg as f32);
    }
}

struct App {
    receiver: Option<Receiver<f32>>,
    last_now: Instant,
    avg: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            receiver: None,
            last_now: Instant::now(),
            avg: 0.0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = Instant::now();
        let deltaTime = (now - self.last_now).as_secs_f32();
        self.last_now = now;

        egui::CentralPanel::default().show(ctx, |ui| {
            let avg = self.receiver.as_ref().unwrap().try_recv();
            match avg {
                Ok(data) => self.avg = data,
                _ => (),
            };

            ui.heading(format!("Average: {:.2}", self.avg));

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
