use iced::widget::{button, column, row, text, text_input, Button};
use iced::{color, Element, Renderer, Theme};

use crate::reg16::EnumValue;
use crate::reg16::ValState;

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub value: u16,
    pub write_value: String,
    pub description: Option<String>,
    pub state: ValState,
    pub offset: u8,
    pub width: u8,
    pub enum_values: Vec<EnumValue>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select,
    InputChanged(String),
    WriteValueSubmit,
}

impl Field {
    pub fn value_from_reg(&self, reg: u16) -> u16 {
        let mask = !(0xffffu16 << self.width);
        (reg >> self.offset) & mask
    }
    
    pub fn value_reg_from_field(&self, reg: u16, field: u16) -> u16 {
        let mask = (!(0xffffu16 << self.width)) << self.offset;
        ((field << self.offset) & mask) | (reg & !mask)
    }
    
    pub fn set_value_from_reg(&mut self, reg: u16) {
        self.value = self.value_from_reg(reg);
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Select => {
                match self.state {
                    ValState::None => self.state = ValState::Selected,
                    ValState::Selected => self.state = ValState::Editing,
                    ValState::Editing => (),
                }
            }
            Message::InputChanged(write_val) => self.write_value = write_val,
            Message::WriteValueSubmit => (),
        }
    }

    pub fn view(&self) -> Element<Message, Theme, Renderer> {
        let mut enum_value = None;
        for val in self.enum_values.iter() {
            if val.value == self.value {
                enum_value = Some(val.name.as_str());
            }
        }
        let field_val = match self.width {
            1 => format!("{}", self.value),
            2..=4 => format!("0x{:01X}", self.value),
            5..=8 => format!("0x{:02X}", self.value),
            9..=12 => format!("0x{:03X}", self.value),
            13..=16 => format!("0x{:04X}", self.value),
            _ => unreachable!(),
        };
        let field_val = value_button(
            field_val,
            self.write_value.clone(),
            self.state.clone(),
            Some(Message::Select),
            Message::InputChanged,
            Message::WriteValueSubmit,
        );
        let field_row = row![text_button(self.name.as_str()), field_val,].spacing(10);
        field_row.push_maybe(enum_value).into()
    }
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
        ValState::Editing => column![
            button(text(value.clone()))
                .style(button::text)
                .padding(0)
                .on_press_maybe(on_press),
            text_input(value.as_str(), write_value.as_str())
                .width(100)
                .on_input(on_input.clone())
                .on_submit(on_submit),
        ]
        .into(),
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

fn text_button<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message> {
    button(content).style(button::text).padding(0)
}