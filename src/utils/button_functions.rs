use crate::{
    MYENV,
    models::book::Book,
    utils::{saveload::{save_data}, envmanager::FontSize}, 
    ReadingState, 
    CrabReaderState, 
    traits::{
        gui::{GUIBook, GUILibrary}, 
        reader::{BookReading, BookManagement}, note::NoteManagement
    },
};
use druid::EventCtx;

/// Activate or deactivate editing mode
/// return the new value of is_editing
/// and the new value of attribute text
#[allow(dead_code)]
pub fn edit_btn_fn(
    reading_state: &mut ReadingState,
    book: &Book,
) {
    if !reading_state.is_editing {
        reading_state.is_editing = true;
        if reading_state.single_view {
            reading_state.text_0 = book.get_page_of_chapter();
        } else {
            let (text_0, text_1) = book.get_dual_pages();
            reading_state.text_0 = text_0;
            reading_state.text_1 = text_1;
        }
    } else {
        println!("DEBUG: EDIT BUTTON DISABLED");
    }
}

/// Go to the next or previous page of the book
fn change_page(
    book: &mut Book,
    is_editing: bool,
    single_view: bool,
    next: bool,
) {
    if !is_editing {
        let mut increaser = if single_view { 1 } else { 2 };

        if !next {
            increaser = -increaser;
        }

        let new_page = book.get_current_page_number() as isize + increaser;
        if new_page > book.get_last_page_number() as isize {
            let last_page = book.get_number_of_pages() - 1;
            println!("DEBUG: current page: {}, last page of book: {}", book.get_current_page_number(), last_page);
            if book.get_cumulative_current_page_number() == last_page {
                println!("DEBUG: LAST PAGE, can't go forward");
                return;
            }

            book.set_chapter_number(book.get_chapter_number() + 1, true);
            println!("DEBUG: Last page of chapter, changing chapter");
        } else if new_page < 0 {
            if book.get_chapter_number() == 0 {
                println!("DEBUG: FIRST PAGE, can't go back");
                return;
            }
            book.set_chapter_number(book.get_chapter_number() - 1, false);
            println!("DEBUG: First page of chapter, changing chapter");
        } else {
            book.set_chapter_current_page_number(new_page as usize);
            println!("DEBUG: Changing page of chapter");
        }
        // function to save the page that the user is reading
        save_data(
            book.get_path().to_string(),
            book.get_chapter_number(),
            book.get_current_page_number(),
            book.get_page_of_chapter(),
            FontSize::from(MYENV.lock().unwrap().font.size),
            false,
        )
        .unwrap();
        println!("DEBUG: Chapter: {}", book.get_chapter_number());
    }
}

// function for going to next page
pub fn go_next(data: &mut CrabReaderState) {
    let book = data.library.get_selected_book_mut().unwrap();
    change_page(book, data.reading_state.is_editing, data.reading_state.single_view, true);
}
// function for going to previous page
pub fn go_prev(data: &mut CrabReaderState) {
    let book = data.library.get_selected_book_mut().unwrap();
    change_page(book, data.reading_state.is_editing, data.reading_state.single_view, false);
}

pub fn save_btn_fn(
    ctx: &mut EventCtx,
    reading_state: &mut ReadingState,
    book: &mut Book,
) {
    if reading_state.single_view {
        if reading_state.text_0 != book.get_page_of_chapter() {
            book.edit_text(reading_state.text_0.clone(), None);
        }
    } else {
        let (text_0, text_1) = book.get_dual_pages();
        if reading_state.text_0 != text_0.to_string() || reading_state.text_1 != text_1.to_string() {
            book.edit_text(reading_state.text_0.clone(), Some(reading_state.text_1.clone()));
        }
    }
    let _ = save_data(
        book.get_path(), 
        book.get_chapter_number(), 
        book.get_current_page_number(), 
        book.get_page_of_chapter(), 
        FontSize::from(MYENV.lock().unwrap().font.size), 
        true
    );
    println!("DEBUG: SAVED");
    reading_state.is_editing = false;
    reading_state.text_0 = String::default();
    reading_state.text_1 = String::default();
    ctx.request_paint();
}

pub fn undo_btn_fn(
    reading_state: &mut ReadingState
) {
    reading_state.is_editing = false;
    reading_state.text_0 = String::default();
    reading_state.text_1 = String::default();
}

pub fn page_number_switch_button(reading_state: &mut ReadingState) {
    let old = reading_state.pages_btn_style;
    reading_state.pages_btn_style = (old+1)%3;
}

pub fn change_chapter(book: &mut Book, chapter_number: usize) {
    // change chapter number in book
    book.set_chapter_number(chapter_number, true);
    // save the new reading position
    save_data(
        book.get_path().to_string(),
        book.get_chapter_number(),
        book.get_current_page_number(),
        book.get_page_of_chapter(),
        FontSize::from(MYENV.lock().unwrap().font.size),
        false,
    )
    .unwrap();
}