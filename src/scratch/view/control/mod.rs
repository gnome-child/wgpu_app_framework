mod button;
mod checkbox;
mod radio;
mod slider;
mod text_area;
mod text_box;
mod wrap;

pub use button::Button;
pub use checkbox::Checkbox;
pub use radio::Radio;
pub use slider::Slider;
pub use text_area::TextArea;
pub use text_box::TextBox;
pub use wrap::Wrap;

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Button(Button),
    Checkbox(Checkbox),
    Radio(Radio),
    Slider(Slider),
    TextArea(TextArea),
    TextBox(TextBox),
}
