extern crate env_logger;
#[macro_use]
extern crate conrod;
extern crate glium;
#[macro_use]
extern crate log;

use conrod::backend::glium::glium::Surface;
use glium::DisplayBuild;
use std::io::prelude::*;

struct Redpitaya {
    socket: std::net::TcpStream,
    started: bool,
}

impl Redpitaya {
    pub fn new(ip: &str, port: u16) -> Redpitaya {
        let socket = match std::net::TcpStream::connect((ip, port)) {
            Ok(socket) => socket,
            Err(_) => panic!("Unable to connect to {}:{}", ip, port),
        };

        Redpitaya {
            socket: socket,
            started: false,
        }
    }

    pub fn aquire_start(&mut self) {
        self.send("ACQ:START");
        self.started = true;
    }

    pub fn aquire_stop(&mut self) {
        self.send("ACQ:STOP");
        self.started = false;
    }

    pub fn acquire_is_started(&self) -> bool {
        self.started
    }

    pub fn aquire_reset(&mut self) {
        self.send("ACQ:RST");
    }

    pub fn get_data(&mut self) -> String {
        self.send("ACQ:SOUR1:DATA?");

        self.receive()
    }

    pub fn generator_start(&mut self) {
        self.send("OUTPUT1:STATE ON");
    }

    pub fn generator_stop(&mut self) {
        self.send("OUTPUT1:STATE OFF");
    }

    pub fn generator_set_form(&mut self, form: &str) {
        self.send(format!("OUTPUT1:FUNC {}", form).as_str())
    }

    fn send(&mut self, command: &str) {
        info!("> {}", command);

        self.socket.write(
            format!("{}\r\n", command).as_bytes()
        );
    }

    fn receive(&mut self) -> String {
        let mut message = String::new();
        let mut reader = std::io::BufReader::new(self.socket.try_clone().unwrap());

        reader.read_line(&mut message);

        let message = message.trim_right_matches("\r\n");

        debug!("< {}", message);

        message.into()
    }
}

pub struct EventLoop {
    ui_needs_update: bool,
    last_update: std::time::Instant,
}

impl EventLoop {
    pub fn new() -> Self {
        EventLoop {
            last_update: std::time::Instant::now(),
            ui_needs_update: true,
        }
    }

    pub fn next(&mut self, display: &glium::Display) -> Vec<glium::glutin::Event> {
        // We don't want to loop any faster than 60 FPS, so wait until it has been at least 16ms
        // since the last yield.
        let last_update = self.last_update;
        let sixteen_ms = std::time::Duration::from_millis(16);
        let duration_since_last_update = std::time::Instant::now().duration_since(last_update);
        if duration_since_last_update < sixteen_ms {
            std::thread::sleep(sixteen_ms - duration_since_last_update);
        }

        // Collect all pending events.
        let mut events = Vec::new();
        events.extend(display.poll_events());

        // If there are no events and the `Ui` does not need updating, wait for the next event.
        if events.is_empty() && !self.ui_needs_update {
            events.extend(display.wait_events().next());
        }

        self.ui_needs_update = false;
        self.last_update = std::time::Instant::now();

        events
    }

    pub fn needs_update(&mut self) {
        self.ui_needs_update = true;
    }
}

widget_ids! {
    struct Ids {
        canvas,
        toggle_oscillo,
        toggle_generator,
        toggle_generator_img,
        plot,
    }
}

struct Application {
    oscillo_started: bool,
    generator_started: bool,
    tx: std::sync::mpsc::Sender<String>,
    rx: std::sync::mpsc::Receiver<String>,
}

impl Application {
    pub fn new(tx: std::sync::mpsc::Sender<String>, rx: std::sync::mpsc::Receiver<String>) -> Application {
        Application {
            oscillo_started: false,
            generator_started: false,
            tx: tx,
            rx: rx,
        }
    }

    pub fn run(&mut self) {
        let display = glium::glutin::WindowBuilder::new()
            .with_title("Redpitaya")
            .build_glium()
            .unwrap();

        let mut ui = conrod::UiBuilder::new([400.0, 200.0])
            .build();

        ui.fonts.insert_from_file("assets/fonts/NotoSans/NotoSans-Regular.ttf")
            .unwrap();

        let ids = Ids::new(ui.widget_id_generator());

        let mut renderer = conrod::backend::glium::Renderer::new(&display)
            .unwrap();

        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        let mut event_loop = EventLoop::new();
        'main: loop {
            for event in event_loop.next(&display) {
                if let Some(event) = conrod::backend::winit::convert(event.clone(), &display) {
                    ui.handle_event(event);
                    event_loop.needs_update();
                }

                match event {
                    glium::glutin::Event::Closed => break 'main,
                    _ => {},
                }
            }

            self.set_widgets(ui.set_widgets(), &ids);

            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map)
                    .unwrap();
                target.finish()
                    .unwrap();
            }
        }

        self.tx.send("oscillo/stop".into());
        self.tx.send("generator/stop".into());
    }

    fn set_widgets(&mut self, ref mut ui: conrod::UiCell, ids: &Ids) {
        use conrod::{Sizeable, Positionable, Colorable, Labelable, Widget};

        let bg_color = conrod::color::rgb(0.2, 0.35, 0.45);

        conrod::widget::Canvas::new()
            .pad(30.0)
            .color(bg_color)
            .set(ids.canvas, ui);

        let label = match self.oscillo_started {
            true => "Stop",
            false => "Run",
        };

        let toggle = conrod::widget::Toggle::new(self.oscillo_started)
            .w_h(100.0, 50.0)
            .mid_right_of(ids.canvas)
            .color(bg_color.plain_contrast())
            .label(label)
            .label_color(bg_color)
            .set(ids.toggle_oscillo, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.tx.send("oscillo/start".into());
            } else {
                self.tx.send("oscillo/stop".into());
            }

            self.oscillo_started = value;
        }

        let toggle = conrod::widget::Toggle::new(self.generator_started)
            .w_h(100.0, 50.0)
            .down_from(ids.toggle_oscillo, 10.0)
            .label("Sin")
            .color(bg_color.plain_contrast())
            .label_color(bg_color)
            .set(ids.toggle_generator, ui);

        if let Some(value) = toggle.last() {
            if value {
                self.tx.send("generator/start".into());
            } else {
                self.tx.send("generator/stop".into());
            }

            self.generator_started = value;
        }

        if self.oscillo_started {
            self.tx.send("oscillo/data".into());
            if let Ok(message) = self.rx.recv() {
                let data: Vec<f64> = message
                    .trim_matches(|c| c == '{' || c == '}')
                    .split(",")
                    .map(|s| s.parse::<f64>().unwrap())
                    .collect();

                let x_min = 0;

                let x_max = data.len();

                let y_min: f64 = *data.iter().min_by(|a, b| {
                    a.partial_cmp(b)
                        .unwrap()
                }).unwrap();

                let y_max: f64 = *data.iter().max_by(|a, b| {
                    a.partial_cmp(b)
                        .unwrap()
                }).unwrap();

                let plot = conrod::widget::PlotPath::new(x_min, x_max, y_min, y_max, |x| {
                    return data[x];
                });

                plot.color(conrod::color::LIGHT_BLUE)
                    .padded_w_of(ids.canvas, 100.0)
                    .h_of(ids.canvas)
                    .top_left_of(ids.canvas)
                    .set(ids.plot, ui);
            }
        }
    }
}

fn main() {
    env_logger::init()
        .unwrap();

    let (redpitaya_tx, redpitaya_rx) = std::sync::mpsc::channel::<String>();
    let (application_tx, application_rx) = std::sync::mpsc::channel::<String>();

    let mut redpitaya = Redpitaya::new("192.168.1.5", 5000);

    std::thread::spawn(move || {
        for message in redpitaya_rx {
            match message.as_str() {
                "oscillo/start" => redpitaya.aquire_start(),
                "oscillo/stop" => redpitaya.aquire_stop(),
                "oscillo/data" => if redpitaya.acquire_is_started() {
                    let data = redpitaya.get_data();

                    application_tx.send(data);
                },
                "generator/start" => redpitaya.generator_start(),
                "generator/stop" => redpitaya.generator_stop(),
                "generator/sinc" => redpitaya.generator_set_form("sine"),
                message => warn!("Invalid action: '{}'", message),
            };
        }
    });

    Application::new(redpitaya_tx, application_rx)
        .run();
}
