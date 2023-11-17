use fltk::{
    app,
    button::Button,
    dialog,
    frame::Frame,
    input::Input,
    prelude::{GroupExt, InputExt, WidgetBase, WidgetExt, WindowExt},
    window::Window,
};
use fltk_theme::{SchemeType, ThemeType, WidgetScheme, WidgetTheme};

use winapi::{
    ctypes::c_int,
    shared::{minwindef::LPARAM, windef::HWND},
    um::winuser::{
        EnumWindows, FindWindowA, GetWindow, GetWindowTextW, IsWindowVisible, SetWindowPos,
        GW_CHILD, GW_HWNDNEXT, GW_OWNER, HWND_TOPMOST, SWP_SHOWWINDOW,
    },
};

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lParam: LPARAM) -> i32 {
    let target_hwnd = lParam as *mut HWND;
    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }
    let mut has_owner = false;
    let mut current = hwnd;
    current = GetWindow(current, GW_OWNER);
    while current != std::ptr::null_mut() {
        current = GetWindow(current, GW_OWNER);
        has_owner = true;
    }
    if has_owner {
        return 1;
    }
    current = GetWindow(hwnd, GW_CHILD);
    while current != std::ptr::null_mut() {
        let next = GetWindow(current, GW_HWNDNEXT);
        if enum_windows_proc(hwnd, lParam) == 0 {
            return 0;
        }
        current = next;
    }
    *target_hwnd = hwnd;
    0
}

fn find_window(title: &str) -> Option<HWND> {
    let title = title;
    let hwnd = std::ptr::null_mut::<HWND>();

    unsafe {
        EnumWindows(Some(enum_windows_proc), hwnd as LPARAM);
        if hwnd.is_null() {
            println!("null");
            return None;
        }
        let mut text = [0u16; 255];
        GetWindowTextW(*hwnd, text.as_mut_ptr(), text.len() as c_int);
        if std::string::String::from_utf16_lossy(&text).contains(title) {
            return Some(*hwnd);
        } else {
            return None;
        }
    }
}

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

        let time = "30";
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
                        let mut time = self.count_time.parse::<i32>().unwrap() * 60;

                        if time < 0 {
                            dialog::message_default("时间不能小于0!");
                            return;
                        }
                        app::add_timeout3(1.0, move |h| {
                            if time == 0 {
                                app::remove_timeout3(h);
                            }

                            time_update(&mut time, h);
                        });
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

fn time_update(t: &mut i32, h: app::TimeoutHandle) {
    *t -= 1;
    if *t == 0 {
        unsafe {
            let hwnd = FindWindowA(
                "Remind Tool".as_ptr() as *const i8,
                "Remind Tool".as_ptr() as *const i8,
            );
            if hwnd != std::ptr::null_mut() {
                println!("not null.");
            } else {
                println!("null.");
            }
            SetWindowPos(hwnd, HWND_TOPMOST, 600, 300, 500, 450, SWP_SHOWWINDOW);
        }

        let choice = dialog::choice2_default("请起立，休息一下！", "Quit", "Cancel", "");

        match choice {
            Some(cho) => {
                if cho == 0 {
                    app::quit();
                } else {
                }
            }
            None => {}
        }
        app::remove_timeout3(h);
    } else {
        app::repeat_timeout3(1.0, h);
    }
}

fn main() {
    let mut app = RemindApp::new();
    app.run();
}
