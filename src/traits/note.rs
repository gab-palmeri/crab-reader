use std::collections::HashMap;

pub trait NoteManagement {
    /// get all notes
    fn get_all_notes(&self) -> HashMap<(usize, usize, String), String>;
    /// get note for the current (chapter, page)
    fn get_notes(&self) -> Option<Vec<(String, String)>>;
    fn get_current_note(&self, start: &String) -> Option<(String, String)>;
    /// add note for the current (chapter, page)
    fn add_note(&mut self, note: &String) -> String;
    /// edit a note for the current chapter, page and start
    fn edit_note(&mut self, start: &String, note: &String); 
    /// delete a note for the current chapter, page and start
    fn delete_note(&mut self, start: &String);
    /// delete all notes
    fn delete_all_notes(&mut self);
}