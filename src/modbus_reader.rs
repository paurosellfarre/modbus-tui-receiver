use std::{collections::HashMap, io};
use tokio_modbus::prelude::*;
use tokio_modbus::client::tcp;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table},
    Terminal,
};
use tokio::time::Duration;

#[derive(Default)]
struct Register {
    name: &'static str,
    address: u16,
    bigger_than_16_bits: bool,
    can_be_negative: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    //TUI Configuration
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let addr = "127.0.0.1:5502".parse()?;
    let mut modbus_client = tcp::connect(addr).await?;

    // Registers we will read
    let registers = vec![
        Register { name: "AI10", address: 10, ..Default::default() },
        Register { name: "AI11", address: 11, bigger_than_16_bits: false, can_be_negative: true },
        Register { name: "AI12", address: 12, bigger_than_16_bits: true, can_be_negative: false },
        Register { name: "AI13", address: 13, bigger_than_16_bits: true, can_be_negative: false },
        Register { name: "AI17", address: 17, ..Default::default() },
        Register { name: "AI18", address: 18, bigger_than_16_bits: false, can_be_negative: true },
        Register { name: "AI19", address: 19, ..Default::default() },
        Register { name: "AI20", address: 20, ..Default::default() },
        Register { name: "AI30", address: 30, ..Default::default() },
        Register { name: "AI50", address: 50, ..Default::default() },
        Register { name: "AI231", address: 231, ..Default::default() },
        Register { name: "AI232", address: 232, ..Default::default() },
        Register { name: "AI233", address: 233, ..Default::default() },
        Register { name: "DI0", address: 00, ..Default::default() },
        Register { name: "DI1", address: 01, ..Default::default() },
        Register { name: "DI8", address: 08, ..Default::default() },
        Register { name: "DI80", address: 080, ..Default::default() },
    ];

    loop {
        let mut rows = vec![];
        
        // Read AI registers
        for reg in &registers {
            let name = reg.name;
            let address = reg.address;
            let bigger_than_16_bits = reg.bigger_than_16_bits;
            let can_be_negative = reg.can_be_negative;

            // If the register is bigger than 16 bits, we need to read 2 registers
            // If the register can be negative, we need to read 3 registers
            let cnt = match (bigger_than_16_bits, can_be_negative) {
                (true, false) => 2,
                (false, true) => 3,
                _ => 1,   
            };

            match modbus_client.read_input_registers(address, cnt).await {
                Ok(result) => {
                    let value = match result {
                        Ok(data) => {
                            match address {

                                12 | 13 => {
                                    let high = data.get(0).copied().unwrap_or(0) as u16;
                                    let low = data.get(1).copied().unwrap_or(0) as u16;
                                    
                                    // Combine the two 16 bits registers into a single 32 bits register
                                    ((high as i32) << 16) | (low as i32)
                                },
                                11 | 18 => {
                                    let negative_indicator = data.get(0).copied().unwrap_or(0) as u16;
                                    let value = data.get(1).copied().unwrap_or(0) as u16;

                                    // If the negative indicator is set, the value is negative
                                    if negative_indicator == 1 {
                                        //Need to subtract 65536 to transform from u16 to i32
                                        (value as i32) - 65536
                                    } else {
                                        value as i32
                                    }
                                }
                                _ => {
                                    data.get(0).copied().unwrap_or(0) as i32
                                }
                            }
                        },
                        Err(_) => 0,
                    };
                    rows.push(Row::new(vec![name.to_string(), value.to_string()]));
                }
                Err(_) => {
                    
                    rows.push(Row::new(vec![name.to_string(), "Error".to_string()]));
                }
            }
        }

        // Draw the TUI
        terminal.draw(|rect| {
            let size = rect.size();
            let block = Block::default()
                .title("Modbus Registers")
                .borders(Borders::ALL);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let table = Table::new(rows)
                .block(block)
                .header(
                    Row::new(vec!["Name", "Value"])
                        .style(Style::default().fg(Color::Yellow))
                )
                .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

            rect.render_widget(table, layout[0]);
        })?;

        // Update every 1Hz
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
