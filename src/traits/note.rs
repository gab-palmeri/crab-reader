use druid::im::Vector;

use crate::models::{note::Note, book::Book};


pub trait NoteManagement {
    fn get(&self) -> &Vector<Note>;
    /// get a borrow for the current (chapter, page, start) notes
    fn get_note(&self, start: &String) -> Option<&Note>;
    /// get a mutable borrow for the current (chapter, page, start) notes
    fn get_note_mut(&mut self, start: &String) -> Option<&mut Note>;
    
    /// add note for the current (chapter, page), return the Note.start
    fn add_note(&mut self, book: &Book, note: String) -> Option<String>;
    /// edit a note for the current chapter, page and start
    fn edit_note(&mut self, book: &Book, start: &String, note: String); 
    /// delete a note for the current chapter, page and start
    fn delete_note(&mut self, book: &Book, start: &String);
    /// delete notes for the current chapter and page
    fn delete_notes(&mut self, book_path: String, chapter: usize, page: usize);
}