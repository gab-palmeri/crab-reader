use druid::{
    widget::{Flex, Label, LineBreaking},
    BoxConstraints, Command, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Size, Target, UpdateCtx, Widget, WidgetExt, WidgetPod,
};

use crate::{
    components::buttons::rbtn::RoundedButton,
    models::book::Book,
    traits::{
        gui::{GUIBook, GUILibrary},
        reader::BookManagement,
    },
    utils::{colors, fonts, saveload::delete_book},
    Library, ENTERING_READING_MODE,
};

pub struct BookDetails {
    inner: WidgetPod<Library<Book>, Box<dyn Widget<Library<Book>>>>,
}

impl BookDetails {
    pub fn new() -> Self {
        let header_label = Label::new("Dettagli del libro")
            .with_text_alignment(druid::TextAlignment::Start)
            .with_line_break_mode(LineBreaking::WordWrap)
            .with_text_color(colors::ON_BACKGROUND)
            .with_font(fonts::bold::xlarge);

        let title_label = Label::dynamic(|data: &Library<Book>, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |book| {
                    format!("Titolo: {}", book.get_title().to_string())
                })
        })
        .with_text_color(colors::ON_BACKGROUND)
        .with_line_break_mode(LineBreaking::WordWrap)
        .with_font(fonts::medium)
        .align_left()
        .padding(5.0);

        let author_label = Label::dynamic(|data: &Library<Book>, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |book| {
                    format!("Autore: {}", book.get_author().to_string())
                })
        })
        .with_font(fonts::medium)
        .with_text_color(colors::ON_BACKGROUND)
        .align_left()
        .padding(5.0);

        let lang_label = Label::dynamic(|data: &Library<Book>, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |book: &Book| {
                    format!("Lingua: {}", lang_parser(&book.get_lang()))
                })
        })
        .with_font(fonts::medium)
        .with_text_color(colors::ON_BACKGROUND)
        .align_left()
        .padding(5.0);

        let completion_label = Label::dynamic(|data: &Library<Book>, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |book: &Book| {
                    let perc = book.get_perc_read();
                    format!("Percentuale avanzamento: {:.0}%", perc)
                })
        })
        .with_font(fonts::small)
        .with_text_color(colors::ON_BACKGROUND)
        .align_left()
        .padding(5.0);

        let keep_reading_btn = RoundedButton::from_text("Continua a Leggere")
            .with_on_click(|ctx, library: &mut Library<Book>, _: &Env| {
                let current_book = library.get_selected_book_mut().unwrap();

                // @cocco: thread?
                current_book.load_chapter();
                // @cocco: thread?
                current_book.load_notes();
                let cmd: Command = Command::new(ENTERING_READING_MODE, (), Target::Auto);
                ctx.submit_command(cmd.clone());
            })
            .with_font(fonts::medium);

        let add_fav_btn = RoundedButton::dynamic(|data: &Library<Book>, _| {
            if let Some(book) = data.get_selected_book() {
                if book.is_favorite() {
                    "Rimuovi dai preferiti".into()
                } else {
                    "Aggiungi ai preferiti".into()
                }
            } else {
                "Aggiungi ai preferiti".into()
            }
        })
        .with_on_click(|_: &mut EventCtx, library: &mut Library<Book>, _: &Env| {
            if let Some(book) = library.get_selected_book_mut() {
                let fav = book.is_favorite();
                book.set_favorite(!fav);
            }
        })
        .with_font(fonts::medium);

        let mut btn_ctls = Flex::row()
            .with_flex_child(keep_reading_btn, 1.0)
            .with_spacer(5.0)
            .with_flex_child(add_fav_btn, 1.0);

        btn_ctls.set_main_axis_alignment(druid::widget::MainAxisAlignment::SpaceAround);

        let btn_ctls = btn_ctls.expand_width().padding(5.0);

        let del_btn = RoundedButton::from_text("Elimina")
            .with_on_click(|ctx, library: &mut Library<Book>, _: &Env| {
                if let Some(book) = library.get_selected_book() {
                    // remove book from epubs and saved_books
                    if let Ok(_) = delete_book(&book.get_path()) {
                        // remove book from library
                        library.remove_book(book.get_index());
                        println!("Eliminato libro");
                        ctx.children_changed();
                        ctx.request_layout();
                    } else {
                        println!("Errore eliminazione libro");
                    }
                }
            })
            .secondary()
            .with_font(fonts::medium)
            .padding(5.0);

        // inside the function to open the book there should be
        // the book's functions lo load chapters and page
        // Book::load_chapter(), Book::load_page()

        let widget = Flex::column()
            .with_child(header_label)
            .with_child(title_label)
            .with_child(author_label)
            .with_child(lang_label)
            .with_child(completion_label)
            .with_child(btn_ctls)
            .with_child(del_btn)
            .padding(10.0)
            .expand()
            .boxed();
        let inner = WidgetPod::new(widget);

        Self { inner }
    }
}

impl Widget<Library<Book>> for BookDetails {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Library<Book>, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Library<Book>,
        env: &Env,
    ) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &Library<Book>,
        data: &Library<Book>,
        env: &Env,
    ) {
        if !old_data.same(data) {
            self.inner.update(ctx, data, env);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Library<Book>,
        env: &Env,
    ) -> Size {
        if let Some(_) = data.get_selected_book() {
            self.inner.layout(ctx, bc, data, env)
        } else {
            Size::ZERO
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Library<Book>, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}

fn lang_parser(lang: &str) -> String {
    match lang {
        "it" => "Italiano".into(),
        "es" => "Spagnolo".into(),
        "en" => "Inglese".into(),
        "fr" => "Francese".into(),
        "de" => "Tedesco".into(),
        "ru" => "Russo".into(),
        "zh" => "Cinese".into(),
        "ja" => "Giapponese".into(),
        "ar" => "Arabo".into(),
        "pt" => "Portoghese".into(),
        "ko" => "Coreano".into(),
        "hi" => "Hindi".into(),
        "tr" => "Turco".into(),
        "ur" => "Urdu".into(),
        "fa" => "Persiano".into(),
        "nl" => "Olandese".into(),
        "pl" => "Polacco".into(),
        "sv" => "Svedese".into(),
        "da" => "Danese".into(),
        "fi" => "Finlandese".into(),
        "no" => "Norvegese".into(),
        "cs" => "Ceco".into(),
        "el" => "Greco".into(),
        "he" => "Ebraico".into(),
        "ro" => "Rumeno".into(),
        "sk" => "Slovacco".into(),
        "sl" => "Sloveno".into(),
        "hu" => "Ungherese".into(),
        "vi" => "Vietnamita".into(),
        "th" => "Tailandese".into(),
        "bg" => "Bulgaro".into(),
        "uk" => "Ucraino".into(),
        "be" => "Bielorusso".into(),
        "ka" => "Georgiano".into(),
        "af" => "Afrikaans".into(),
        "sq" => "Albanese".into(),
        "am" => "Amharico".into(),
        "hy" => "Armeno".into(),
        "az" => "Azero".into(),
        "eu" => "Basco".into(),
        "bn" => "Bengalese".into(),
        "my" => "Birmano".into(),
        "km" => "Cambogiano".into(),
        "hr" => "Croato".into(),
        "eo" => "Esperanto".into(),
        "et" => "Estone".into(),
        "fo" => "Faroese".into(),
        "gl" => "Galiziano".into(),
        "gu" => "Gujarati".into(),
        "iw" => "Hebreo".into(),
        "is" => "Islandese".into(),
        _ => "Lingua non riconosciuta".into(),
        // Grazie Copilot <3
    }
}
