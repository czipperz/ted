use std::sync::mpsc::*;
use display::Display;
use state::State;
use input::Input;

enum DisplayRequest {
    Show(*const State),
    Getch,
}
enum DisplayResult {
    Show(Result<(), ()>),
    Getch(Option<Input>),
}

pub struct DisplayReceiver<D> {
    display: D,
    receiveRequest: Receiver<DisplayRequest>,
    sendResult: Sender<DisplayResult>,
}

pub struct DisplaySender {
    sendRequest: Sender<DisplayRequest>,
    receiveResult: Receiver<DisplayResult>,
}

pub fn new_display_coordinator<D>(display: D) -> (DisplaySender, DisplayReceiver<D>) {
    let (sendRequest, receiveRequest) = channel();
    let (sendResult, receiveResult) = channel();
    (DisplaySender {
    },
     DisplayReceiver {
         display
     })
}

impl<D: Display> Display for DisplayReceiver<D> {
    fn show(&mut self, state: &State) -> Result<(), ()> {
        unimplemented!()
    }

    fn getch(&mut self) -> Option<Input> {
        unimplemented!()
    }
}

impl Display for DisplaySender {
    fn show(&mut self, state: &State) -> Result<(), ()> {
        unimplemented!()
    }

    fn getch(&mut self) -> Option<Input> {
        unimplemented!()
    }
}

fn f() {
    let _: Box<Send> = Box::new(DisplayCoordinator::new());
}
