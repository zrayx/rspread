pub enum Command {
    Quit,
    None,
}

impl Command {
    pub fn new() -> Command {
        Command::None
    }
}
