use std::num::ParseIntError;

use iced::alignment::Vertical::Top;
use iced::widget::{button, column, row, text, text_input, Button};
use iced::{color, Element, Renderer, Theme};

#[derive(Debug, Clone)]
pub enum ValState {
    None,
    Selected,
    Editing,
}

pub struct Reg16 {
    pub name: String,
    pub description: Option<String>,
    pub expanded: bool,
    pub state: ValState,
    pub value: u16,
    pub write_value: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleExpand,
    Select,
    InputChanged(String),
    WriteValueSubmit,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub state: ValState,
    pub offset: u8,
    pub width: u8,
    pub enum_values: Vec<EnumValue>,
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,
    pub value: u16,
}

impl Field {
    pub fn get_value(&self, reg: u16) -> u16 {
        let mask = !(0xffffu16 << self.width);
        (reg >> self.offset) & mask
    }
}

impl Reg16 {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::ToggleExpand => self.expanded = !self.expanded,
            Message::Select => match self.state {
                ValState::None => self.state = ValState::Selected,
                ValState::Selected => self.state = ValState::Editing,
                ValState::Editing => (),
            },
            Message::InputChanged(val) => self.write_value = val,
            Message::WriteValueSubmit => {
                if let Ok(value) = from_str_to_u16(self.write_value.as_str()) {
                    self.value = value;
                    self.state = ValState::Selected;
                }
            }
        }
    }

    pub fn view(&self) -> Element<Message, Theme, Renderer> {
        let but_text = if self.expanded { "-" } else { "+" };
        let read_value = value_button(
            format!("0x{:04X}", self.value),
            self.write_value.clone(),
            self.state.clone(),
            Some(Message::Select),
            Message::InputChanged,
            Message::WriteValueSubmit,
        );
        let mut reg = row![
            text_button(but_text).on_press(Message::ToggleExpand),
            text_button(self.name.as_str()),
            read_value,
        ]
        .align_y(Top)
        .spacing(10);
        if self.expanded {
            let mut fields_col = column![];
            for field in self.fields.iter() {
                let mut enum_value = None;
                for val in field.enum_values.iter() {
                    if val.value == field.get_value(self.value) {
                        enum_value = Some(val.name.as_str());
                    }
                }
                let field_val = match field.width {
                    1 => format!("{}", field.get_value(self.value)),
                    1..=4 => format!("0x{:01X}", field.get_value(self.value)),
                    5..=8 => format!("0x{:02X}", field.get_value(self.value)),
                    9..=12 => format!("0x{:03X}", field.get_value(self.value)),
                    13..=16 => format!("0x{:04X}", field.get_value(self.value)),
                    _ => unreachable!(),
                };
                let field_val = value_button(
                    field_val,
                    String::from(""),
                    field.state.clone(),
                    None,
                    Message::InputChanged,
                    Message::WriteValueSubmit,
                );
                let mut field_row = row![text_button(field.name.as_str()), field_val,].spacing(10);
                field_row = field_row.push_maybe(enum_value);
                fields_col = fields_col.push(field_row);
            }
            reg = reg.push(fields_col);
        }
        reg.into()
    }
}

fn text_button<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message> {
    button(content).style(button::text).padding(0)
}

fn value_button<'a>(
    value: String,
    write_value: String,
    state: ValState,
    on_press: Option<Message>,
    on_input: impl Fn(String) -> Message + Clone + 'a,
    on_submit: Message,
) -> Element<'a, Message, Theme, Renderer> {
    match state {
        ValState::Editing => {
            column![button(text(value.clone()))
            .style(button::text)
            .padding(0)
            .on_press_maybe(on_press),
            text_input(value.as_str(), write_value.as_str())
            .width(100)
            .on_input(on_input.clone())
            .on_submit(on_submit),
            ].into()
        },
        ValState::None => button(text(value.clone()))
            .style(button::text)
            .padding(0)
            .on_press_maybe(on_press)
            .into(),
        ValState::Selected => button(text(value.clone()))
            .style(|theme, status| {
                button::text(theme, status).with_background(color!(0x3399FF))
            })
            .padding(0)
            .on_press_maybe(on_press)
            .into(),
    }
}

fn from_str_to_u16(src: &str) -> Result<u16, ParseIntError> {
    let mut src = src.trim();
    src = src.trim_start_matches("+");
    if let Some(src) = src.strip_prefix("0x") {
        u16::from_str_radix(src, 16)
    } else if let Some(src) = src.strip_prefix("0b") {
        u16::from_str_radix(src, 2)
    } else {
        src.parse::<u16>()
    }
}