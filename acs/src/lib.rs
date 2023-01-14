use std::fmt::{Debug, Formatter};
use std::io::Cursor;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use binread::BinReaderExt;

mod parsing;
mod compression;
mod bit_reader;

use parsing::*;
pub use parsing::AcsString;
use crate::compression::decompress;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AcsError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    BinRead(#[from] binread::Error),
    #[error("invalid compressed data: {0}")]
    InvalidCompressedData(&'static str)
}

pub type AcsResult<T> = Result<T, AcsError>;

pub struct AcsFile<D: AsRef<[u8]>> {
    data: D,
    character: AcsCharacterInfo,
    animations: List32<AcsAnimationInfo>,
    images: List32<AcsImageInfo>,
    audio: List32<AcsAudioInfo>
}

pub struct AcsAnimation {
    info: AcsAnimationInfoEntry,
}

pub struct AcsFrame<'b> {
    info: &'b AcsFrameInfo
}

pub struct AcsFrameImage<'b> {
    info: &'b parsing::AcsFrameImage
}

pub struct AcsImage<'a, D: AsRef<[u8]>> {
    file: &'a AcsFile<D>,
    info: AcsImageInfoEntry,
    decompressed_data: Vec<u8>
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct AcsImagePixel(u32); // represented as argb

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AcsImageIndex(u32);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AcsAudioIndex(u16);

impl AcsFile<Vec<u8>> {
    pub fn open_path(path: impl AsRef<Path>) -> AcsResult<Self> {
        use std::io::Read;

        let mut file = vec![];
        std::fs::File::open(path)
            .unwrap()
            .read_to_end(&mut file)
            .unwrap();

        AcsFile::open(file)
    }
}

impl<D: AsRef<[u8]>> AcsFile<D> {
    pub fn open(data: D) -> AcsResult<Self> {
        let mut cursor = Cursor::new(&data);

        let header: AcsHeader = cursor.read_le()?;
        let character = header.character_info.get(&mut cursor)?;
        let animations = header.animation_info.get(&mut cursor)?;
        let images = header.image_info.get(&mut cursor)?;
        let audio = header.audio_info.get(&mut cursor)?;

        Ok(AcsFile {
            data,
            character,
            animations,
            images,
            audio
        })
    }

    pub fn animations(&self) -> impl Iterator<Item = AcsResult<AcsAnimation>> + '_ {
        self.animations.items.iter().map(|info| Ok(AcsAnimation {
            info: info.entry.get(self.cursor())?
        }))
    }

    pub fn image(&self, index: AcsImageIndex) -> AcsResult<AcsImage<D>> {
        let mut image = AcsImage {
            file: self,
            info: self.images.items[index.0 as usize].location.get(self.cursor())?,
            decompressed_data: vec![]
        };

        image.decompress()?;

        Ok(image)
    }

    pub fn audio(&self, index: AcsAudioIndex, target: &mut Vec<u8>) -> AcsResult<()> {
        let data = &self.audio.items[index.0 as usize].data;

        data.read_bytes(self.cursor(), target)?;

        Ok(())
    }

    pub fn char_size(&self) -> (u16, u16) {
        (self.character.char_width, self.character.char_height)
    }

    fn cursor(&self) -> Cursor<&D> {
        Cursor::new(&self.data)
    }
}

impl AcsAnimation {
    pub fn name(&self) -> &AcsString {
        &self.info.name
    }

    pub fn return_animation(&self) -> &AcsString {
        &self.info.return_animation
    }

    pub fn frames(&self) -> AcsResult<impl Iterator<Item = AcsFrame>> {
        Ok(self.info.frame_info.items.iter().map(|info| AcsFrame {
            info
        }))
    }
}

impl<'a> AcsFrame<'a> {
    pub fn duration(&self) -> Duration {
        // Specified in 1/100 seconds
        Duration::from_millis(self.info.frame_duration as u64 * 10)
    }

    pub fn images(&self) -> AcsResult<impl Iterator<Item = AcsFrameImage>> {
        Ok(self.info.images.items.iter().map(|info| AcsFrameImage {
            info
        }))
    }

    pub fn audio_index(&self) -> Option<AcsAudioIndex> {
        if self.info.audio_info_index == 0xFFFF {
            None
        } else {
            Some(AcsAudioIndex(self.info.audio_info_index))
        }
    }
}

impl<'b> AcsFrameImage<'b> {
    pub fn offset(&self) -> (i16, i16) {
        (self.info.x_offset, self.info.y_offset)
    }

    pub fn image_index(&self) -> AcsImageIndex {
        AcsImageIndex(self.info.image_info_index)
    }
}

impl<'a, D: AsRef<[u8]>> AcsImage<'a, D> {
    pub fn size(&self) -> (u16, u16) {
        (self.info.width, self.info.height)
    }

    pub fn pixel(&self, x: u16, y: u16) -> AcsImagePixel {
        let (width, height) = self.size();

        assert!(x < width);
        assert!(y < height);

        let character = &self.file.character;
        let color_table_index = self.data()[(x + (height - y - 1) * width) as usize] as usize;

        if color_table_index == character.transparent_color_index as usize {
            AcsImagePixel::zero()
        } else {
            let RgbQuad(b, g, r, _) = character.palette_colors.items[color_table_index].color;

            AcsImagePixel::new(0xFF, r, g, b)
        }
    }

    pub fn read_argb(&self, target: &mut Vec<u32>) {
        let (width, height) = self.size();

        for y in 0..height {
            for x in 0..width {
                target.push(self.pixel(x, y).as_argb());
            }
        }
    }

    fn data(&self) -> &[u8] {
        if self.decompressed_data.is_empty() {
            &self.info.data.data
        } else {
            &self.decompressed_data
        }
    }

    fn decompress(&mut self) -> AcsResult<()> {
        if self.info.compression_flag != 0 {
            decompress(&self.info.data.data, &mut self.decompressed_data)?;
        }

        Ok(())
    }
}

impl<D: AsRef<[u8]>> Debug for AcsFile<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "character (width={},height={}), {} animations, {} images, {} waveforms",
            self.character.char_width,
            self.character.char_height,
            self.animations.items.len(),
            self.images.items.len(),
            self.audio.items.len())
    }
}

impl Debug for AcsAnimation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "animation {}, {} frames", self.info.name, self.info.frame_info.items.len())
    }
}

impl<'a> Debug for AcsFrame<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "frame with duration {:?}, {} images", self.duration(), self.info.images.items.len())
    }
}

impl<'b> Debug for AcsFrameImage<'b> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "image {:?} at (x={},y={})", self.image_index(), self.info.x_offset, self.info.y_offset)
    }
}

impl<'a, D: AsRef<[u8]>> Debug for AcsImage<'a, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "image (width={},height={}), {} bytes", self.info.width, self.info.height, self.data().len())
    }
}

impl AcsImagePixel {
    pub fn new(a: u8, r: u8, g: u8, b: u8) -> AcsImagePixel {
        AcsImagePixel((a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | b as u32)
    }

    pub fn zero() -> AcsImagePixel {
        AcsImagePixel(0)
    }

    pub fn a(self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn r(self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn g(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn b(self) -> u8 {
        self.0 as u8
    }

    pub fn as_argb(self) -> u32 {
        self.0
    }
}