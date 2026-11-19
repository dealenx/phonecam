// GUI поверх phonecam: кнопки дёргают скрипт, настройки живут в ~/.config/phonecam/settings.
use gpui::{
    div, prelude::*, px, rgb, App, Application, Div, FontWeight, Context, SharedString,
    Stateful, Window, WindowBounds, WindowOptions, Bounds, size,
};
use std::fs;
use std::process::Command;

const RES: [(&str, &str); 4] = [
    ("авто", ""),
    ("720p", "1280x720"),
    ("1080p", "1920x1080"),
    ("4K", "3840x2160"),
];
const BR: [&str; 5] = ["авто", "8M", "16M", "24M", "40M"];
const FPS: [&str; 3] = ["авто", "15", "30"];
const FACING: [&str; 2] = ["задняя", "фронтальная"];
const CODEC: [&str; 2] = ["h264", "h265"];
const MODE: [(&str, &str); 3] = [
    ("Окно с камерой", "on"),
    ("Виртуальная камера", "vcam"),
    ("ВЫКЛ", "off"),
];

fn config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    std::path::PathBuf::from(home).join(".config/phonecam/settings")
}

#[derive(Clone, Copy)]
struct Settings {
    res: usize,
    br: usize,
    fps: usize,
    facing: usize,
    codec: usize,
    torch: bool,
}

impl Settings {
    fn load() -> Self {
        let mut s = Settings {
            res: 0,
            br: 0,
            fps: 0,
            facing: 0,
            codec: 0,
            torch: false,
        };
        if let Ok(text) = fs::read_to_string(config_path()) {
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else { continue };
                match k {
                    "PC_RES" => {
                        if let Some(i) = RES.iter().position(|(_, val)| *val == v) { s.res = i }
                    }
                    "PC_BR" => {
                        if let Some(i) = BR.iter().position(|val| *val == v) { s.br = i }
                    }
                    "PC_FPS" => {
                        if let Some(i) = FPS.iter().position(|val| *val == v) { s.fps = i }
                    }
                    "PC_FACING" => s.facing = usize::from(v == "front"),
                    "PC_CODEC" => s.codec = usize::from(v == "h265"),
                    "PC_TORCH" => s.torch = v == "1",
                    _ => {}
                }
            }
        }
        s
    }

    fn save(&self) {
        let _ = fs::create_dir_all(config_path().parent().unwrap());
        let text = format!(
            "PC_RES={}\nPC_BR={}\nPC_FPS={}\nPC_FACING={}\nPC_TORCH={}\nPC_CODEC={}\n",
            RES[self.res].1,
            BR[self.br],
            if self.fps == 0 { "0" } else { FPS[self.fps] },
            if self.facing == 0 { "back" } else { "front" },
            usize::from(self.torch),
            CODEC[self.codec],
        );
        let _ = fs::write(config_path(), text);
    }
}
struct Phonecam {
    status: Vec<String>,
    output: Vec<String>,
    busy: bool,
    mode: usize,
    set: Settings,
}

fn lines(s: &str) -> Vec<String> {
    s.lines().map(str::to_string).collect()
}

impl Phonecam {
    fn refresh(&mut self, cx: &mut Context<Self>) {
        self.status = match Command::new("phonecam").arg("status").output() {
            Ok(o) => lines(&String::from_utf8_lossy(&o.stdout)),
            Err(e) => vec![format!("phonecam status не запустился: {e}")],
        };
        let head = self.status.first().map(String::as_str).unwrap_or("");
        self.mode = if head.contains("виртуалка") {
            1
        } else if head.contains("работает") {
            0
        } else {
            2
        };
        cx.notify();
    }

    fn run(&mut self, arg: String, cx: &mut Context<Self>) {
        self.set.save();
        self.busy = true;
        self.output = vec!["...".into()];
        cx.notify();
        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move { Command::new("phonecam").arg(arg).output() })
                .await;
            this.update(cx, |s, cx| {
                s.busy = false;
                s.output = match res {
                    Ok(o) => {
                        let mut t = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        if !o.status.success() {
                            let e = String::from_utf8_lossy(&o.stderr).trim().to_string();
                            if !e.is_empty() {
                                t = format!("{t}\n{e}");
                            }
                        }
                        lines(&t)
                    }
                    Err(e) => vec![format!("запуск не удался: {e}")],
                };
                s.refresh(cx);
            })
            .ok();
        })
        .detach();
    }
}

fn chip(
    cx: &Context<Phonecam>,
    id: String,
    label: String,
    active: bool,
    set: impl Fn(&mut Settings) + Copy + 'static,
) -> Stateful<Div> {
    div()
        .id(SharedString::from(id))
        .px_2()
        .py_1()
        .rounded_md()
        .bg(if active { rgb(0x89b4fa) } else { rgb(0x313244) })
        .hover(|s| s.bg(if active { rgb(0x89b4fa) } else { rgb(0x45475a) }))
        .cursor_pointer()
        .text_color(if active { rgb(0x1e1e2e) } else { rgb(0xcdd6f4) })
        .text_sm()
        .child(label)
        .on_click(cx.listener(move |s, _, _, cx| {
            set(&mut s.set);
            s.set.save();
            cx.notify();
        }))
}

fn row(label: &'static str, children: Vec<impl IntoElement>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(90.))
                .text_sm()
                .text_color(rgb(0xa6adc8))
                .child(label),
        )
        .child(div().flex().gap_1().children(children))
}

fn mode_chip(
    cx: &Context<Phonecam>,
    i: usize,
    label: &'static str,
    arg: &'static str,
    active: bool,
) -> Stateful<Div> {
    div()
        .id(SharedString::from(format!("mode{i}")))
        .px_3()
        .py_2()
        .rounded_md()
        .bg(if active { rgb(0x89b4fa) } else { rgb(0x313244) })
        .hover(|s| s.bg(if active { rgb(0x89b4fa) } else { rgb(0x45475a) }))
        .cursor_pointer()
        .text_color(if active { rgb(0x1e1e2e) } else { rgb(0xcdd6f4) })
        .child(label)
        .on_click(cx.listener(move |s, _, _, cx| {
            s.mode = i;
            s.run(arg.to_string(), cx);
        }))
}

impl Render for Phonecam {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status: Vec<_> = self
            .status
            .iter()
            .map(|l| div().text_sm().text_color(rgb(0xa6adc8)).child(l.clone()))
            .collect();
        let output: Vec<_> = self
            .output
            .iter()
            .map(|l| div().text_sm().text_color(rgb(0xf9e2af)).child(l.clone()))
            .collect();
        let cur = self.set;

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .size_full()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            .child(div().text_lg().font_weight(FontWeight::BOLD).child("phonecam"))
            .child(div().flex().flex_col().gap_1().children(status))
            .child(div().h_1().bg(rgb(0x313244)))
            .child(row(
                "Разрешение",
                RES.iter()
                    .enumerate()
                    .map(|(i, (l, _))| {
                        chip(cx, format!("res{i}"), (*l).to_string(), i == cur.res, move |s| s.res = i)
                    })
                    .collect(),
            ))
            .child(row(
                "Битрейт",
                BR.iter()
                    .enumerate()
                    .map(|(i, l)| {
                        chip(cx, format!("br{i}"), (*l).to_string(), i == cur.br, move |s| s.br = i)
                    })
                    .collect(),
            ))
            .child(row(
                "FPS",
                FPS.iter()
                    .enumerate()
                    .map(|(i, l)| {
                        chip(cx, format!("fps{i}"), (*l).to_string(), i == cur.fps, move |s| s.fps = i)
                    })
                    .collect(),
            ))
            .child(row(
                "Камера",
                FACING
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        chip(cx, format!("facing{i}"), (*l).to_string(), i == cur.facing, move |s| s.facing = i)
                    })
                    .collect(),
            ))
            .child(row(
                "Кодек",
                CODEC
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        chip(cx, format!("codec{i}"), (*l).to_string(), i == cur.codec, move |s| s.codec = i)
                    })
                    .collect(),
            ))
            .child(row(
                "Фонарик",
                ["выкл", "вкл"]
                    .iter()
                    .enumerate()
                    .map(|(i, l)| {
                        chip(cx, format!("torch{i}"), (*l).to_string(), i == usize::from(cur.torch), move |s| s.torch = i == 1)
                    })
                    .collect(),
            ))
            .child(div().h_1().bg(rgb(0x313244)))
            .child(row(
                "Режим",
                MODE
                    .iter()
                    .enumerate()
                    .map(|(i, (l, a))| mode_chip(cx, i, l, a, i == self.mode))
                    .collect(),
            ))
            .child(div().flex().flex_col().gap_1().children(output))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(480.), px(560.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("phonecam".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|cx| {
                    let mut app = Phonecam {
                        status: vec![],
                        output: vec![],
                        busy: false,
                        mode: 0,
                        set: Settings::load(),
                    };
                    app.refresh(cx);
                    app
                })
            },
        )
        .unwrap();
    });
}
