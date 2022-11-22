use druid::{Data, widget::ListIter, im::Vector};

#[derive(Data, Clone, Debug)]
pub struct Note {
    start: String,
    text: String,
}

impl Note {
    pub fn new(start: String, text: String) -> Note {
        Note { start, text }
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn set_start(&mut self, start: String) {
        self.start = start;
    }

    pub fn get_start(&self) -> &String {
        &self.start
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    pub fn with_text(mut self, text: String) -> Note {
        self.text = text;
        self
    }

    pub fn with_start(mut self, start: String) -> Note {
        self.start = start;
        self
    }
}
