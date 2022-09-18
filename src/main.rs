use std::convert::TryInto;
use std::sync::Arc;
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

    let (tx_avg, rx_avg) = mpsc::channel();
    // let (tx_data: Sender<f32>, rx_data: Receiver<f32>) = mpsc::channel();
    let (tx_data, rx_data) = mpsc::channel::<Vec<f64>>();

    thread::spawn(move || analogue_input(tx_avg, tx_data));

    let mut app = App::default();
    app.avg_receiver = Some(rx_avg);
    app.data_receiver = Some(rx_data);

    eframe::run_native("DAQmx test", options, Box::new(|_cc| Box::new(app)));
}

fn analogue_input(tx_avg: Sender<f32>, tx_data: Sender<Vec<f64>>) {
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

        tx_data.send(buffer.to_vec());

        let mut avg = 0.0;

        for sample in buffer {
            avg = avg + sample;
        }
        avg = avg / 100.0;

        tx_avg.send(avg as f32);
    }
}

struct App {
    avg_receiver: Option<Receiver<f32>>,
    data_receiver: Option<Receiver<Vec<f64>>>,
    last_now: Instant,
    avg: f32,
    data: [f64; 100],
}

impl Default for App {
    fn default() -> Self {
        Self {
            avg_receiver: None,
            data_receiver: None,
            last_now: Instant::now(),
            avg: 0.0,
            data: [0.0; 100],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = Instant::now();
        let deltaTime = (now - self.last_now).as_secs_f32();
        self.last_now = now;

        egui::CentralPanel::default().show(ctx, |ui| {
            let avg = self.avg_receiver.as_ref().unwrap().try_recv();
            match avg {
                Ok(data) => self.avg = data,
                _ => (),
            };

            ui.heading(format!("Average: {:.2}", self.avg));

            let data = self.data_receiver.as_ref().unwrap().try_recv();
            match data {
                Ok(x) => {
                    self.data = x.try_into().unwrap();
                }
                _ => (),
            }

            strings_ui(ui, &self.data);
        });
    }
}

fn strings_ui(ui: &mut Ui, data: &[f64; 100]) {
    let color = if ui.visuals().dark_mode {
        Color32::from_additive_luminance(196)
    } else {
        Color32::from_black_alpha(240)
    };

    Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();

        let desired_size = ui.available_width() * vec2(1.0, 1.0);
        let (_id, rect) = ui.allocate_space(desired_size);

        let to_screen =
            emath::RectTransform::from_to(Rect::from_x_y_ranges(0.0..=1.0, -5.0..=5.0), rect);

        let mut shapes = vec![];

        let n = data.len();

        let points: Vec<Pos2> = (0..n)
            .map(|i| {
                let t = i as f64 / (n as f64);
                let y = data[i];
                to_screen * pos2(t as f32, y as f32)
            })
            .collect();

        let thickness = 1.0;
        shapes.push(epaint::Shape::line(points, Stroke::new(thickness, color)));
        //   }

        ui.painter().extend(shapes);
    });
}
