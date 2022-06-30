pub enum Command {
    Quit,
    None,
    ExitEditor,
}

impl Command {
    pub fn new() -> Command {
        Command::None
    }
}
