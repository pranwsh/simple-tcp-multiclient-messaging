use ratatui::{
    layout::{Rect, Alignment},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, BorderType},
};
use common::ClientMessage;

pub struct MessageRenderer;

impl MessageRenderer {
    pub fn render(f: &mut ratatui::Frame, area: Rect, messages: &[ClientMessage], from_id: u64) {
        if messages.is_empty() {
            return;
        }

        let mut current_y = area.y + area.height;
        
        for msg in messages.iter().rev() {
            let is_outgoing = msg.get_sender_id() == from_id;
            let content = msg.get_content();
            
            let max_text_width = (area.width / 2 + 10).max(15) - 2;
            let width = (content.len() as u16 + 2).min(max_text_width).max(10) + 2;
            let wrapped_lines = (content.len() as u16 + (width - 3)) / (width - 2);
            let height = wrapped_lines + 2;
            
            if current_y < area.y + height {
                break;
            }
            
            current_y -= height;
            
            let x = if is_outgoing {
                area.x + area.width - width
            } else {
                area.x
            };
            
            let msg_area = Rect::new(x, current_y, width, height);
            
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(if is_outgoing { Color::Cyan } else { Color::Green }))
                .title(if is_outgoing { " Me " } else { " Them " });
            
            let p = Paragraph::new(content)
                .block(block)
                .alignment(if is_outgoing { Alignment::Right } else { Alignment::Left })
                .wrap(ratatui::widgets::Wrap { trim: true });
            
            f.render_widget(p, msg_area);
            
            if current_y > area.y + 1 {
                current_y -= 1; 
            }
        }
    }
}
