use druid::widget::{Button, Flex, Label, LineBreaking};
use druid::{
    BoxConstraints, Color, Command, Env, Event, EventCtx, FontDescriptor, FontFamily, FontWeight,
    LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size, Target, UpdateCtx, Widget, WidgetExt,
    WidgetPod,
};

use crate::ENTERING_READING_MODE;

use super::{
    book::{Book, GUIBook},
    library::GUILibrary,
    mockup::MockupLibrary,
};

type Library = MockupLibrary<Book>;

pub struct BookDetails {
    inner: WidgetPod<Library, Box<dyn Widget<Library>>>,
}

impl BookDetails {
    pub fn new() -> Self {
        let header_font = FontDescriptor::new(FontFamily::new_unchecked("Roboto"))
            .with_weight(FontWeight::BOLD)
            .with_size(28.0);
        let info_font = FontDescriptor::new(FontFamily::new_unchecked("Roboto"))
            .with_weight(FontWeight::NORMAL)
            .with_size(14.0);

        let mut header_label = Label::new("Dettagli del libro")
            .with_text_color(Color::rgb8(0, 0, 0))
            .with_font(header_font)
            .with_text_alignment(druid::TextAlignment::Start);
        header_label.set_line_break_mode(LineBreaking::WordWrap);

        let title_label = Label::dynamic(|data: &Library, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |book| {
                    format!("Titolo: {}", book.get_title().to_string())
                })
        })
        .with_text_color(Color::BLACK)
        .with_font(info_font.clone())
        .align_left()
        .padding(5.0);

        let author_label = Label::dynamic(|data: &Library, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |book| {
                    format!("Autore: {}", book.get_author().to_string())
                })
        })
        .with_text_color(Color::BLACK)
        .with_font(info_font.clone())
        .align_left()
        .padding(5.0);

        let lang_label = Label::dynamic(|data: &Library, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |book: &Book| {
                    format!("Lingua: {}", lang_parser(&book.get_lang()))
                })
        })
        .with_font(info_font.clone())
        .with_text_color(Color::BLACK)
        .align_left()
        .padding(5.0);

        let completion_label = Label::dynamic(|data: &Library, _| {
            data.get_selected_book()
                .map_or("Nessun libro selezionato".into(), |_: &Book| {
                    format!("Letto al {}%", String::from("//TODO: Add get_compl_perc()"))
                })
        })
        .with_font(info_font.clone())
        .with_text_color(Color::BLACK)
        .align_left()
        .padding(5.0);

        let keep_reading_btn =
            Button::new("Continua a Leggere").on_click(|ctx, _: &mut Library, _: &Env| {
                let cmd: Command = Command::new(ENTERING_READING_MODE, (), Target::Auto);
                ctx.submit_command(cmd.clone());
                println!("Notification submitted");
            });

        let mut btn_ctls = Flex::row()
            .with_flex_child(keep_reading_btn, 1.0)
            .with_flex_child(Button::new("Aggiungi ai Preferiti"), 1.0);

        btn_ctls.set_main_axis_alignment(druid::widget::MainAxisAlignment::SpaceAround);

        let btn_ctls = btn_ctls.expand_width().padding(5.0);

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
            .expand()
            .boxed();
        let inner = WidgetPod::new(widget);

        Self { inner }
    }
}

impl Widget<Library> for BookDetails {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Library, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Library, env: &Env) {
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &Library, data: &Library, env: &Env) {
        self.inner.update(ctx, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Library,
        env: &Env,
    ) -> Size {
        if let Some(_) = data.get_selected_book() {
            self.inner.layout(ctx, bc, data, env)
        } else {
            Size::ZERO
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Library, env: &Env) {
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
