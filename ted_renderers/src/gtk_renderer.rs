use parking_lot::Mutex;
use std::sync::{Arc, mpsc};
use std::thread;
use ted_core::draw::*;
use ted_core::*;

pub struct GtkRenderer {
    input_receiver: mpsc::Receiver<Input>,
    _thread: thread::JoinHandle<()>,
}

fn run_thread(input_sender: mpsc::Sender<Input>) {
    use gtk::*;
    init().expect("Failed to initialize GTK.");
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Ted");
    window.set_default_size(500, 500);
    let text_view = TextView::new();
    window.add(&text_view);
    window.show_all();

    window.connect_delete_event(|_, _| {
        main_quit();
        Inhibit(false)
    });
    window.connect_key_press_event(move |_, event_key| {
        if event_key.get_keyval() == 65515 {
            // This character is emitted without meaning anything
            return Inhibit(false);
        }
        let state = event_key.get_state();
        let input = Input {
            control: state.intersects(gdk::ModifierType::CONTROL_MASK),
            alt: state.intersects(gdk::ModifierType::MOD1_MASK),
            key: Key::Key(std::char::from_u32(event_key.get_keyval()).unwrap()),
        };
        if input == kbd("q") {
            main_quit();
        }
        input_sender.send(input).unwrap();
        Inhibit(true)
    });
    idle_add(|| {
        Continue(true)
    });

    main();
}

impl GtkRenderer {
    pub fn new() -> Self {
        let (input_sender, input_receiver) = mpsc::channel();
        GtkRenderer {
            input_receiver,
            _thread: thread::spawn(|| run_thread(input_sender)),
        }
    }
}

impl Renderer for GtkRenderer {
    fn show(
        &mut self,
        _layout: &Layout,
        _selected_window: Option<&Arc<Mutex<Window>>>,
        _message: Option<&str>,
    ) -> Result<(), String> {
        unimplemented!()
    }

    fn getch(&mut self) -> Option<Input> {
        self.input_receiver.recv().ok()
    }
}

// impl DrawableRenderer for GtkRenderer {
//     fn erase(&mut self) -> Result<(), String> {
//     }

//     fn putch(&mut self, y: usize, x: usize, ch: Character) -> Result<(), String> {
//     }

//     fn set_attribute(&mut self, y: usize, x: usize, at: Attribute) -> Result<(), String> {
//     }
// }
