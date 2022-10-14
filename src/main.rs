use components::book_details::BookDetails;
use components::library::{CoverLibrary, Library, ListLibrary};
use components::mockup;
use components::view_switcher::{SwitcherButton, ViewMode};
use druid::widget::{Either, Flex, Scroll};
use druid::{AppLauncher, Color, Data, Lens, PlatformError, Widget, WidgetExt, WindowDesc};

mod components;

#[derive(Clone, Data, Lens)]
struct CrabReaderState {
    user: UserState,
    library: Library,
    display_mode: ViewMode,
}

impl Default for CrabReaderState {
    fn default() -> Self {
        Self {
            user: UserState::new(),
            library: Library::new(),
            display_mode: ViewMode::Cover,
        }
    }
}

#[derive(Clone, Data)]
struct UserState {
    username: String,
}

impl UserState {
    pub fn new() -> Self {
        Self {
            username: "Username".to_string(),
        }
    }
}

fn book_details_panel() -> impl Widget<CrabReaderState> {
    BookDetails::new()
        .background(Color::GRAY)
        .rounded(10.0)
        .lens(CrabReaderState::library)
}

fn build_ui() -> impl Widget<CrabReaderState> {
    let library_cover = CoverLibrary::new().lens(CrabReaderState::library);
    let library_list = ListLibrary::new().lens(CrabReaderState::library);

    let view_either = Either::new(
        |data: &CrabReaderState, _env| data.display_mode == ViewMode::List,
        library_list.padding(5.0),
        library_cover,
    )
    .background(Color::GRAY)
    .rounded(10.0)
    .padding(10.0);

    let scroll = Scroll::new(view_either).vertical();

    let right_panel = Scroll::new(book_details_panel()).vertical().padding(5.0);
    let right_col = Flex::column()
        .with_child(
            SwitcherButton
                .padding(10.0)
                .expand_width()
                .lens(CrabReaderState::display_mode),
        )
        .with_flex_child(right_panel, 1.0);

    let inner = Flex::row()
        .with_flex_child(scroll, 2.0)
        .with_flex_child(right_col, 1.0);

    Flex::column().with_flex_child(inner, 1.0)
}

fn main() -> Result<(), PlatformError> {
    let mut crab_state = CrabReaderState::default();

    mockup::get_mockup_book_vec().into_iter().for_each(|book| {
        crab_state.library.add_book(book);
    });

    AppLauncher::with_window(WindowDesc::new(build_ui).title("CrabReader")).launch(crab_state)?;
    Ok(())
}
