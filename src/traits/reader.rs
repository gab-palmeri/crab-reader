use druid::{text::RichText, im::Vector};

use crate::models::note::BookNotes;

/// trait that describes the book reading functions
pub trait BookReading {
    /// Method that returns the current chapter number
    fn get_chapter_number(&self) -> usize;

    /// Method that set the chapter number
    /// chapter number must be in the range [0, number_of_chapters)
    /// next is true if the chapter number is incremented, false otherwise
    fn set_chapter_number(&mut self, chapter: usize, next: bool);

    /// Method that returns the number of
    /// the last page of the current chapter
    fn get_last_page_number(&self) -> usize;

    /// Method that returns the current page number with respect to the chapter
    fn get_current_page_number(&self) -> usize;

    /// Method that return the current page number with respect to the total number of pages
    fn get_cumulative_current_page_number(&self) -> usize;

    /// Method that set the current page number
    /// you can use it to change page
    /// Example: go back and go forward
    fn set_chapter_current_page_number(&mut self, page: usize);

    /// Method that returns rich text of the current chapter
    fn get_chapter_rich_text(&self) -> RichText;

    /// Method that returns the page as String of the current chapter
    fn get_page_of_chapter(&self) -> String;

    /// Method that returns two pages dealing with two page mode
    fn get_dual_pages(&self) -> (String, String);

    fn get_number_of_chapters(&self) -> usize;
}

/// Trait that describes book management functions
/// not related directly to the reading
pub trait BookManagement {
    /// Method that returns the path of the book
    fn get_path(&self) -> String;

    /// Method that splits the chapter in blocks of const NUMBER_OF_LINES
    /// and returns a vector of strings. Each string is a page of the chapter
    fn split_chapter_in_pages(&self, is_single_view: bool) -> Vector<String>;

    /// Method that edits the text of the current chapter
    fn edit_text<S: Into<Option<String>>>(&mut self, new_text: String, other_new_text: S);

    /// Method that extracts the book's chapters in local files
    fn save_chapters(&self) -> Result<(), Box<dyn std::error::Error>>;

    fn load_chapter(&mut self);

    fn set_favorite(&mut self, favorite: bool);

    fn get_notes(&self) -> &BookNotes;

    fn get_notes_mut(&mut self) -> &mut BookNotes;
}
