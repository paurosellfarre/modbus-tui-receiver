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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    //TUI Configuration
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let addr = "127.0.0.1:5502".parse()?;
    let mut modbus_client = tcp::connect(addr).await?;

    // Registers we will read
    let registers_ai: HashMap<&str, u16> = [
        ("AI10", 10), ("AI11", 11),
        ("AI12", 12), ("AI13", 13), // 32 bits registers = 2 16 bits registers
        ("AI17", 17), ("AI18", 18), ("AI19", 19), ("AI20", 20),
        ("AI30", 30), ("AI50", 50), ("AI231", 231), ("AI232", 232), ("AI233", 233),
    ].iter().cloned().collect();

    let registers_di: HashMap<&str, u16> = [
        ("DI0", 00), ("DI1", 01), ("DI8", 08), ("DI80", 080),
    ].iter().cloned().collect();

    loop {
        let mut rows = vec![];
        
        // Read AI registers
        for (name, address) in &registers_ai {
            let cnt;

            match *address {
                12 | 13 => cnt = 2,
                11 | 18 => cnt = 3,
                _ => cnt = 1,
            }

            match modbus_client.read_input_registers(*address, cnt).await {
                Ok(result) => {
                    let value = match result {
                        Ok(data) => {
                            match *address {

                                12 | 13 => {
                                    let high = data.get(0).copied().unwrap_or(0) as u16;
                                    let low = data.get(1).copied().unwrap_or(0) as u16;
                                    
                                    // Combina los dos valores de 16 bits en un valor de 32 bits
                                    ((high as i32) << 16) | (low as i32)
                                },
                                11 | 18 => {
                                    let negative_indicator = data.get(0).copied().unwrap_or(0) as u16;
                                    let value = data.get(1).copied().unwrap_or(0) as u16;

                                    // If the negative indicator is set, the value is negative
                                    if negative_indicator == 1 {
                                        //print!("Negative value: {}", value);
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

        // Read DI registers
        for (name, address) in &registers_di {
            match modbus_client.read_input_registers(*address, 1).await {
                Ok(result) => {
                    let value = match result {
                        Ok(data) => data.get(0).copied().unwrap_or(0),
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
