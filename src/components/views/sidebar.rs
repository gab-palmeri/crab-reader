use druid::{Widget, UnitPoint, widget::{Flex, Either, Scroll, Label, LineBreaking, TextBox, List, ListIter}, LensExt, WidgetExt, im::Vector, Env, EventCtx};

use crate::{CrabReaderState, components::{buttons::rbtn::RoundedButton, chapter_selector::ChapterSelector, note_widget::{get_notes_list}}, ReadingState, traits::{gui::GUILibrary, note::NoteManagement, reader::{BookReading, BookManagement}}};

pub enum Sidebar {
    LEFT,
    RIGHT
}

impl Sidebar {
    pub fn get(&self) -> impl Widget<CrabReaderState> {
        match self {
            Sidebar::LEFT => left_sidebar_widget(),
            Sidebar::RIGHT => right_sidebar_widget(),
        }
    }
}

fn left_sidebar_widget() -> Flex<CrabReaderState> {
    let btn = RoundedButton::dynamic(|data: &ReadingState, _env: &_| {
        if !data.sidebar_open {
            "Apri selezione capitoli".into()
        } else {
            "Chiudi selezione capitoli".into()
        }
    })
    .with_on_click(|ctx, data: &mut ReadingState, _env| {
        data.sidebar_open = !data.sidebar_open;
    })
    .with_text_size(18.0)
    .align_horizontal(UnitPoint::CENTER)
    .lens(CrabReaderState::reading_state);

    let sidebar_closed = Flex::column();

    let cs = ChapterSelector::new().lens(CrabReaderState::library);
    let sidebar_open = Flex::column().with_child(cs);

    let sidebar = Either::new(
        |data: &CrabReaderState, _env| data.reading_state.sidebar_open,
        Scroll::new(sidebar_open).vertical(),
        sidebar_closed,
    );

    Flex::column()
        .with_child(btn)
        .with_default_spacer()
        .with_flex_child(sidebar, 1.0)
}

fn right_sidebar_widget() -> Flex<CrabReaderState> {
    // list of notes
    let notes = Scroll::new(get_notes_list()).vertical();

    let tb = TextBox::multiline()
        .with_placeholder("Scrivi...")
        .lens(CrabReaderState::reading_state.then(ReadingState::notes))
        .expand_width();
    /*
    let add_note = RoundedButton::from_text("Aggiungi nota")
        .disabled_if(|data: &CrabReaderState, _env: &_| data.library.get_selected_book().unwrap().get_notes().len() > 0)
        .with_on_click(|_, data: &mut CrabReaderState, _| {
            let book = data.library.get_selected_book().unwrap().clone();
            let note = data.reading_state.notes.clone();
            data.library.get_selected_book_mut().unwrap().get_notes_mut().add_note(&book, note);
        });
    */
    Flex::column()
        .with_child(notes)
        .with_flex_spacer(1.0)
        .with_child(tb)
        //.with_child(add_note)
    /* 
    let notes = Label::dynamic(|data: &CrabReaderState, _env: &_| {
        data.library
            .get_selected_book().unwrap()
            .get_current_note()
            .unwrap_or("".to_string())
    })
    .with_line_break_mode(LineBreaking::WordWrap);

    let tb = TextBox::multiline()
        .with_placeholder("Scrivi...")
        .lens(CrabReaderState::reading_state.then(ReadingState::notes))
        .expand_width();

    let notes_either = Either::new(
        |data: &CrabReaderState, _env| data.reading_state.is_editing_notes,
        tb,
        notes,
    );

    let edit_note_btn = RoundedButton::dynamic(
        |data: &CrabReaderState, _env: &_| {
            if data.library.get_selected_book().unwrap().get_current_note().is_none() {
                "Aggiungi nota".into()
            } else {
                "Modifica nota".into()
            }
        },
    )
    .with_on_click(|_, data: &mut CrabReaderState, _| {
        data.reading_state.notes = data
            .library
            .get_selected_book()
            .unwrap()
            .get_current_note()
            .unwrap_or_default();

        data.reading_state.is_editing_notes = true;
    });

    let del_note_btn = RoundedButton::from_text("Rimuovi nota").with_on_click(|ctx, data: &mut CrabReaderState, _| {
        data.library.get_selected_book_mut().unwrap().delete_note();
    });

    let undo_note_btn = RoundedButton::from_text("Annulla").with_on_click(|ctx, data: &mut CrabReaderState, _| {
        data.reading_state.is_editing_notes = false;
    });
    
    let save_note_btn = RoundedButton::from_text("Salva").with_on_click(|ctx, data: &mut CrabReaderState, _| {
        data.library.get_selected_book_mut().unwrap().edit_note(data.reading_state.notes.clone());
        data.reading_state.is_editing_notes = false;
    })
    .disabled_if(
        |data: &CrabReaderState, _env| data.reading_state.notes.is_empty()
    );


    let bottom_bts = Either::new(
        |data: &CrabReaderState, _env| data.reading_state.is_editing_notes,
        Flex::row()
        .with_flex_child(save_note_btn, 5.0)
        .with_flex_spacer(1.0)
        .with_flex_child(undo_note_btn, 5.0),
        Flex::row()
        .with_flex_child(edit_note_btn, 5.0)
        .with_flex_spacer(1.0)
        .with_flex_child(del_note_btn, 5.0),
    );

    Flex::column()
        .with_flex_child(notes_either, 5.0)
        .with_flex_spacer(1.0)
        .with_flex_child(bottom_bts, 1.0)
    */
}
