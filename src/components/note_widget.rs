use druid::{widget::{Label, LineBreaking, TextBox, Either, Flex}, Widget, LensExt, WidgetExt};

use crate::{CrabReaderState, ReadingState, traits::{gui::GUILibrary, note::NoteManagement}};

use super::buttons::rbtn::RoundedButton;

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