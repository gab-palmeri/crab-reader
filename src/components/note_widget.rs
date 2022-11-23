use druid::{widget::{Label, LineBreaking, TextBox, Either, Flex, List}, Widget, LensExt, WidgetExt, Lens, EventCtx};

use crate::{CrabReaderState, ReadingState, traits::{gui::GUILibrary, note::NoteManagement, reader::BookManagement}, models::note::{BookNotes, Note}};

use super::{buttons::rbtn::RoundedButton, mockup::LibrarySelectedBookLens};

pub struct SelectedBookNotesLens;

impl<B: BookManagement> Lens<B, BookNotes> for SelectedBookNotesLens {
    fn with<V, F: FnOnce(&BookNotes) -> V>(&self, data: &B, f: F) -> V {
        f(data.get_notes())
    }

    fn with_mut<V, F: FnOnce(&mut BookNotes) -> V>(&self, data: &mut B, f: F) -> V {
        f(data.get_notes_mut())
    }
}

pub fn get_notes_list() -> impl Widget<CrabReaderState> {
    let notes = List::new( || {
        let header = Label::new(|note: &Note, _env: &_| note.get_start().to_string())
            .with_text_size(8.0)
            .with_line_break_mode(LineBreaking::WordWrap)
            .with_text_alignment(druid::TextAlignment::Start);

        let content = Label::new(|note: &Note, _env: &_| note.get_text().to_string())
            .with_line_break_mode(LineBreaking::WordWrap)
            .with_text_alignment(druid::TextAlignment::Start);

        let del_btn = RoundedButton::from_text("X")
            .with_on_click(|_ctx: &mut EventCtx, note: &mut Note, _env: &_| {
                println!("Deleting note: {}", note.get_start());
                // chiamare funzione per cancellare la nota
        });

        Flex::column()
            .with_child(header)
            .with_child(content)
            .with_child(del_btn)
    })
    .lens(CrabReaderState::library.then(LibrarySelectedBookLens).then(SelectedBookNotesLens));

    notes
}
/* 


pub fn get_note_widget(start: &String, text: &String) -> impl Widget<CrabReaderState> {
    let notes = Label::new(text.as_str())
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

    let text_clone = text.clone();
    let edit_note_btn = RoundedButton::from_text(
        "Modifica",
    )
    .with_on_click(move |_, data: &mut CrabReaderState, _| {
        let text_str = text_clone.as_str();
        data.reading_state.notes = text_str.to_string();

        data.reading_state.is_editing_notes = true;
    });
    
    let start_clone = start.clone();
    let del_note_btn = RoundedButton::from_text("Rimuovi nota")
    .with_on_click(move |ctx, data: &mut CrabReaderState, _| {
        let start_str = start_clone.as_str();
        data.library.get_selected_book_mut().unwrap().delete_note(&start_str.to_string());
    });

    let undo_note_btn = RoundedButton::from_text("Annulla").with_on_click(|ctx, data: &mut CrabReaderState, _| {
        data.reading_state.is_editing_notes = false;
    });
    
    let start_clone_2 = start.clone();
    
    let save_note_btn = RoundedButton::from_text("Salva").with_on_click(move|ctx, data: &mut CrabReaderState, _| {
        let start_str = start_clone_2.as_str();
        data.library.get_selected_book_mut().unwrap().edit_note(&start_str.to_string(), &data.reading_state.notes);
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
}

*/