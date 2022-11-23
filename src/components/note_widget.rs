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
        let header = Label::new(|note: &Note, _env: &_| format!("{}...", note.get_start()[0..20].to_string()))
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
            .with_default_spacer()
            .with_child(content)
            .with_default_spacer()
            .with_child(del_btn)
    })
    .lens(CrabReaderState::library.then(LibrarySelectedBookLens).then(SelectedBookNotesLens));

    notes
}
