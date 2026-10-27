use crate::cli::Options;
use nes_board::Console;
use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub struct Output {
    wav: Option<BufWriter<File>>,
    sample_count: u32,
}
impl Output {
    pub fn new(options: &Options) -> Result<Self, String> {
        let wav = options
            .wav
            .as_ref()
            .map(|path| {
                let mut writer = BufWriter::new(File::create(path)?);
                writer.write_all(&[0; 44])?;
                Ok::<_, std::io::Error>(writer)
            })
            .transpose()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            wav,
            sample_count: 0,
        })
    }
    pub fn audio(&mut self, samples: &[f32]) -> Result<(), String> {
        if let Some(wav) = &mut self.wav {
            for &sample in samples {
                wav.write_all(&((sample.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes())
                    .map_err(|e| e.to_string())?;
            }
            self.sample_count = self
                .sample_count
                .checked_add(samples.len() as u32)
                .filter(|count| *count <= (u32::MAX - 36) / 2)
                .ok_or("WAV exceeds 4 GiB")?;
        }
        Ok(())
    }
    pub fn finish(mut self, options: &Options, console: &Console) -> Result<(), String> {
        if let Some(path) = &options.screenshot {
            image::save_buffer_with_format(
                path,
                console.ppu.borrow().framebuffer().rgb(),
                256,
                240,
                image::ColorType::Rgb8,
                image::ImageFormat::Png,
            )
            .map_err(|e| e.to_string())?;
        }
        if let Some(path) = &options.ram_dump {
            let ram: Vec<_> = (0..0x800)
                .map(|address| console.board.ram.borrow().read(address))
                .collect();
            std::fs::write(path, ram).map_err(|e| e.to_string())?;
        }
        if let Some(wav) = &mut self.wav {
            use std::io::{Seek, SeekFrom};
            wav.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            let size = self.sample_count * 2;
            let mut header = Vec::with_capacity(44);
            header.extend_from_slice(b"RIFF");
            header.extend_from_slice(&(size + 36).to_le_bytes());
            header.extend_from_slice(b"WAVEfmt ");
            header.extend_from_slice(&16u32.to_le_bytes());
            header.extend_from_slice(&1u16.to_le_bytes());
            header.extend_from_slice(&1u16.to_le_bytes());
            header.extend_from_slice(&nes_apu::SAMPLE_RATE.to_le_bytes());
            header.extend_from_slice(&(nes_apu::SAMPLE_RATE * 2).to_le_bytes());
            header.extend_from_slice(&2u16.to_le_bytes());
            header.extend_from_slice(&16u16.to_le_bytes());
            header.extend_from_slice(b"data");
            header.extend_from_slice(&size.to_le_bytes());
            wav.write_all(&header)
                .and_then(|_| wav.flush())
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
