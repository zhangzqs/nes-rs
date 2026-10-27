use nes_board::Console;
use std::{fs, path::Path};

#[derive(Default)]
pub struct Replay {
    events: Vec<(u64, [u8; 2])>,
    next: usize,
}
impl Replay {
    pub fn load(path: Option<&Path>) -> Result<Self, String> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
        let mut replay = Self::default();
        for (line_number, line) in text.lines().enumerate() {
            let columns: Vec<_> = line.split('#').next().unwrap().split_whitespace().collect();
            if columns.is_empty() {
                continue;
            }
            let invalid = || {
                format!(
                    "{}:{}: expected increasing FRAME P1_HEX P2_HEX",
                    path.display(),
                    line_number + 1
                )
            };
            if columns.len() != 3 {
                return Err(invalid());
            }
            let frame = columns[0].parse::<u64>().map_err(|_| invalid())?;
            let mut buttons = [0; 2];
            for (index, mask) in columns[1..].iter().enumerate() {
                buttons[index] =
                    u8::from_str_radix(mask.trim_start_matches("0x"), 16).map_err(|_| invalid())?;
            }
            if replay.events.last().is_some_and(|(last, _)| *last >= frame) {
                return Err(invalid());
            }
            replay.events.push((frame, buttons));
        }
        Ok(replay)
    }
    pub fn apply(&mut self, frame: u64, console: &mut Console) {
        if let Some(&(at, buttons)) = self.events.get(self.next)
            && at == frame
        {
            for (player, mask) in buttons.into_iter().enumerate() {
                console.set_buttons(player, mask);
            }
            self.next += 1;
        }
    }
}
