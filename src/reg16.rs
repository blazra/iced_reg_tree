use std::num::ParseIntError;

use iced::alignment::Vertical::Top;
use iced::widget::{button, column, row, text, text_input, Button};
use iced::{color, Element, Renderer, Task, Theme};

use crate::field;
use crate::field::Field;

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
    pub input_id: text_input::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleExpand,
    Select(text_input::Id),
    InputChanged(String),
    WriteValueSubmit,
    FieldChanged(usize, field::Message),
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,
    pub value: u16,
}

impl Reg16 {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleExpand => self.expanded = !self.expanded,
            Message::Select(id) => match self.state {
                ValState::None => {
                    self.state = ValState::Selected;
                    return text_input::focus(id);
                }
                ValState::Selected => self.state = ValState::Editing,
                ValState::Editing => (),
            },
            Message::InputChanged(val) => self.write_value = val,
            Message::WriteValueSubmit => {
                if let Ok(value) = from_str_to_u16(self.write_value.as_str()) {
                    self.value = value;
                    self.state = ValState::Selected;
                    for field in self.fields.iter_mut() {
                        field.set_value_from_reg(value);
                    }
                }
            }
            Message::FieldChanged(index, message) => match message {
                field::Message::Select(id) => {
                    for (j, field) in self.fields.iter_mut().enumerate() {
                        if j == index {
                            field.update(field::Message::Select(id.clone()));
                        } else {
                            field.state = ValState::None;
                        }
                    }
                }
                field::Message::InputChanged(_) => self.fields[index].update(message),
                field::Message::WriteValueSubmit => {
                    if let Ok(value) = from_str_to_u16(self.fields[index].write_value.as_str()) {
                        self.value = self.fields[index].value_reg_from_field(self.value, value);
                        self.fields[index].state = ValState::Selected;
                        for field in self.fields.iter_mut() {
                            field.set_value_from_reg(self.value)
                        }
                    }
                },
                _ => self.fields[index].update(message)
            },
        }
        Task::none()
    }

    pub fn view(&self) -> Element<Message, Theme, Renderer> {
        let but_text = if self.expanded { "-" } else { "+" };
        let read_value = self.value_button();
        let mut reg = row![
            text_button(but_text).on_press(Message::ToggleExpand),
            text_button(self.name.as_str()),
            read_value,
        ]
        .align_y(Top)
        .spacing(10);
        if self.expanded {
            let fields_col = column(self.fields.iter().map(Field::view).enumerate().map(
                |(index, field)| field.map(move |message| Message::FieldChanged(index, message)),
            ));
            reg = reg.push(fields_col);
        }
        reg.into()
    }

    fn value_button<'a>(&self) -> Element<'a, Message, Theme, Renderer> {
        let value = format!("0x{:04X}", self.value);
        let val_but = button(text(value.clone()))
            .style(button::text)
            .padding(0)
            .on_press(Message::Select(self.input_id.clone()));
        match self.state {
            ValState::Editing => {
                let val_input = text_input(value.as_str(), self.write_value.as_str())
                    .id(self.input_id.clone())
                    .width(100)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::WriteValueSubmit);
                column![val_but, val_input,].into()
            }
            ValState::None => val_but.into(),
            ValState::Selected => val_but
                .style(|theme, status| {
                    button::text(theme, status).with_background(color!(0x3399FF))
                })
                .into(),
        }
    }
}

fn text_button<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message> {
    button(content).style(button::text).padding(0)
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
