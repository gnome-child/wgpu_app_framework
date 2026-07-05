#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Root,
    Stack,
    MenuBar,
    Menu,
    Binding,
    Separator,
    TextArea,
    Button,
    Checkbox,
    Radio,
    Slider,
    TextBox,
    Panel,
    Popup,
    Label,
}
