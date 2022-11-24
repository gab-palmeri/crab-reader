use crate::{MYENV, utils::{envmanager::FontSize, dir_manager::get_edited_books_dir}};

use super::{saveload::{get_chapter_bytes, FileExtension, remove_edited_chapter}, dir_manager::{get_saved_books_dir, get_saved_covers_dir, get_metadata_path}};
use epub::doc::EpubDoc;
use html2text::from_read;
use serde_json::json;
use std::{
    collections::HashMap,
    error,
    fs::{File, OpenOptions},
    io::{BufReader, Cursor, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
};

/// Method to extract metadata from epub file
/// and returns explicit metadata.
/// title: title of the book
/// author: author of the book
/// lang: language of the book
/// chapters: number of chapters in the book as String
/// source: source of the book
/// date: date of the book
/// rights: rights of the book
/// identifier: identifier of the book


fn get_metadata_from_epub(
    book: &EpubDoc<File>,
) -> Result<HashMap<String, String>, Box<dyn error::Error>> {
    for key in book.metadata.keys() {
        println!("DEBUG: {}: {}", key, book.mdata(key).unwrap());
    }
    let mut metadata = HashMap::new();
    metadata.insert(
        "title".to_string(),
        book.mdata("title").unwrap_or("no title".to_string()),
    );

    metadata.insert(
        "author".to_string(),
        book.mdata("creator").unwrap_or("no author".to_string()),
    );

    metadata.insert(
        "lang".to_string(),
        book.mdata("language").unwrap_or("no lang".to_string()),
    );
    /*
    metadata.insert(
        "cover".to_string(),
        String::from_utf8(book.get_cover().unwrap_or(Vec::<u8>::default()))?,
    );
    */

    metadata.insert(
        "source".to_string(),
        book.mdata("source").unwrap_or("no source".to_string()),
    );

    metadata.insert(
        "date".to_string(),
        book.mdata("date").unwrap_or("no date".to_string()),
    );

    metadata.insert(
        "rights".to_string(),
        book.mdata("rights").unwrap_or("no rights".to_string()),
    );

    metadata.insert(
        "identifier".to_string(),
        book.mdata("identifier")
            .unwrap_or("no indetifier".to_string()),
    );

    metadata.insert("chapters".to_string(), book.get_num_pages().to_string());
    metadata.insert("favorite".to_string(), "false".to_string());

    Ok(metadata)
}

/// Method to save the cover of the book as a png file
/// in the path specified.
/// image: String of vec[u8] (as u8) of the cover
/// name: name of the file
/// path: path where to save the cover
/// returns: Result with the path of the cover saved
#[allow(dead_code)]
pub fn save_book_cover(image: &Vec<u8>, name: &String) -> Result<String, Box<dyn error::Error>> {
    // create dir
    let mut path = get_saved_covers_dir();
    std::fs::create_dir_all(&path)?;

    path.push(format!("{}.png", &name));
    let mut file = File::create(&path)?;
    file.write_all(image)?;

    Ok(path.as_os_str().to_str().unwrap().to_string())
}

pub fn edit_chapter(
    path: &str,
    chapter_number: usize,
    text: impl Into<String>,
) -> Result<(), Box<dyn error::Error>> {
    let folder_name = Path::new(path).file_stem().unwrap().to_str().unwrap();
    let mut path_name: PathBuf = get_edited_books_dir().join(folder_name);
    println!("DEBUG: Folder path: {:?}", path_name);
    std::fs::create_dir_all(&path_name)?;
    path_name = path_name.join(format!("page_{}.txt", chapter_number));

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path_name)?;

    file.write_all(text.into().as_bytes())?;

    Ok(())
}

pub fn extract_all(path: &str) -> Result<(), Box<dyn error::Error>> {

    let mut book = EpubDoc::new(path)?;
    let path_name = get_metadata_path(&path.to_string());

    let mut metadata_file = File::create(&path_name).unwrap();
    let metadata_map = get_metadata_from_epub(&book)?;

    let json = json!(metadata_map);
    //json["cover"] = json!(book.get_cover().unwrap_or_default());
    metadata_file
        .write_all(json.to_string().as_bytes())
        .unwrap();
    let len = book.get_num_pages();

    //extract all chapters
    let pool = threadpool::Builder::new().build();

    let arc_book = Arc::new(Mutex::new(book));
    for i in 0..len {
        let this_book = arc_book.clone();
        let this_path = path_name.clone();
        pool.execute(move || {
            let mut locked_book = this_book.lock().unwrap();
            if let Err(error) = locked_book.set_current_page(i) {
                println!("ERROR: {}", error);
                return;
            }
            let chapter = locked_book.get_current_str().unwrap();
            let page_path = this_path.with_file_name(format!("page_{}.html", i));
            let mut file = File::create(page_path).unwrap();
            file.write_all(chapter.as_bytes()).unwrap();
        })
    }

    Ok(())
}

pub fn extract_metadata(path: &str) -> Result<HashMap<String, String>, Box<dyn error::Error>> {
    let path_name = get_metadata_path(&path.to_string());
    let mut metadata_file = File::create(&path_name).unwrap();
    let book = EpubDoc::new(path)?;
    let metadata_map = get_metadata_from_epub(&book)?;

    let json = json!(metadata_map);
    //json["cover"] = json!(book.get_cover().unwrap_or_default());
    metadata_file
        .write_all(json.to_string().as_bytes())
        .unwrap();
    Ok(metadata_map)
}

pub fn extract_chapters(path: &str) -> Result<(), Box<dyn error::Error>> {
    let folder_name = Path::new(path).file_stem().unwrap().to_str().unwrap();
    println!("DEBUG: Folder name: {:?}", folder_name);
    let path_name: PathBuf = get_saved_books_dir().join(folder_name);
    println!("DEBUG: Folder path: {:?}", path_name);
    std::fs::create_dir_all(&path_name)?;

    let mut book = EpubDoc::new(path)?;

    let len = book.get_num_pages();

    //extract all chapters
    let pool = threadpool::Builder::new().build();

    let arc_book = Arc::new(Mutex::new(book));
    for i in 0..len {
        let this_book = arc_book.clone();
        let this_path = path_name.clone();
        pool.execute(move || {
            let mut locked_book = this_book.lock().unwrap();
            if let Err(error) = locked_book.set_current_page(i) {
                println!("ERROR: {}", error);
                return;
            }
            let chapter = locked_book.get_current_str().unwrap();
            let page_path = this_path.with_file_name(format!("page_{}.html", i));
            let mut file = File::create(page_path).unwrap();
            file.write_all(chapter.as_bytes()).unwrap();
        })
    }
    Ok(())
}

pub fn get_chapter_text(path: &str, chapter_number: usize) -> Rc<String> {
    let slice = get_chapter_text_utf8(path, chapter_number);
    let text = std::str::from_utf8(&slice).unwrap();
    text.to_string().into()
}

pub fn get_chapter_text_utf8(path: impl Into<String>, chapter_number: usize) -> Vec<u8> {
    let path = path.into();
    let folder_name = Path::new(&path).file_stem().unwrap().to_str().unwrap();

    // try to read from txt files (where edited text is saved)
    if let Ok(text) = get_chapter_bytes(folder_name, chapter_number, FileExtension::TXT) {
        println!("DEBUG: reading from txt file");
        return text;
    }
    // at this point we know that the chapter is not edited,
    // so we update the savedata in the case in which the user edited the book
    // and then try to read from html files
    else if let Ok(text) = get_chapter_bytes(folder_name, chapter_number, FileExtension::HTML) {
        remove_edited_chapter(path, chapter_number);
        println!("DEBUG: reading from html files");
        /*
        let text = Cursor::new(text);
        return from_read(text, 100).as_bytes().to_vec();
        */

        let text = std::str::from_utf8(&text).unwrap();
        return rhtml2md::parse_html(text).into_bytes();
    }
    // if it fails, read from epub and save html page
    else if let Ok(mut book) = EpubDoc::new(&path) {
        println!("DEBUG: reading from epub file");
        book.set_current_page(chapter_number).unwrap();
        let content = book.get_current().unwrap();
        
        //let cursor = Cursor::new(content);
        // new crate to parse html
        //let text = from_read(cursor, 100).as_bytes().to_vec();
        let text = rhtml2md::parse_html(std::str::from_utf8(&content).unwrap());
        // save html page
        let page_path: PathBuf = get_saved_books_dir()
        .join(folder_name)
        .join(&format!("page_{}.html", chapter_number));

        println!("DEBUG: path to save chapter: {:?}", page_path);
        let mut file = File::create(page_path).unwrap();
        /*
        file.write_all(&text).unwrap();

        return text;
        */

        file.write_all(text.as_bytes()).unwrap();
        return text.into_bytes();
    }

    [0u8].into()
}

pub fn get_metadata_of_book(path: &str) -> HashMap<String, String> {
    let metadata_path = get_metadata_path(&path.to_string());
    if let Ok(metadata_file) = File::open(metadata_path) {
        let reader = BufReader::new(metadata_file);
        if let Ok(metadata) = serde_json::from_reader(reader) {
            return metadata;
        }
    }

    // if it fails, read from epub, saves and return metadata
    let metadata = extract_metadata(path).expect("Failed to extract metadata from epub");
    metadata
}

pub fn calculate_number_of_pages(
    path: &str,
    number_of_lines: usize,
    font_size: f64,
) -> Result<(usize, Vec<(usize, usize)>), Box<dyn error::Error>> {
    let mut metadata = get_metadata_of_book(path);
    let number_of_chapters = metadata["chapters"].parse::<usize>().unwrap_or_default();

    let pool = threadpool::Builder::new().build();


    let (tx, rx) = std::sync::mpsc::channel();
    for i in 0..number_of_chapters {
        let tx = tx.clone();
        let path = path.to_string();
        pool.execute(move || {
            let pages = split_chapter_in_vec(
                path.as_str(),
                Option::None,
                i,
                number_of_lines,
                font_size,
                800.0,
                300.0,
            );
            println!("DEBUG: chapter {} has {} pages", i, pages.len());
            // send tuple with index of chapter and number of pages
            tx.send((i, pages.len())).unwrap();
        })
    }
    let mut pages_per_chapter = vec![0; number_of_chapters];
    let mut number_of_pages = 0;

    drop(tx);
    while let Ok((i, pages)) = rx.recv() {
        println!("DEBUG: receiving chapter {} has {} pages", i, pages);
        pages_per_chapter[i] = pages;
        number_of_pages += pages;
    }

    let mut pages_per_chapter_start_end = vec![(0, 0); number_of_chapters];

    // cumulative pages per chapter
    for i in 0..pages_per_chapter.len() {
        if i == 0 {
            pages_per_chapter_start_end[i] = (0, pages_per_chapter[i] - 1);
        } else {
            let start = pages_per_chapter_start_end[i - 1].1 + 1;
            let end = start + pages_per_chapter[i] - 1;
            pages_per_chapter_start_end[i] = (start, end);
        }
    }

    // save number of pages per chapter in metadata
    metadata.insert(
        format!("pages_per_chapter_{}", FontSize::from(font_size).to_string()),
        format!(
            "[{}]",
            pages_per_chapter_start_end
                .iter()
                .map(|(a, b)| format!("({}-{})", a, b))
                .collect::<Vec<String>>()
                .join(", ")
        ),
    );
    // save number of pages in metadata
    metadata.insert("total_pages".into(), number_of_pages.to_string());

    println!("DEBUG metadata: {:?}", metadata);

    let json = json!(metadata);
    let metadata_path = get_metadata_path(&path.to_string());

    let metadata_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(metadata_path)
        .unwrap();

    serde_json::to_writer_pretty(metadata_file, &json)?;

    Ok((number_of_pages, pages_per_chapter_start_end))
}

// get total number of pages in the book
pub fn get_number_of_pages(path: &str) -> usize {
    let metadata = get_metadata_of_book(path);

    let result = metadata.get("total_pages");
    if let Some(number_of_pages) = result {
        number_of_pages.parse::<usize>().unwrap_or_default()
    } else {
        calculate_number_of_pages(path, 8, MYENV.lock().unwrap().font.size).unwrap_or_default().0
    }
}

// get number of pages per chapter where the index of the vector is the chapter number
// and the tuple is the start and end indexes page of the chapter (start, end)
pub fn get_start_end_pages_per_chapter(path: &str) -> Vec<(usize, usize)> {
    let metadata = get_metadata_of_book(path);

    let result = metadata.get(format!("pages_per_chapter_{}", FontSize::from(MYENV.lock().unwrap().font.size).to_string()).as_str());
    if let Some(pages_per_chapter) = result {
        let vec_as_str = pages_per_chapter.to_string();
        vec_as_str
            .trim_matches(|c| c == '[' || c == ']')
            .split(',')
            .map(|s| {
                let start_end = s
                    .trim_matches(|c| c == '(' || c == ')' || c == ' ')
                    .split('-')
                    .map(|s| s.parse::<usize>().unwrap_or_default())
                    .collect::<Vec<usize>>();
                (start_end[0], start_end[1])
            })
            .collect()
    } else {
        calculate_number_of_pages(path, 8, MYENV.lock().unwrap().font.size).unwrap_or_default().1
    }
}

// get the number of pages with respect to the total number of pages in the book
pub fn get_cumulative_current_page_number(path: &str, chapter: usize, page: usize) -> usize {
    let pages_per_chapter = get_start_end_pages_per_chapter(path);
    if chapter > 0 {
        pages_per_chapter[chapter].0 + page
    } else {
        page
    }
}

/// Function that split the text of the chapter
/// into a vector of strings, each string is a paragraph
/// calculated by the number of lines and (not now) the font size
/// You  have to provide the path. Number of lines and font size
/// You can provide the text of the chapter as a RC String or
/// you can provide the chapter number
pub fn split_chapter_in_vec<S: Into<Option<Rc<String>>>, U: Into<Option<usize>>>(
    path: &str,
    opt_text: S,
    chapter_number: U,
    number_of_lines: usize,
    font_size: f64,
    width: f32,
    height: f32,
) -> Vec<Rc<String>> {
    // todo(): consider also the font size

    let text = match opt_text.into() {
        Some(book_chapter_text) => book_chapter_text,
        None => get_chapter_text(path, chapter_number.into().unwrap_or(0)),
    };

    //through font-size, we can calculate the number N of lines that fit in the page
    //split text in paragraphs long N lines
    let wf = (width / (font_size as f32)) as usize;
    let hf = (height / (font_size as f32)) as usize;

    let wfhf = wf * hf;

    //Split text by lines
    let chapter_lines = text.split('\n').collect::<Vec<&str>>();

    //Add as many lines as possible to each component of a pages vector, until we reach wfhf characters
    let mut pages = vec![];
    let mut page = String::new();
    let mut page_length = 0;

    //remove all empty strings leading and trailing the vector
    let chapter_lines = chapter_lines.into_iter().skip_while(|s| s.is_empty()).collect::<Vec<&str>>();

    for i in 0..chapter_lines.len() {

        let line = chapter_lines[i];

        if page_length + line.len() < wfhf && i != chapter_lines.len() - 1 {
            
            //if line is equal to \n
            if line.to_string() == "" {
                page.push_str("\n\n");
                page_length += wf;
            }
            else {
                //add blank space at the end of line
                let line = format!("{} ", line);
                page.push_str(line.as_str());
                page_length += line.len();
            }
        } else {

            if i == chapter_lines.len() - 1 {
                let line = format!("{} ", line);
                page.push_str(line.as_str());
            }

            pages.push(Rc::from(page));
            page = String::new();
            page.push_str(line);
            page_length = line.len();
        }
    }

    pages

}
