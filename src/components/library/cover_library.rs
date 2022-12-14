use druid::{
    BoxConstraints, Env, Event, EventCtx, Key, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
    Size, UpdateCtx, Widget, WidgetPod,
};

use crate::{
    components::book::book_cover::{BookCover, BOOK_WIDGET_SIZE},
    models::book::Book,
    models::library::SELECTED_BOOK_SELECTOR,
    traits::gui::{GUIBook, GUILibrary},
    Library,
};

pub struct CoverLibrary {
    children: Vec<WidgetPod<Book, BookCover<Book>>>,
}

pub const DO_PAINT_SHADOWS: Key<bool> = Key::new("crabreader.do_paint_shadows");

impl CoverLibrary {
    pub fn new() -> Self {
        Self { children: vec![] }
    }

    pub fn add_child(&mut self) {
        let widget = BookCover::new(); // .with_cover_image(book.get_path());
        let pod = WidgetPod::new(widget);
        self.children.push(pod);
    }

    fn update_child_count(&mut self, ctx: &mut LifeCycleCtx, data: &Library<Book>) -> bool {
        let target = data.number_of_books();
        let current = self.children.len();
        if target > current {
            for _ in current..target {
                self.add_child();
            }
            ctx.children_changed();
            true
        } else if target < current {
            self.children.truncate(target);
            ctx.children_changed();
            true
        } else {
            false
        }
    }
}

impl Widget<Library<Book>> for CoverLibrary {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Library<Book>, env: &Env) {
        for (idx, inner) in self.children.iter_mut().enumerate() {
            if let Some(book) = data.get_book_mut(idx) {
                if !event.should_propagate_to_hidden() && !inner.is_initialized() {
                    continue;
                }
                inner.event(ctx, event, book, env);
            }
        }

        if data.check_books_loaded() {
            ctx.request_layout();
        }

        if data.check_covers_loaded() {
            ctx.request_update();
        }

        match event {
            Event::MouseDown(_) => {
                if !ctx.is_handled() {
                    data.unselect_current_book();
                    ctx.request_paint();
                }
            }
            Event::Notification(cmd) => {
                if let Some(idx) = cmd.get(SELECTED_BOOK_SELECTOR) {
                    if let Some(idx) = idx {
                        data.set_selected_book_idx(*idx);
                    } else {
                        data.unselect_current_book();
                    }
                    ctx.request_paint();
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Library<Book>,
        env: &Env,
    ) {
        if self.update_child_count(ctx, data) {
            ctx.request_layout();
        }

        for (idx, inner) in self.children.iter_mut().enumerate() {
            if let Some(book) = data.get_book(idx) {
                if !event.should_propagate_to_hidden() && !inner.is_initialized() {
                    continue;
                }
                inner.lifecycle(ctx, event, book, env);
            }
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &Library<Book>,
        data: &Library<Book>,
        env: &Env,
    ) {
        if data.only_fav() != old_data.only_fav() {
            ctx.request_layout();
        }

        for (idx, inner) in self.children.iter_mut().enumerate() {
            if let Some(book) = data.get_book(idx) {
                inner.update(ctx, book, env);
            }
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Library<Book>,
        env: &Env,
    ) -> Size {
        let book_w = BOOK_WIDGET_SIZE.width;
        let book_h = BOOK_WIDGET_SIZE.height;
        let width = bc.max().width;
        let min_spacing = 30.0;
        let mut cnt = 0;

        let books_per_row =
            (((width - min_spacing) / (book_w + min_spacing)).floor() as usize).max(1);
        let rows = ((data.get_number_of_visible_books() as f64 / books_per_row as f64).ceil()
            as usize)
            .max(1);
        let spacing = (width - (books_per_row as f64 * book_w)) / (books_per_row as f64 + 1.0);
        let xspacing = ((width - (data.get_number_of_visible_books() as f64 * book_w))
            / (data.get_number_of_visible_books() as f64 + 1.0))
            .max(spacing);

        for (idx, inner) in self.children.iter_mut().enumerate() {
            if let Some(book) = data.get_book(idx) {
                if !inner.is_initialized() {
                    continue;
                }

                if book.is_filtered_out() {
                    inner.layout(ctx, bc, book, env);
                    continue;
                }

                let row = cnt / books_per_row;
                let col = cnt % books_per_row;

                let x = xspacing + (col as f64 * (book_w + xspacing));
                let y = spacing + (row as f64 * (book_h + spacing));

                let origin = Point::new(x, y);
                inner.layout(ctx, bc, book, env);
                inner.set_origin(ctx, book, env, origin); // TLDR: must be a WidgetPod...
                cnt += 1;
            }
        }

        let w = books_per_row as f64 * book_w + (books_per_row as f64 + 1.0) * spacing;
        let h = rows as f64 * (book_h + spacing) + spacing;
        (w, h).into()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Library<Book>, env: &Env) {
        for (idx, inner) in self.children.iter_mut().enumerate() {
            if let Some(book) = data.get_book(idx) {
                if inner.is_initialized() {
                    inner.paint(ctx, book, env);
                }
            }
        }
    }
}
