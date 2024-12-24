/// Text
pub trait Text {
    fn text(&self) -> &'static str;

    fn hover_text(&self) -> &'static str;
}
