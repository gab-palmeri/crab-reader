use druid::{AppDelegate, Code, Env, Event, Handled, KeyEvent, commands::OPEN_FILE};
use std::rc::Rc;

use super::button_functions::{self, go_next, go_prev};
use crate::{
    components::{
        mockup::SortBy,
    },
    utils::ocrmanager,
    CrabReaderState, DisplayMode, ENTERING_READING_MODE, traits::{gui::{GUILibrary, GUIBook}, reader::{BookReading, BookManagement}, note::NoteManagement},
};

pub struct ReadModeDelegate;

impl AppDelegate<CrabReaderState> for ReadModeDelegate {
    fn command(
        &mut self,
        _: &mut druid::DelegateCtx,
        _: druid::Target,
        cmd: &druid::Command,
        data: &mut CrabReaderState,
        _: &Env,
    ) -> Handled {
        match cmd {
            notif if notif.is(ENTERING_READING_MODE) => {
                data.reading = true;
                data.reading_state.enable(Rc::new(
                    data.library
                        .get_selected_book()
                        .unwrap()
                        .get_page_of_chapter(),
                ),
                data.library.get_selected_book().unwrap().get_notes()
            );
                Handled::Yes
            }
            notif if notif.is(ENTERING_READING_MODE) => {
                data.reading = false;
                data.reading_state.disable();
                Handled::Yes
            }
            notif if notif.is(OPEN_FILE) => {
                println!("Opening file!");
                let file = cmd.get_unchecked(OPEN_FILE);

                //get file path
                let path = file.path();

                let selected_book_path = data.library.get_selected_book().unwrap().get_path();

                //split by slash, get last element, split by dot, get first element
                let folder_name = selected_book_path
                    .split("/")
                    .last()
                    .unwrap()
                    .split(".")
                    .next()
                    .unwrap();

                //call ocr on the img path
                let ocr_result = ocrmanager::get_ebook_page(
                    folder_name.to_string(),
                    path.to_str().unwrap().to_string(),
                );

                match ocr_result {
                    Some(ocr_result) => {
                        //move to the found page
                        data.library
                            .get_selected_book_mut()
                            .unwrap()
                            .set_chapter_number(ocr_result.0, true);
                        data.library
                            .get_selected_book_mut()
                            .unwrap()
                            .set_chapter_current_page_number(ocr_result.1);
                    }
                    None => {
                        println!("ERROR: OCR page not found");
                    }
                }
                Handled::Yes
            }
            _ => Handled::No,
        }
    }

    fn event(
        &mut self,
        ctx: &mut druid::DelegateCtx,
        window_id: druid::WindowId,
        event: druid::Event,
        data: &mut CrabReaderState,
        env: &Env,
    ) -> Option<druid::Event> {
        match &event {
            Event::KeyDown(key_event) => {
                let key = key_event.code;

                if !key_event.mods.ctrl() {
                    return Some(event);
                };

                match key {
                    Code::Escape => {
                        handle_esc(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::ArrowLeft => {
                        handle_arrow_left(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::ArrowRight => {
                        handle_arrow_right(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::Tab => {
                        handle_tab(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::Enter | Code::NumpadEnter => {
                        handle_enter(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::KeyF => {
                        handle_f(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::KeyA => {
                        handle_a(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::KeyP => {
                        handle_p(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::KeyT => {
                        handle_t(ctx, window_id, key_event, data, env);
                        None
                    }
                    Code::KeyU => {
                        handle_u(ctx, window_id, key_event, data, env);
                        None
                    }
                    _ => Some(event),
                }
            }
            _ => Some(event),
        }
    }
}

fn handle_arrow_right(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        return;
    }

    if data.reading {
        go_next(data);
        return;
    }

    let Some(idx) = data.library.next_book_idx() else {
        return;
    };

    if let Some(book) = data.library.get_selected_book_mut() {
        book.unselect();
        data.library.unselect_current_book();
    };

    data.library.set_selected_book_idx(idx);
}

fn handle_arrow_left(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        return;
    }

    if data.reading {
        go_prev(data);
        return;
    }

    let Some(idx) = data.library.prev_book_idx() else {
        return;
    };

    if let Some(book) = data.library.get_selected_book_mut() {
        book.unselect();
        data.library.unselect_current_book();
    };

    data.library.set_selected_book_idx(idx);
}

fn handle_esc(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        data.reading_state.is_editing = false;
        return;
    }

    if data.reading {
        data.reading = false;
        return;
    }

    if let Some(book) = data.library.get_selected_book_mut() {
        book.unselect();
        data.library.unselect_current_book();
        return;
    }
}

fn handle_tab(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading {
        if data.reading_state.is_editing {
            button_functions::undo_btn_fn(&mut data.reading_state);
        } else {
            button_functions::edit_btn_fn(
                &mut data.reading_state,
                data.library.get_selected_book().unwrap(),
            );
        }
    }

    if data.display_mode == DisplayMode::Cover {
        data.display_mode = DisplayMode::List;
    } else {
        data.display_mode = DisplayMode::Cover;
    }
}

fn handle_enter(
    ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if let Some(book) = data.library.get_selected_book_mut() {
        book.load_chapter();
        ctx.submit_command(ENTERING_READING_MODE);
    }
}

fn handle_f(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        return;
    }

    if data.reading {
        return;
    }

    data.library.toggle_fav_filter();
}

fn handle_p(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        return;
    }

    if data.reading {
        return;
    }

    let new_sort = match data.library.get_sort_order() {
        SortBy::PercRead => SortBy::PercReadRev,
        SortBy::PercReadRev => SortBy::PercRead,
        _ => SortBy::PercRead,
    };

    data.library.sort_by(new_sort);
}

fn handle_a(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        return;
    }

    if data.reading {
        return;
    }

    let new_sort = match data.library.get_sort_order() {
        SortBy::Author => SortBy::AuthorRev,
        SortBy::AuthorRev => SortBy::Author,
        _ => SortBy::Author,
    };

    data.library.sort_by(new_sort);
}

fn handle_t(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        return;
    }

    if data.reading {
        return;
    }

    let new_sort = match data.library.get_sort_order() {
        SortBy::Title => SortBy::TitleRev,
        SortBy::TitleRev => SortBy::Title,
        _ => SortBy::Title,
    };

    data.library.sort_by(new_sort);
}

fn handle_u(
    _ctx: &mut druid::DelegateCtx,
    _window_id: druid::WindowId,
    _event: &KeyEvent,
    data: &mut CrabReaderState,
    _env: &Env,
) {
    if data.reading_state.is_editing {
        return;
    }

    if data.reading {
        return;
    }

    if let Some(selected_book) = data.library.get_selected_book_mut() {
        let fav = selected_book.is_favorite();
        selected_book.set_favorite(!fav);
    }
}
