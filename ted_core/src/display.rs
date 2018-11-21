use state::State;
use input::Input;

pub trait Display {
    fn show(&mut self, state: &State) -> Result<(), ()>;
    fn getch(&mut self) -> Option<Input>;
}
