use derivative::Derivative;
use druid::{
    im::Vector,
    piet::{Error, ImageFormat, PietImage},
    Data, Lens, PaintCtx, RenderContext,
};
use epub::doc::EpubDoc;
use image::io::Reader as ImageReader;
use std::{
    cell::{Ref, RefCell},
    io::Cursor as ImageCursor,
    rc::Rc,
    string::String,
    sync::Arc,
};

use crate::{
    traits::{
        gui::GUIBook,
        reader::{BookManagement, BookReading},
    },
    utils::{
        envmanager::FontSize,
        epub_utils,
        epub_utils::{
            calculate_number_of_pages, edit_chapter, get_cumulative_current_page_number,
            split_chapter_in_vec,
        },
        saveload::{load_data, remove_edited_chapter, save_favorite},
    },
    MYENV,
};

use super::note::BookNotes;

const NUMBER_OF_LINES: usize = 8;
pub const PAGE_WIDTH: f32 = 1000.0;
pub const PAGE_HEIGHT: f32 = 800.0;
/// Struct that models EPUB file
/// Metadata are attributes
#[derive(Derivative, Clone, Data, Lens)]
#[derivative(PartialEq)]
pub struct Book {
    chapter_number: usize,
    current_page: usize,
    number_of_pages: usize,
    number_of_chapters: usize,
    cumulative_current_page: usize,
    idx: usize,
    selected: bool,
    title: Rc<String>,
    author: Rc<String>,
    lang: Rc<String>,
    path: Rc<String>,
    is_favorite: bool,
    chapter_text_split: Vector<String>,
    description: Rc<String>,
    cover_buffer: Arc<Vec<u8>>,
    #[derivative(PartialEq = "ignore")]
    #[data(ignore)]
    cover_image: RefCell<Option<PietImage>>,
    filtered_out: bool,
    notes: BookNotes,
}

impl Book {
    pub fn empty_book() -> Book {
        let e: Rc<String> = String::from("").into();
        Self {
            chapter_number: 0,
            current_page: 0,
            number_of_pages: 0,
            number_of_chapters: 0,
            cumulative_current_page: 0,
            idx: 0,
            selected: false,
            title: Rc::new(String::new()),
            author: e.clone(),
            lang: e.clone(),
            path: e.clone(),
            is_favorite: false,
            chapter_text_split: vec![].into(),
            description: e.clone(),
            cover_buffer: vec![].into(),
            cover_image: None.into(),
            filtered_out: true,
            notes: BookNotes::default(),
        }
    }

    /// Method that instantiates a new Book from a epub file
    /// given its path
    pub fn new(path: impl Into<String>) -> Book {
        let path = path.into();
        let path_str = path.as_str();

        // this function has to be called when the book is first added to the library
        // epub_utils::extract_pages(&path_str).expect("Couldn't extract pages in Book::new()");

        let book_map = epub_utils::get_metadata_of_book(path_str);
        let title = book_map
            .get("title")
            .unwrap_or(&"No title".to_string())
            .to_string();
        let author = book_map
            .get("author")
            .unwrap_or(&"No author".to_string())
            .to_string();
        let lang = book_map
            .get("lang")
            .unwrap_or(&"No language".to_string())
            .to_string();
        let desc = book_map
            .get("desc")
            .unwrap_or(&"No description".to_string())
            .to_string();
        let is_fav = book_map
            .get("favorite")
            .unwrap_or(&"false".to_string())
            .parse::<bool>()
            .unwrap();
        let number_of_chapters = book_map
            .get("chapters")
            .map_or(1, |x| x.parse::<usize>().unwrap_or_default());

        let (chapter_number, current_page, _font_size) =
            load_data(path_str, false).unwrap_or((1, 0, FontSize::SMALL.to_f64()));

        let number_of_pages = match book_map.get("total_pages") {
            Some(x) => x.parse::<usize>().unwrap_or_default(),
            None => calculate_number_of_pages(path_str, 8, MYENV.lock().unwrap().font.size)
                .map_or(0, |(x, _)| x as usize),
        };

        let cumulative_current_page = get_cumulative_current_page_number(
            path_str,
            chapter_number,
            current_page,
            Some(book_map),
        );

        let notes = BookNotes::with_loading(path_str.into(), chapter_number, current_page);

        Book {
            title: title.into(),
            author: author.into(),
            lang: lang.into(),
            path: path.into(),
            chapter_number: chapter_number,
            current_page: current_page,
            number_of_chapters: number_of_chapters,
            cumulative_current_page: cumulative_current_page,
            number_of_pages: number_of_pages,
            idx: 0, // How to set early?
            is_favorite: is_fav,
            selected: false,
            description: desc.into(),
            chapter_text_split: Vector::new(),
            cover_buffer: vec![].into(),
            cover_image: None.into(),
            filtered_out: false,
            notes: notes,
        }
    }

    pub fn get_lang(&self) -> Rc<String> {
        self.lang.clone()
    }

    pub fn get_perc_read(&self) -> f64 {
        let total = self.get_number_of_pages() as f64;
        let read = self.get_number_of_read_pages() as f64;
        (read / total) * 100.0
    }
}

impl BookReading for Book {
    fn get_chapter_number(&self) -> usize {
        self.chapter_number
    }

    fn set_chapter_number(&mut self, chapter: usize, next: bool) {
        self.chapter_number = chapter;

        self.chapter_text_split = self.split_chapter_in_pages(true);
        self.current_page = if next { 0 } else { self.get_last_page_number() };
        self.cumulative_current_page = epub_utils::get_cumulative_current_page_number(
            self.path.as_str(),
            chapter,
            self.current_page,
            None,
        );
        self.notes.update_current(chapter, self.current_page);
    }

    fn calculate_chars_until_current_page(&self) -> usize {
        let mut chars = 0;
        for i in 0..self.chapter_number {
            //get chapter text
            let chapter_text = epub_utils::get_chapter_text(self.path.as_str(), i);
            chars += chapter_text.len();
        }

        let chapter_text = epub_utils::get_chapter_text(self.path.as_str(), self.chapter_number);
        let chapter_pages = epub_utils::split_chapter_in_vec(
            "",
            chapter_text,
            self.chapter_number,
            8,
            16.0,
            PAGE_WIDTH,
            PAGE_HEIGHT,
        );
        for i in 0..self.current_page {
            chars += chapter_pages[i].len();
        }

        chars
    }

    fn get_last_page_number(&self) -> usize {
        self.chapter_text_split.len() - 1 as usize
    }

    fn get_current_page_number(&self) -> usize {
        self.current_page
    }

    fn get_cumulative_current_page_number(&self) -> usize {
        self.cumulative_current_page
    }

    fn set_chapter_current_page_number(&mut self, page: usize) {
        self.current_page = page;
        self.cumulative_current_page = epub_utils::get_cumulative_current_page_number(
            self.path.as_str(),
            self.chapter_number,
            page,
            None,
        );
        self.notes.update_current(self.chapter_number, page);
    }

    fn get_page_of_chapter(&self) -> String {
        //possibile entry point
        self.chapter_text_split
            .get(self.current_page)
            .unwrap()
            .clone()
    }

    fn get_dual_pages(&self) -> (String, String) {
        //possibile entry point

        let odd = self.current_page % 2;
        let left_page = if odd == 0 {
            self.chapter_text_split
                .get(self.current_page)
                .map_or(String::default(), |s| s.clone())
        } else {
            self.chapter_text_split
                .get(self.current_page - 1)
                .map_or(String::default(), |s| s.clone())
        };

        let right_page = if odd == 0 {
            self.chapter_text_split
                .get(self.current_page + 1)
                .map_or(String::default(), |s| s.clone())
        } else {
            self.chapter_text_split
                .get(self.current_page)
                .map_or(String::default(), |s| s.clone())
        };

        (left_page, right_page)
    }

    fn get_number_of_chapters(&self) -> usize {
        self.number_of_chapters
    }
}

impl BookManagement for Book {
    fn get_path(&self) -> String {
        self.path.to_string()
    }

    fn split_chapter_in_pages(&self, is_single_view: bool) -> Vector<String> {

        epub_utils::split_chapter_in_vec(
            self.path.as_str(),
            None,
            self.chapter_number,
            NUMBER_OF_LINES,
            MYENV.lock().unwrap().font.size,
            PAGE_WIDTH,
            PAGE_HEIGHT,
        )
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn edit_text<S: Into<Option<String>>>(&mut self, new_text: String, other_new_text: S) {
        let mut split = self.chapter_text_split.clone();

        if let Some(other_new_text) = other_new_text.into() {
            // two pages
            let odd = self.current_page % 2;
            if odd == 0 {
                // left = x, right = x+1
                split[self.current_page] = new_text;
                split[self.current_page + 1] = other_new_text;
            } else {
                // left = x-1, right = x
                split[self.current_page - 1] = new_text;
                split[self.current_page] = other_new_text;
            }
        } else {
            // one page
            split[self.current_page] = new_text;
        }

        let joined_text = split.into_iter().collect::<String>();

        let _ = edit_chapter(self.path.as_str(), self.chapter_number, joined_text);
        let old_len = self.get_last_page_number() + 1;
        // check if the split's number of pages is the same as before
        self.load_chapter();

        let new_len = split_chapter_in_vec(
            self.path.as_str(),
            None,
            self.chapter_number,
            NUMBER_OF_LINES,
            MYENV.lock().unwrap().font.size,
            PAGE_WIDTH,
            PAGE_HEIGHT,
        )
        .len();
        if new_len != old_len {
            println!("DEBUG: new_len: {}, old_len: {}", new_len, old_len);
            // recalculate pages
            let (total_len, _) = calculate_number_of_pages(
                self.path.as_str(),
                NUMBER_OF_LINES,
                MYENV.lock().unwrap().font.size,
            )
            .unwrap();
            self.number_of_pages = total_len;
            self.cumulative_current_page = epub_utils::get_cumulative_current_page_number(
                self.path.as_str(),
                self.chapter_number,
                self.current_page,
                None,
            );
        }
    }

    fn save_chapters(&self) -> Result<(), Box<dyn std::error::Error>> {
        epub_utils::extract_chapters(&self.path)
    }

    fn load_chapter(&mut self) {
        self.chapter_text_split = self.split_chapter_in_pages(true);
        if self.current_page > self.chapter_text_split.len() - 1 {
            if let Ok((_, index, _)) = load_data(self.get_path(), true) {
                self.current_page = index;
                remove_edited_chapter(self.get_path(), self.chapter_number);
                let result = calculate_number_of_pages(
                    self.path.as_str(),
                    NUMBER_OF_LINES,
                    MYENV.lock().unwrap().font.size,
                );
                self.number_of_pages = result.unwrap().0;
                self.cumulative_current_page = get_cumulative_current_page_number(
                    self.get_path().as_str(),
                    self.chapter_number,
                    self.current_page,
                    None,
                );
            }
        }
    }

    fn load_notes(&mut self) {
        self.notes = BookNotes::with_loading(
            self.path.to_string(),
            self.chapter_number,
            self.current_page,
        );
    }

    fn set_favorite(&mut self, favorite: bool) {
        if self.is_favorite == favorite {
            println!("DEBUG: already set");
            return;
        }

        self.is_favorite = favorite;
        if let Ok(()) = save_favorite(self.path.to_string(), self.is_favorite) {
            println!("DEBUG: saved favorite new status: {}", self.is_favorite);
        } else {
            println!("DEBUG: failed to save favorite");
        }
    }

    fn get_notes(&self) -> &BookNotes {
        &self.notes
    }

    fn get_notes_mut(&mut self) -> &mut BookNotes {
        &mut self.notes
    }
}

impl GUIBook for Book {
    fn get_title(&self) -> String {
        self.title.to_string()
    }

    fn with_title(mut self, title: impl Into<String>) -> Self {
        self.set_title(title);
        self
    }

    fn set_title(&mut self, title: impl Into<String>) {
        self.title = Rc::new(title.into());
    }

    fn get_author(&self) -> String {
        self.author.to_string()
    }

    fn with_author(mut self, author: impl Into<String>) -> Self {
        self.set_author(author);
        self
    }

    fn set_author(&mut self, author: impl Into<String>) {
        self.author = Rc::new(author.into());
    }

    fn get_number_of_pages(&self) -> usize {
        self.number_of_pages
    }

    fn with_number_of_pages(mut self, number_of_pages: usize) -> Self {
        self.set_number_of_pages(number_of_pages);
        self
    }

    fn set_number_of_pages(&mut self, number_of_pages: usize) {
        self.number_of_pages = number_of_pages;
    }

    fn get_number_of_read_pages(&self) -> usize {
        self.cumulative_current_page
    }

    fn with_number_of_read_pages(mut self, read_pages: usize) -> Self {
        self.set_number_of_read_pages(read_pages);
        self
    }

    fn set_number_of_read_pages(&mut self, read_pages: usize) {
        self.current_page = read_pages;
    }

    fn get_index(&self) -> usize {
        self.idx
    }

    fn with_index(mut self, idx: usize) -> Self {
        self.set_index(idx);
        self
    }

    fn set_index(&mut self, idx: usize) {
        self.idx = idx as usize
    }

    fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.set_description(desc);
        self
    }

    fn set_description(&mut self, desc: impl Into<String>) {
        self.description = desc.into().into();
    }

    fn is_selected(&self) -> bool {
        self.selected == true
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    fn select(&mut self) {
        self.set_selected(true);
    }

    fn unselect(&mut self) {
        self.set_selected(false);
    }

    fn build_cover(&self) -> Result<Box<[u8]>, String> {
        self.build_cover_with_size(150, 250)
    }

    fn build_cover_with_size(&self, width: u32, height: u32) -> Result<Box<[u8]>, String> {
        let epub_path = self.get_path();
        let mut epub = EpubDoc::new(epub_path.as_str()).map_err(|e| e.to_string())?;
        let cover = epub.get_cover().map_err(|e| e.to_string())?;
        let reader = ImageReader::new(ImageCursor::new(cover))
            .with_guessed_format()
            .map_err(|e| e.to_string())?;
        let image = reader.decode().map_err(|e| e.to_string())?;
        let thumbnail = image.thumbnail_exact(width, height);
        let rgb = thumbnail.to_rgb8().to_vec();
        Ok(rgb.into())
    }

    fn get_cover_buffer(&self) -> Arc<Vec<u8>> {
        self.cover_buffer.clone()
    }

    fn is_filtered_out(&self) -> bool {
        self.filtered_out
    }

    fn set_filtered_out(&mut self, filtered_out: bool) {
        self.filtered_out = filtered_out;
    }

    fn set_cover_buffer(&mut self, cover_image: Vec<u8>) {
        self.cover_buffer = cover_image.into();
    }

    fn is_favorite(&self) -> bool {
        self.is_favorite
    }

    fn set_cover_image(&self, ctx: &mut PaintCtx) -> Result<(), Error> {
        let mut inner = self.cover_image.borrow_mut();
        let buf = self.get_cover_buffer();
        let img = ctx.make_image(150, 250, &buf, ImageFormat::Rgb)?;
        inner.replace(img);
        Ok(())
    }

    fn get_cover_image(&self) -> Ref<Option<PietImage>> {
        self.cover_image.borrow()
    }
}
