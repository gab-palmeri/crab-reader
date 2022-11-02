use crate::utils::{epub_utils, saveload, text_descriptor};
use derivative::Derivative;
use druid::image::io::Reader as ImageReader;
use std::io::Cursor as ImageCursor;
use std::rc::Rc;
use std::string::String;
use std::sync::{Arc, RwLock};

/// This trait defines all the methods that a `Book` struct must implement
/// in order to be rendered visually correct in the GUI of the application.
pub trait GUIBook {
    /// Returns the title
    fn get_title(&self) -> String;

    /// Builder pattern for title
    fn with_title(self, title: impl Into<String>) -> Self;

    /// Sets the title for the book
    fn set_title(&mut self, title: impl Into<String>);

    /// Returns the author
    fn get_author(&self) -> String;

    /// Builder pattern for author
    fn with_author(self, author: impl Into<String>) -> Self;

    /// Sets the author for the book
    fn set_author(&mut self, author: impl Into<String>);

    /// Returns the number of pages
    fn get_number_of_pages(&self) -> usize;

    /// Builder pattern for number of pages
    fn with_number_of_pages(self, npages: usize) -> Self;

    /// Sets the number of pages for the book
    fn set_number_of_pages(&mut self, npages: usize);

    /// Returns the number of read pages
    fn get_number_of_read_pages(&self) -> usize;

    /// Builder pattern for number of read pages
    fn with_number_of_read_pages(self, read_pages: usize) -> Self;

    /// Sets the number of read pages for the book
    fn set_number_of_read_pages(&mut self, read_pages: usize);

    /// Returns the index of the book.
    ///
    /// The idx is intended to be the position in the array of the `Library` struct (relax this constraint?)
    fn get_index(&self) -> usize;

    /// Builder pattern for index
    ///
    /// The idx is intended to be the position in the array of the `Library` struct (relax this constraint?)
    fn with_index(self, idx: usize) -> Self;

    /// Sets the index of the book.
    ///
    /// The idx is intended to be the position in the array of the `Library` struct (relax this constraint?)
    fn set_index(&mut self, idx: usize);

    /// Builds the cover image from the cover image data
    fn build_cover(&self) -> Result<Box<[u8]>, String>;

    /// Builds the cover image from the cover image data with the specified size
    fn build_cover_with_size(&self, width: u32, height: u32) -> Result<Box<[u8]>, String>;

    /// Builder pattern for the description (i.e, like a synopsis for the book)
    fn with_description(self, description: impl Into<String>) -> Self;

    /// Sets the description (i.e, like a synopsis for the book)
    fn set_description(&mut self, description: impl Into<String>);

    /// Returns the selected state of the book
    fn is_selected(&self) -> bool;

    /// Sets the book as selected
    fn set_selected(&mut self, selected: bool);

    /// Set the book as selected
    fn select(&mut self);

    /// Set the book as unselected
    fn unselect(&mut self);

    /// Returns the cover of this book
    fn get_cover_image(&self) -> Option<Arc<Vec<u8>>>;

    /// Uh?
    fn get_cover_image_setter(&self) -> Arc<RwLock<Option<Arc<Vec<u8>>>>>;
}

use druid::text::RichText;
use druid::{Data, Lens};
use epub::doc::EpubDoc;

const NUMBER_OF_LINES: usize = 8;
const FONT_SIZE: usize = 12;

/// trait that describes the book reading functions
pub trait BookReading {
    /// Method that returns the current chapter number
    fn get_chapter_number(&self) -> usize;

    /// Method that set the chapter number
    /// chapter number must be in the range [0, number_of_chapters)
    /// next is true if the chapter number is incremented, false otherwise
    fn set_chapter_number(&mut self, chapter: usize, next: bool);

    /// Method that returns the number of
    /// the last page of the current chapter
    fn get_last_page_number(&self) -> usize;

    /// Method that returns the current page number with respect to the chapter
    fn get_current_page_number(&self) -> usize;

    /// Method that return the current page number with respect to the total number of pages
    fn get_cumulative_current_page_number(&self) -> usize;

    /// Method that set the current page number
    /// you can use it to change page
    /// Example: go back and go forward
    fn set_chapter_current_page_number(&mut self, page: usize);

    /// Method that returns the text of the current chapter
    fn get_chapter_text(&self) -> Rc<String>;

    /// Method that returns rich text of the current chapter
    fn get_chapter_rich_text(&self) -> RichText;

    /// Method that returns the page as Rc<String> of the current chapter
    fn get_page_of_chapter(&self) -> Rc<String>;

    /// Method that returns two pages dealing with two page mode
    fn get_dual_pages(&self) -> (Rc<String>, Rc<String>);
}

/// Trait that describes book management functions
/// not related directly to the reading
pub trait BookManagement {
    /// Method that returns the path of the book
    fn get_path(&self) -> String;

    /// Method that splits the chapter in blocks of const NUMBER_OF_LINES
    /// and returns a vector of strings. Each string is a page of the chapter
    fn split_chapter_in_pages(&self) -> Vec<Rc<String>>;

    /// Method that edits the text of the current chapter
    /// new text is already in self.chapter_page_text
    fn edit_text(&mut self, old_text: String);

    /// Method that extracts the book's chapters in local files
    fn save_chapters(&self) -> Result<(), Box<dyn std::error::Error>>;

    fn load_chapter(&mut self);

    fn load_page(&mut self);
}

/// Struct that models EPUB file
/// Metadata are attributes
#[derive(Derivative, Clone, Data, Lens)]
#[derivative(PartialEq)]
pub struct Book {
    chapter_number: usize,
    current_page: usize,
    number_of_pages: usize,
    cumulative_current_page: usize,
    idx: usize,
    selected: bool,
    title: Rc<String>,
    author: Rc<String>,
    lang: Rc<String>,
    path: Rc<String>,
    chapter_text: Rc<String>,
    chapter_page_text: Rc<String>,
    description: Rc<String>,
    #[derivative(PartialEq = "ignore")]
    cover_img: Arc<RwLock<Option<Arc<Vec<u8>>>>>,
}

impl Book {
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

        let (chapter_number, current_page) = saveload::get_page_of_chapter(path_str).unwrap();

        let number_of_pages = epub_utils::get_number_of_pages(path_str);
        let cumulative_current_page =
            epub_utils::get_cumulative_current_page_number(path_str, chapter_number, current_page);
        /*
        // these functions have to be called when the you click to read the book
        let chapter_text = epub_utils::get_chapter_text(&path_str, chapter_number);
        let chapter_page_text = chapter_text[0..200].to_string();
        */

        let mut this = Book {
            title: title.into(),
            author: author.into(),
            lang: lang.into(),
            path: path.into(),
            chapter_number: chapter_number,
            current_page: current_page,
            cumulative_current_page: cumulative_current_page,
            number_of_pages: number_of_pages,
            idx: 0, // How to set early?
            selected: false,
            description: desc.into(),
            chapter_text: Rc::new("".into()),
            chapter_page_text: Rc::new("".into()),
            cover_img: Arc::from(RwLock::from(None)),
        };
        this.load_cover_image();
        this
    }

    fn load_cover_image(&mut self) {
        let path = self.get_path();
        let arc = self.get_cover_image_setter();
        let _title = self.get_title();
        std::thread::spawn(move || {
            let mut epub = EpubDoc::new(path).map_err(|e| e.to_string()).unwrap();
            let cover = epub.get_cover().map_err(|e| e.to_string()).unwrap();
            let reader = ImageReader::new(ImageCursor::new(cover))
                .with_guessed_format()
                .map_err(|e| e.to_string())
                .unwrap();
            let image = reader.decode().map_err(|e| e.to_string()).unwrap();
            let thumbnail = image.thumbnail_exact(150, 250);
            let rgb = thumbnail.to_rgb8().to_vec();
            let mut lock = arc.write().unwrap();
            *lock = Some(Arc::from(rgb));
        });
    }

    pub fn get_lang(&self) -> Rc<String> {
        self.lang.clone()
    }
}

impl BookReading for Book {
    fn get_chapter_number(&self) -> usize {
        self.chapter_number
    }

    fn set_chapter_number(&mut self, chapter: usize, next: bool) {
        self.chapter_number = chapter;
        self.chapter_text = epub_utils::get_chapter_text(self.path.clone().as_str(), chapter);
        self.current_page = if next { 0 } else { self.get_last_page_number() };
        self.chapter_page_text = Rc::clone(&self.split_chapter_in_pages()[self.current_page]);
        self.cumulative_current_page = epub_utils::get_cumulative_current_page_number(
            self.path.as_str(),
            chapter,
            self.current_page,
        );
    }

    fn get_last_page_number(&self) -> usize {
        self.split_chapter_in_pages().len() - 1 as usize
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
        self.chapter_page_text = Rc::clone(&self.split_chapter_in_pages()[page]);
    }

    fn get_chapter_text(&self) -> Rc<String> {
        if self.chapter_text.len() > 0 {
            return self.chapter_text.clone();
        }
        epub_utils::get_chapter_text(self.path.as_str(), self.chapter_number)
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

    fn get_page_of_chapter(&self) -> Rc<String> {
        self.chapter_page_text.clone()
    }

    fn get_dual_pages(&self) -> (Rc<String>, Rc<String>) {
        let page = self.split_chapter_in_pages();

        let odd = self.current_page % 2;
        let left_page = if odd == 0 {
            page.get(self.current_page)
                .unwrap_or(&Rc::new("".into()))
                .clone()
        } else {
            page.get(self.current_page - 1)
                .unwrap_or(&Rc::new("".into()))
                .clone()
        };

        let right_page = if odd == 0 {
            page.get(self.current_page + 1)
                .unwrap_or(&Rc::new("".into()))
                .clone()
        } else {
            page.get(self.current_page)
                .unwrap_or(&Rc::new("".into()))
                .clone()
        };

        (left_page, right_page)
    }
}

impl BookManagement for Book {
    fn get_path(&self) -> String {
        self.path.to_string()
    }

    fn split_chapter_in_pages(&self) -> Vec<Rc<String>> {
        epub_utils::split_chapter_in_vec(
            self.path.as_str(),
            self.get_chapter_text(),
            None,
            NUMBER_OF_LINES,
            FONT_SIZE,
        )
    }

    fn edit_text(&mut self, old_text: String) {
        let new_text = self.chapter_page_text.clone();
        for (old_line, new_line) in old_text.lines().zip(new_text.lines()) {
            if old_line != new_line {
                println!("DEBUG: old_line: {}", old_line);
                println!("DEBUG: new_line: {}", new_line);
                let new_chapter = self
                    .chapter_text
                    .replace(old_text.as_str(), new_text.as_str());
                if let Ok(()) = epub_utils::edit_chapter(
                    self.path.clone().as_str(),
                    self.chapter_number,
                    new_chapter,
                ) {
                    println!("DEBUG: Text edited");
                    self.chapter_text = epub_utils::get_chapter_text(
                        self.path.clone().as_str(),
                        self.chapter_number,
                    );
                    self.chapter_page_text =
                        self.split_chapter_in_pages()[self.current_page].clone();
                }
            }
        }
    }

    fn save_chapters(&self) -> Result<(), Box<dyn std::error::Error>> {
        epub_utils::extract_chapters(&self.path)
    }

    fn load_chapter(&mut self) {
        self.chapter_text = epub_utils::get_chapter_text(self.path.as_str(), self.chapter_number);
    }

    fn load_page(&mut self) {
        self.chapter_page_text = self.split_chapter_in_pages()[self.current_page].clone();
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

    fn get_cover_image(&self) -> Option<Arc<Vec<u8>>> {
        if let Ok(cover) = self.cover_img.read() {
            return (*cover).clone();
        }
        None
    }

    fn get_cover_image_setter(&self) -> Arc<RwLock<Option<Arc<Vec<u8>>>>> {
        self.cover_img.clone()
    }
}
