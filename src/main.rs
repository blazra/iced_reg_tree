use svd_parser as svd;

use std::fs::File;
use std::io::Read;

use iced::widget::{column, Column};
use iced::{Font, Task};

pub mod field;
pub mod reg16;
use field::Field;
use reg16::{EnumValue, Reg16, ValState};

pub fn main() -> iced::Result {
    iced::application("Iced Reg Tree", App::update, App::view)
        .default_font(Font::MONOSPACE)
        .run_with(App::init)
}

struct App {
    regs: Vec<Reg16>,
}

#[derive(Debug, Clone)]
enum Message {
    Reg(usize, reg16::Message),
}

impl App {
    fn init() -> (App, Task<Message>) {
        let xml = &mut String::new();
        let _ = File::open("example.svd").unwrap().read_to_string(xml);
        let device = svd::parse(xml).unwrap();
        let mut regs = vec![];

        for periph in device.peripherals {
            for reg in periph.all_registers() {
                let mut fields = vec![];
                for field in reg.fields() {
                    let mut enum_values = vec![];
                    for svd_enum_values in field.enumerated_values.iter() {
                        for svd_enum_value in svd_enum_values.values.iter() {
                            if let Some(val) = svd_enum_value.value {
                                enum_values.push(EnumValue {
                                    name: svd_enum_value.name.clone(),
                                    description: svd_enum_value.description.clone(),
                                    value: val as u16,
                                })
                            }
                        }
                    }
                    fields.push(Field {
                        name: field.name.clone(),
                        description: field.description.clone(),
                        value: 0,
                        write_value: String::from("0x0000"),
                        state: ValState::None,
                        offset: field.bit_range.offset as u8,
                        width: field.bit_range.width as u8,
                        enum_values,
                    });
                }
                regs.push(Reg16 {
                    name: reg.name.clone(),
                    description: reg.description.clone(),
                    expanded: false,
                    state: ValState::None,
                    value: 0,
                    write_value: String::from("0x0000"),
                    fields,
                })
            }
        }

        (App { regs }, Task::none())
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Reg(idx, msg) => {
                if matches!(msg, reg16::Message::Select) {
                    for (j, reg) in self.regs.iter_mut().enumerate() {
                        if idx != j {
                            reg.state = ValState::None;
                        }
                        for field in reg.fields.iter_mut() {
                            field.state = ValState::None;
                        }
                    }
                }
                if let reg16::Message::FieldChanged(field_idx, field::Message::Select) = msg {
                    for (j, reg) in self.regs.iter_mut().enumerate() {
                        reg.state = ValState::None;
                        for (k, field) in reg.fields.iter_mut().enumerate() {
                            if !(idx == j && field_idx == k) {
                                field.state = ValState::None;
                            }
                        }
                    }
                }
                self.regs[idx].update(msg)
            }
        }
    }

    fn view(&self) -> Column<Message> {
        column(
            self.regs
                .iter()
                .map(Reg16::view)
                .enumerate()
                .map(|(index, reg)| reg.map(move |message| Message::Reg(index, message))),
        )
        .padding(20)
        .spacing(20)
    }
}
