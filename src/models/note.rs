
use std::{rc::Rc, collections::HashMap};

use druid::{Data, widget::ListIter, im::{Vector}};

use crate::{traits::{note::NoteManagement, reader::{BookReading, BookManagement}}, utils::saveload::{save_note, load_notes, delete_note, delete_all_notes}};

use super::book::{Book, book_derived_lenses::chapter_number};

#[derive(Data, Clone, Debug, PartialEq)]
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

#[derive(Data, Clone, Debug, PartialEq)]
/// A struct that contains all the notes for chapter and page of a book
pub struct BookNotes {
    #[data(ignore)]
    all_notes: HashMap<(usize, usize), Vector<Note>>,
    chapter_page_notes: Vector<Note>
}

impl BookNotes {
    pub fn new() -> BookNotes {
        BookNotes {
            all_notes: HashMap::new(),
            chapter_page_notes: Vector::new()
        }
    }

    pub fn update_current(&mut self, chapter: usize, page: usize) {
        self.chapter_page_notes = self.all_notes.get(&(chapter, page)).unwrap_or(&Vector::new()).clone();
    }

    pub fn with_loading(path: String, chapter: usize, page: usize) -> BookNotes {
        let Ok(all_notes) = load_notes(path) else {
            return BookNotes::default();
        };

        BookNotes { 
            all_notes: all_notes.clone(),
            chapter_page_notes: all_notes.get(&(chapter, page)).unwrap_or(&Vector::new()).clone()
        }
    }
}

impl Default for BookNotes {
    fn default() -> Self {
        BookNotes::new()
    }
}

impl NoteManagement for BookNotes {
    fn get(&self) -> &Vector<Note> {
        &self.chapter_page_notes
    }

    fn get_note(&self, start: &String) -> Option<&Note> {
        self.chapter_page_notes.iter().find(|note| note.get_start() == start)
    }

    fn get_note_mut(&mut self, start: &String) -> Option<&mut Note> {
        self.chapter_page_notes.iter_mut().find(|note| note.get_start() == start)
    }

    fn add_note(&mut self, book: &Book, note: String) -> Option<String> {
        let book_path = book.get_path();

        let chapter = book.get_chapter_number();
        let page = book.get_current_page_number();

        let text = book.get_page_of_chapter();

        let Ok(start) = save_note(book_path, chapter, text, note.clone()) else {
            return None;
        };

        let this_note = Note::new(start.clone(), note);

        self.all_notes.entry((chapter, page)).and_modify(
            |notes| notes.push_back(this_note.clone())
        ).or_insert(Vector::from(vec![this_note.clone()]));

        self.chapter_page_notes.push_back(this_note);

        Some(start)
    }

    fn edit_note(&mut self, book: &Book, start: &String, note: String) {
        let book_path = book.get_path();

        let chapter = book.get_chapter_number();
        let page = book.get_current_page_number();


        let Ok(_) = save_note(book_path, chapter, start.into(), note.clone()) else {
            return;
        };

        let this_note = Note::new(start.clone(), note.clone());

        self.all_notes.entry((chapter, page)).and_modify(
            |notes| {
                let Some(index) = notes.iter().position(|note| note.get_start() == start) else { return; };
                notes[index] = this_note.clone();
            }
        );

        self.chapter_page_notes.iter_mut().find(|n| n.get_start() == start).map(|n| n.set_text(note));
    }

    fn delete_note(&mut self, book: &Book, start: &String) {
        let book_path = book.get_path();

        let chapter = book.get_chapter_number();
        let page = book.get_current_page_number();

        let Ok(_) = delete_note(book_path, chapter, start.into()) else {
            return;
        };

        self.all_notes.entry((chapter, page)).and_modify(
            |notes| {
                notes.retain(|note| note.get_start() != start);
            }
        );

        self.chapter_page_notes.retain(|n| n.get_start() != start);
    }

}

impl ListIter<Note> for BookNotes {
    fn data_len(&self) -> usize {
        self.chapter_page_notes.len()
    }

    fn for_each(&self, mut cb: impl FnMut(&Note, usize)) {
        for (i, note) in self.chapter_page_notes.iter().enumerate() {
            cb(note, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Note, usize)) {
        for (i, note) in self.chapter_page_notes.iter_mut().enumerate() {
            cb(note, i);
        }
    }
}