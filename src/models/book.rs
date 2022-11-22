use druid::{im::Vector, image::io::Reader as ImageReader, text::RichText, Data, Lens};
use epub::doc::EpubDoc;
use std::{collections::HashMap, io::Cursor as ImageCursor, rc::Rc, string::String, sync::Arc};

use crate::{
    traits::{
        gui::GUIBook,
        note::NoteManagement,
        reader::{BookManagement, BookReading},
    },
    utils::{
        envmanager::FontSize,
        epub_utils,
        epub_utils::{
            calculate_number_of_pages, edit_chapter, get_cumulative_current_page_number,
            get_number_of_pages, split_chapter_in_vec,
        },
        saveload::{
            delete_all_notes, delete_note, load_data, load_notes, remove_edited_chapter,
            save_favorite, save_note,
        },
        text_descriptor,
    },
    MYENV,
};

const NUMBER_OF_LINES: usize = 8;

/// Struct that models EPUB file
/// Metadata are attributes
#[derive(PartialEq, Clone, Data, Lens)]
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
    cover_img: Arc<Vec<u8>>,
    filtered_out: bool,
    #[data(ignore)]
    notes: HashMap<(usize, usize, String), String>,
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
            cover_img: vec![].into(),
            filtered_out: true,
            notes: HashMap::default(),
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
            load_data(path_str, false).unwrap_or((1, 0, FontSize::MEDIUM.to_f64()));

        let number_of_pages = get_number_of_pages(path_str);

        let cumulative_current_page =
            get_cumulative_current_page_number(path_str, chapter_number, current_page);

        let notes = load_notes(path_str).unwrap_or_default();
        println!("Notes: {:?}", notes);
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
            cover_img: vec![].into(),
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
        );
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
        );
    }

    fn get_chapter_rich_text(&self) -> RichText {
        let mut book = EpubDoc::new(self.path.as_str()).unwrap();
        book.set_current_page(self.chapter_number).unwrap();
        let content = book.get_current_str().unwrap();
        let vec_tagged = html2text::from_read_rich(content.as_bytes(), 50);

        let mut text = String::new();
        // first usize is the starting index of the tag
        // last usize is the ending index of the tag
        // String is the tag
        let mut tags = Vec::<(usize, usize, String)>::new();
        for line in vec_tagged {
            println!("DEBUG: line: {:?}", line);
            for item in line.into_tagged_strings() {
                let starting_index = text.len();
                if item.s.starts_with("*") & item.s.ends_with("*") {
                    text.push_str(item.s.replace("*", "").as_str());
                } else {
                    text.push_str(item.s.as_str());
                }
                let ending_index = text.len() - 1;

                if let Some(tag) = item.tag.get(0) {
                    let tag = match tag {
                        html2text::render::text_renderer::RichAnnotation::Default => "default",
                        //html2text::render::text_renderer::RichAnnotation::Link(link) => "a",
                        html2text::render::text_renderer::RichAnnotation::Link(_link) => "em",
                        //html2text::render::text_renderer::RichAnnotation::Image(link) => "img",
                        html2text::render::text_renderer::RichAnnotation::Emphasis => "em",
                        html2text::render::text_renderer::RichAnnotation::Strong => "strong",
                        //html2text::render::text_renderer::RichAnnotation::Strikeout => "strike",
                        //html2text::render::text_renderer::RichAnnotation::Code => "code",
                        //html2text::render::text_renderer::RichAnnotation::Preformat(boolean) => "preformat",
                        _ => "none",
                    };
                    tags.push((starting_index, ending_index, tag.to_string()));
                }
            }
            text.push_str("\n");
        }
        text_descriptor::get_rich_text(text, tags)
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
        let mut width = 800.0;
        let height = 300.0;
        if !is_single_view {
            width = 400.0;
        }

        epub_utils::split_chapter_in_vec(
            self.path.as_str(),
            None,
            self.chapter_number,
            NUMBER_OF_LINES,
            MYENV.lock().unwrap().font.size,
            width,
            height,
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
            800.0,
            300.0,
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
                );
            }
        }
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
        self.current_page
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

    fn get_cover_image(&self) -> Arc<Vec<u8>> {
        self.cover_img.clone()
    }

    fn is_filtered_out(&self) -> bool {
        self.filtered_out
    }

    fn set_filtered_out(&mut self, filtered_out: bool) {
        self.filtered_out = filtered_out;
    }

    fn set_cover_image(&mut self, cover_image: Vec<u8>) {
        self.cover_img = cover_image.into();
    }

    fn is_favorite(&self) -> bool {
        self.is_favorite
    }
}

impl NoteManagement for Book {
    // get all notes
    fn get_all_notes(&self) -> HashMap<(usize, usize, String), String> {
        self.notes.clone()
    }
    // get notes for the current page
    fn get_notes(&self) -> Option<Vec<(String, String)>> {
        let mut notes = Vec::new();
        for (key, value) in self.notes.iter() {
            if key.0 == self.chapter_number && key.1 == self.current_page {
                notes.push((key.2.clone(), value.clone()));
            }
        }

        match notes.len() {
            0 => None,
            _ => Some(notes),
        }
    }

    fn get_current_note(&self, start: &String) -> Option<(String, String)> {
        for (key, value) in self.notes.iter() {
            if key.0 == self.chapter_number
                && key.1 == self.current_page
                && key.2.as_str() == start.as_str()
            {
                return Some((key.2.clone(), value.clone()));
            }
        }
        None
    }

    fn add_note(&mut self, note: &String) -> String {
        let start = save_note(
            self.path.to_string(),
            self.chapter_number,
            self.get_page_of_chapter(),
            note.into(),
        )
        .unwrap();
        self.notes.insert(
            (self.chapter_number, self.current_page, start.clone()),
            note.into(),
        );
        start
    }

    fn edit_note(&mut self, start: &String, note: &String) {
        let _ = save_note(
            self.path.to_string(),
            self.chapter_number,
            self.get_page_of_chapter(),
            note.into(),
        )
        .unwrap();
        self.notes.insert(
            (self.chapter_number, self.current_page, start.into()),
            note.clone(),
        );
    }
    // delete a note for the current chapter, page and start
    fn delete_note(&mut self, start: &String) {
        if delete_note(self.path.to_string(), self.chapter_number, start.into()).is_ok() {
            self.notes
                .remove(&(self.chapter_number, self.current_page, start.into()));
        }
    }
    // delete all notes
    fn delete_all_notes(&mut self) {
        if delete_all_notes(self.path.to_string()).is_ok() {
            self.notes.clear();
        }
    }
}
