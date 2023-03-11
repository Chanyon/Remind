use fltk::{
    app,
    window::{ Window, self },
    prelude::{ WidgetExt, WindowExt, GroupExt, WidgetBase, InputExt },
    button::Button,
    input::Input,
    frame::Frame,
    dialog,
};
use fltk_theme::{ WidgetScheme, SchemeType, ThemeType, WidgetTheme };
use std::{ time::{ SystemTime, UNIX_EPOCH }, thread };
use std::sync::mpsc;

#[derive(Clone, Copy)]
enum Message {
    Start,
    Reset,
}

struct RemindApp {
    app: app::App,
    main_win: Window,
    count_time: String,
    msg: String,
    send: app::Sender<Message>,
    receiver: app::Receiver<Message>,
    start_btn: Button,
    input: Input,
}

impl RemindApp {
    pub fn new() -> Self {
        let app = app::App::default().with_scheme(app::Scheme::Gtk);

        let widget_theme = WidgetTheme::new(ThemeType::HighContrast);
        widget_theme.apply();
        let widget_scheme = WidgetScheme::new(SchemeType::Fluent);
        widget_scheme.apply();

        let time = "60";
        let msg: String = "请起立，休息一下！".into();
        let (s, rec) = app::channel::<Message>();

        let mut win = Window::default()
            .with_size(500, 450)
            .with_label("Remind Tool")
            .center_screen();
        win.make_resizable(true);

        let mut frame = Frame::new(130, 130, 240, 40, "");
        frame.set_label("请输入时间/分钟");
        frame.set_label_size(30);

        let mut input = Input::new(130, 180, 240, 60, "");
        input.set_text_size(36);
        input.set_value(time);

        let mut start = Button::new(195, 255, 50, 50, "Start");
        start.set_color(start.color().lighter());
        start.emit(s, Message::Start);

        let mut reset = Button::new(255, 255, 50, 50, "Reset");
        reset.set_color(reset.color().lighter());
        reset.emit(s, Message::Reset);

        win.end();
        win.show();

        RemindApp {
            app,
            main_win: win,
            count_time: time.into(),
            msg,
            send: s,
            receiver: rec,
            start_btn: start,
            input,
        }
    }

    pub fn run(&mut self) {
        while self.app.wait() {
            if let Some(msg) = self.receiver.recv() {
                match msg {
                    Message::Start => {
                        self.count_time = self.input.value();
                        let time = self.count_time.parse::<u64>().unwrap();

                        let (tx, rx) = mpsc::channel();

                        let th = thread::spawn(move || {
                            let start_time = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs();

                            loop {
                                let end_time = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs();

                                if end_time - start_time > time {
                                    while let Err(_e) = tx.send(true) {}
                                    break;
                                }
                            }
                        });

                        // th.join().unwrap();
                        while let Ok(res) = rx.try_recv() {
                            println!("{res}");
                            if res {
                                // self.main_win.set_visible_focus();
                                // self.main_win.clone().center_screen();
                                // self.main_win

                                dialog::message_title("久坐提醒!");
                                let choice = dialog::choice2_default(
                                    &self.msg,
                                    "Quit",
                                    "Continue",
                                    "Cancel"
                                );

                                match choice {
                                    Some(cho) => {
                                        if cho == 0 {
                                            self.app.quit();
                                        } else if cho == 1 {
                                            self.input.set_value(&self.count_time);
                                        } else {
                                            self.input.set_value("0");
                                        }
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                    Message::Reset => {
                        self.count_time = "0".into();
                        self.input.set_value(&self.count_time);
                    }
                }
            }
        }
    }
}

fn main() {
    let mut app = RemindApp::new();
    app.run();
}