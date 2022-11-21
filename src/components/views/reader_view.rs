use druid::{
    widget::{Container, Either, Flex, Label, LineBreaking, Scroll, TextBox},
    Color, FontDescriptor, LensExt, TextAlignment, Widget, WidgetExt,
};

use crate::{
    CrabReaderState, 
    ReadingState,
    MYENV,
    traits::{
        gui::{GUIBook, GUILibrary},
        reader::BookReading,
    },
};

pub enum ReaderView {
    Single,
    SingleEdit,
    Dual,
    DualEdit,
}

impl ReaderView {
    pub fn get_view(&self) -> impl Widget<CrabReaderState> {
        let myenv = MYENV.lock().unwrap();
        let font = myenv.font.clone();
        let font_color = myenv.font_color.clone();

        match self {
            ReaderView::Single => single_view_widget(font, font_color),
            ReaderView::SingleEdit => single_view_edit_widget(font, font_color), // single_view_edit_widget(font, font_color),
            ReaderView::Dual => dual_view_widget(font, font_color),
            ReaderView::DualEdit => dual_view_edit_widget(font, font_color),
        }
    }

    /// Returns a widget with the correct widget to show page(s) in reading or edit mode
    pub fn dynamic_view() -> impl Widget<CrabReaderState> {
        Either::new(
            |data: &CrabReaderState, _env| data.reading_state.single_view,
            Either::new(
                |data: &CrabReaderState, _env| data.reading_state.is_editing,
                ReaderView::SingleEdit.get_view(),
                ReaderView::Single.get_view(),
            ),
            Either::new(
                |data: &CrabReaderState, _env| data.reading_state.is_editing,
                ReaderView::DualEdit.get_view(),
                ReaderView::Dual.get_view(),
            ),
        )
        .center()
        .expand()
    }
}

// single page view for text reader
fn single_view_widget(font: FontDescriptor, font_color: Color) -> Container<CrabReaderState> {
    let inner = Scroll::new(
        Label::dynamic(|data: &CrabReaderState, _env: &_| {
            data.library
                .get_selected_book()
                .unwrap()
                .get_page_of_chapter()
        })
        .with_text_color(font_color)
        .with_font(font)
        .with_text_alignment(TextAlignment::Justified)
        .with_line_break_mode(LineBreaking::WordWrap),
    )
    .vertical();

    Container::new(inner)
}

// single page view for text editing
fn single_view_edit_widget(font: FontDescriptor, font_color: Color) -> Container<CrabReaderState> {
    let tb = TextBox::multiline()
        .with_text_color(font_color)
        .with_font(font)
        .with_placeholder("Text editing is not yet implemented")
        .lens(CrabReaderState::reading_state.then(ReadingState::text_0))
        .expand_width();

    Container::new(Scroll::new(tb).vertical())
}

// dual page view for text reader
fn dual_view_widget(font: FontDescriptor, font_color: Color) -> Container<CrabReaderState> {
    let inner = Flex::row()
        .with_flex_child(
            Scroll::new(
                Label::dynamic(|data: &CrabReaderState, _env: &_| {
                    data.library.get_selected_book().unwrap().get_dual_pages().0
                })
                .with_text_color(font_color.clone())
                .with_font(font.clone())
                .with_text_alignment(TextAlignment::Justified)
                .with_line_break_mode(LineBreaking::WordWrap),
            )
            .vertical(),
            1.0,
        )
        .with_flex_spacer(0.1)
        .with_flex_child(
            Scroll::new(
                Label::dynamic(|data: &CrabReaderState, _env: &_| {
                    data.library.get_selected_book().unwrap().get_dual_pages().1
                })
                .with_text_color(font_color)
                .with_font(font)
                .with_text_alignment(TextAlignment::Justified)
                .with_line_break_mode(LineBreaking::WordWrap)
                .expand_width(),
            )
            .vertical(),
            1.0,
        );
    Container::new(inner)
}

// dual page view for text editing
fn dual_view_edit_widget(font: FontDescriptor, font_color: Color) -> Container<CrabReaderState> {
    let text_box_page_0 = TextBox::multiline()
        .with_text_color(font_color.clone())
        .with_font(font.clone())
        .with_placeholder("There is no page here... but you can add one!")
        .lens(CrabReaderState::reading_state.then(ReadingState::text_0));

    let text_box_page_1 = TextBox::multiline()
        .with_text_color(font_color)
        .with_font(font)
        .with_placeholder("There is no page here... but you can add one!")
        .lens(CrabReaderState::reading_state.then(ReadingState::text_1));

    let inner = Flex::row()
        .with_flex_child(Scroll::new(text_box_page_0).vertical(), 1.0)
        .with_flex_spacer(0.1)
        .with_flex_child(Scroll::new(text_box_page_1).vertical(), 1.0);

    Container::new(inner)
}

pub fn title_widget() -> impl Widget<CrabReaderState> {
    Label::dynamic(|data: &CrabReaderState, _env: &_| {
        data.library
            .get_selected_book()
            .unwrap()
            .get_title()
            .to_string()
    })
    .with_line_break_mode(LineBreaking::Clip)
    .with_text_size(32.0)
    .padding(10.0)
    .center()
}

pub fn current_chapter_widget() -> Label<CrabReaderState> {
    Label::dynamic(|data: &CrabReaderState, _env: &_| {
        // + 1
        let display_number = data
            .library
            .get_selected_book()
            .unwrap()
            .get_chapter_number()
            + 1;

        format!("Chapter {}", display_number)
    })
}
