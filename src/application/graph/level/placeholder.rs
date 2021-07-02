use crate::color::Colorable;
use gtk::prelude::WidgetExt;

#[derive(relm_derive::Msg, Clone)]
pub enum Msg {
    Draw,
}

#[relm_derive::widget(Clone)]
impl relm::Widget for Widget {
    fn model(_: ()) {}

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Draw => {
                let context = crate::create_context(&self.widgets.drawing_area).unwrap();
                context.set_color(crate::color::BACKGROUND);
                context.rectangle(0.0, 0.0, 20.0, 20.0);
                context.fill().unwrap();
            }
        }
    }

    view! {
        #[name="drawing_area"]
        gtk::DrawingArea {
            draw(_, context) => (Msg::Draw, gtk::Inhibit(false)),
        },
    }

    fn init_view(&mut self) {
        self.widgets.drawing_area.set_size_request(20, 20);
    }
}
