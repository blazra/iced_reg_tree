use iced::widget::{button, column, row, text, text_input, Button};
use iced::{color, Color, Element, Renderer, Theme};

use crate::reg16::EnumValue;
use crate::reg16::ValState;

use crate::combo_box::{self, ComboBox};

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub value_read: u16,
    pub value_write: u16,
    pub input_text: String,
    pub description: Option<String>,
    pub state: ValState,
    pub offset: u8,
    pub width: u8,
    pub enum_values: Vec<EnumValue>,
    pub enum_combo_state: combo_box::State<EnumValue>,
    pub selected_enum: Option<EnumValue>,
    pub input_id: text_input::Id,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(text_input::Id),
    InputChanged(String),
    WriteValueSubmit,
    ValSelected(EnumValue),
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

    pub fn set_value_read_from_reg(&mut self, reg: u16) {
        self.value_read = self.value_from_reg(reg);
    }

    pub fn set_value_write_from_reg(&mut self, reg: u16) {
        self.value_write = self.value_from_reg(reg);
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Select(_) => match self.state {
                ValState::None => self.state = ValState::Selected,
                ValState::Selected => self.state = ValState::Editing,
                ValState::Editing => (),
            },
            Message::InputChanged(text) => self.input_text = text,
            Message::ValSelected(val) => {
                self.input_text = val.value.to_string();
                self.selected_enum = Some(val);
            },
            Message::WriteValueSubmit => (),
        }
    }

    pub fn view(&self) -> Element<Message, Theme, Renderer> {
        let mut enum_value_read = None;
        for val in self.enum_values.iter() {
            if val.value == self.value_read {
                enum_value_read = Some(val.name.as_str());
            }
        }
        let mut enum_value_write = None;
        for val in self.enum_values.iter() {
            if val.value == self.value_write {
                enum_value_write = Some(val.name.as_str());
            }
        }
        let field_val_read = match self.width {
            1 => format!("{}", self.value_read),
            2..=4 => format!("0x{:01X}", self.value_read),
            5..=8 => format!("0x{:02X}", self.value_read),
            9..=12 => format!("0x{:03X}", self.value_read),
            13..=16 => format!("0x{:04X}", self.value_read),
            _ => unreachable!(),
        };
        let field_val_write = match self.width {
            1 => format!("{}", self.value_write),
            2..=4 => format!("0x{:01X}", self.value_write),
            5..=8 => format!("0x{:02X}", self.value_write),
            9..=12 => format!("0x{:03X}", self.value_write),
            13..=16 => format!("0x{:04X}", self.value_write),
            _ => unreachable!(),
        };
        let enum_combobox = ComboBox::new(&self.enum_combo_state, "placeholder", self.selected_enum.as_ref(), Message::ValSelected)
        .width(100)
        .on_input(Message::InputChanged);
        let field_val = match self.state {
            ValState::Editing => column![
                button(text(field_val_read.clone()))
                    .style(button::text)
                    .padding(0)
                    .on_press(Message::Select(self.input_id.clone())),
                text_input(field_val_read.as_str(), self.input_text.as_str())
                    .width(100)
                    .on_input(Message::InputChanged)
                    .on_submit(Message::WriteValueSubmit)
                    .id(self.input_id.clone()),
                enum_combobox,
            ],
            ValState::None => column![button(text(field_val_read.clone()))
                .style(button::text)
                .padding(0)
                .on_press(Message::Select(self.input_id.clone()))],
            ValState::Selected => column![button(text(field_val_read.clone()).color(Color::WHITE))
                .style(|theme, status| {
                    button::text(theme, status).with_background(color!(0x3399FF))
                })
                .padding(0)
                .on_press(Message::Select(self.input_id.clone()))],
        };
        let mut field_row = row![text_button(self.name.as_str()), field_val,].spacing(10);
        field_row = field_row.push_maybe(enum_value_read);
        if self.value_write != self.value_read {
            field_row = field_row.push("→");
            field_row = field_row.push(text(field_val_write));
            field_row = field_row.push_maybe(enum_value_write);
        }
        field_row.into()
    }
}

impl std::fmt::Display for EnumValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{} {}", self.value, self.name).as_str())
    }
}

fn text_button<'a>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Button<'a, Message> {
    button(content).style(button::text).padding(0)
}
