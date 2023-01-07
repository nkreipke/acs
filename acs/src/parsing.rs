use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use binread::{BinRead, BinReaderExt, BinResult, ReadOptions};

#[derive(BinRead, Debug)]
#[br(magic = 0xABCDABC3_u32)]
pub struct AcsHeader {
    pub character_info: AcsLocator<AcsCharacterInfo>,
    pub animation_info: AcsLocator<List32<AcsAnimationInfo>>,
    pub image_info: AcsLocator<List32<AcsImageInfo>>,
    pub audio_info: AcsLocator<List32<AcsAudioInfo>>
}

#[derive(BinRead, Debug)]
pub struct AcsLocator<T: BinRead<Args=()>> {
    offset: u32,
    size: u32,
    #[br(default)]
    _p: PhantomData<T>
}

#[derive(BinRead, Debug)]
pub struct AcsCharacterInfo {
    pub minor_version: u16,
    pub major_version: u16,
    pub localized_info: AcsLocator<u32>,
    pub guid: AcsGuid,
    pub char_width: u16,
    pub char_height: u16,
    pub transparent_color_index: u8,
    pub flags: AcsCharacterInfoFlags,
    pub animation_set_major_version: u16,
    pub animation_set_minor_version: u16,
    #[br(if(flags.contains(AcsCharacterInfoFlags::VOICE_OUTPUT_ENABLED)))]
    pub voice_info: Option<AcsVoiceInfo>,
    #[br(if(!flags.contains(AcsCharacterInfoFlags::WORD_BALLOON_DISABLED)))]
    pub balloon_info: Option<AcsBalloonInfo>,
    pub palette_colors: List32<PaletteColor>,
    pub tray_icon_flag: u8,
    #[br(if(tray_icon_flag == 0x1))]
    pub tray_icon: Option<TrayIcon>,
    pub states: List16<StateInfo>
}

#[derive(BinRead, Debug)]
pub struct AcsAnimationInfo {
    pub name: AcsString,
    pub entry: AcsLocator<AcsAnimationInfoEntry>
}

#[derive(BinRead, Debug)]
pub struct AcsAnimationInfoEntry {
    pub name: AcsString,
    pub transition_type: u8,
    pub return_animation: AcsString,
    pub frame_info: List16<AcsFrameInfo>
}

#[derive(BinRead, Debug)]
pub struct AcsFrameInfo {
    pub images: List16<AcsFrameImage>,
    pub audio_info_index: u16,
    pub frame_duration: u16,
    pub exit_frame_index: i16,
    pub branches: List8<BranchInfo>,
    pub mouth_overlays: List8<AcsOverlayInfo>
}

#[derive(BinRead, Debug)]
pub struct BranchInfo {
    pub frame_index: u16,
    pub probability: u16
}

#[derive(BinRead, Debug)]
pub struct AcsOverlayInfo {
    pub overlay_type: u8,
    pub replace_enabled: u8,
    pub image_info_index: u16,
    _unknown: u8,
    pub region_data_flag: u8,
    pub x_offset: i16,
    pub y_offset: i16,
    pub width: u16,
    pub height: u16,
    #[br(if(region_data_flag == 1))]
    pub region_data: Option<AcsDataBlock>
}

#[derive(BinRead, Debug)]
pub struct AcsFrameImage {
    pub image_info_index: u32,
    pub x_offset: i16,
    pub y_offset: i16
}

#[derive(BinRead, Debug)]
pub struct AcsImageInfo {
    pub location: AcsLocator<AcsImageInfoEntry>,
    pub checksum: u32
}

#[derive(BinRead, Debug)]
pub struct AcsImageInfoEntry {
    _unknown: u8,
    pub width: u16,
    pub height: u16,
    pub compression_flag: u8,
    pub data: AcsDataBlock,
    pub compressed_regdata_size: u32,
    pub uncompressed_regdata_size: u32,
    #[br(count = (if compressed_regdata_size == 0 { uncompressed_regdata_size } else { compressed_regdata_size }))]
    pub regdata: Vec<u8>
}

#[derive(BinRead, Debug)]
pub struct AcsAudioInfo {
    pub data: AcsLocator<()>,
    pub checksum: u32
}

#[derive(BinRead, Debug)]
pub struct AcsDataBlock {
    pub size: u32,
    #[br(count = size)]
    pub data: Vec<u8>
}

#[derive(BinRead, Debug)]
pub struct AcsVoiceInfo {
    pub tts_engine_id: AcsGuid,
    pub tts_mode_id: AcsGuid,
    pub speed: u32,
    pub pitch: u16,
    pub extra_data_flag: u8,
    #[br(if(extra_data_flag == 0x1))]
    pub extra_data: Option<AcsVoiceInfoExtraData>
}

#[derive(BinRead, Debug)]
pub struct AcsVoiceInfoExtraData {
    pub lang_id: AcsLangId,
    pub dialect: AcsString,
    pub gender: u16,
    pub age: u16,
    pub style: AcsString
}

#[derive(BinRead, Debug)]
pub struct AcsBalloonInfo {
    pub lines: u8,
    pub chars_per_line: u8,
    pub foreground_color: RgbQuad,
    pub background_color: RgbQuad,
    pub border_color: RgbQuad,
    pub font_name: AcsString,
    pub font_height: i32,
    pub font_wright: i32,
    pub italic: u8,
    _unused: u8
}

#[derive(BinRead, Debug)]
pub struct AcsLangId {
    pub id: u16
}

#[derive(BinRead)]
pub struct AcsString {
    _count: u32,
    #[br(count = _count)]
    chars: Vec<u16>,
    #[br(if(_count > 0))]
    _nullterm: Option<u16>
}

#[derive(BinRead, Debug)]
pub struct List8<T: BinRead<Args=()>> {
    _count: u8,
    #[br(count = _count)]
    pub items: Vec<T>
}

#[derive(BinRead, Debug)]
pub struct List16<T: BinRead<Args=()>> {
    _count: u16,
    #[br(count = _count)]
    pub items: Vec<T>
}

#[derive(BinRead, Debug)]
pub struct List32<T: BinRead<Args=()>> {
    _count: u32,
    #[br(count = _count)]
    pub items: Vec<T>
}

#[derive(BinRead, Debug)]
pub struct PaletteColor {
    pub color: RgbQuad
}

#[derive(BinRead, Debug)]
pub struct StateInfo {
    pub name: AcsString,
    pub animations: List16<AcsString>
}

#[derive(BinRead, Debug)]
pub struct BitmapInfoHeader {
    pub size: u32,
    pub width: i32,
    pub height: i32,
    pub planes: u16,
    pub bits_per_pixel: u16,
    pub compression_type: u32,
    pub image_data_size: u32,
    pub horiz_resolution: i32,
    pub vert_resolution: i32,
    pub color_index_count: u32,
    pub important_color_index_count: u32
}

#[derive(BinRead, Debug)]
pub struct IconImage {
    pub header: BitmapInfoHeader,
    #[br(count(header.color_index_count))]
    pub color_table: Vec<RgbQuad>,
    #[br(count(header.color_index_count))]
    pub xor_bits: Vec<u8>,
    #[br(count(header.color_index_count))]
    pub and_bits: Vec<u8>
}

#[derive(BinRead, Debug)]
pub struct TrayIcon {
    pub mono_size: u32,
    pub mono: IconImage,
    pub color_size: u32,
    pub color: IconImage
}

#[derive(BinRead, Debug)]
pub struct RgbQuad(pub u8, pub u8, pub u8, pub u8);

#[derive(BinRead)]
pub struct AcsGuid(pub u32, pub u16, pub u16, pub u64);

bitflags::bitflags! {
    #[repr(transparent)]
    pub struct AcsCharacterInfoFlags: u32 {
        const VOICE_OUTPUT_ENABLED = 1 << 4;
        const WORD_BALLOON_ENABLED = 1 << 8;
        const WORD_BALLOON_DISABLED = 1 << 9;
        const SIZE_TO_TEXT_ENABLED = 1 << 16;
        const AUTO_HIDE_DISABLED = 1 << 17;
        const AUTO_PACE_DISABLED = 1 << 18;
        const STANDARD_ANIMATION_SET_SUPPORTED = 1 << 20;
    }
}

impl BinRead for AcsCharacterInfoFlags {
    type Args = ();

    fn read_options<R: Read + Seek>(reader: &mut R, _options: &ReadOptions, args: Self::Args) -> BinResult<Self> {
        let bits = reader.read_be_args(args)?;

        // Additional flags are unknown and unused, therefore this is safe
        Ok(unsafe { AcsCharacterInfoFlags::from_bits_unchecked(bits) })
    }
}

impl<T: BinRead<Args=()>> AcsLocator<T> {
    pub fn get(&self, mut reader: impl BinReaderExt) -> BinResult<T> {
        // We don't use FilePtr32<T> because it eager-loads all data, which takes way too long.
        reader.seek(SeekFrom::Start(self.offset as u64))?;
        reader.read_le()
    }

    pub fn read_bytes(&self, mut reader: (impl Read + Seek), target: &mut Vec<u8>) -> std::io::Result<()> {
        let size = self.size as usize;
        let capacity = target.capacity();
        if capacity < size {
            target.reserve(size - capacity);
        }

        reader.seek(SeekFrom::Start(self.offset as u64))?;
        reader.take(self.size as u64).read_to_end(target)?;

        Ok(())
    }
}

impl std::fmt::Debug for AcsGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X}-{:04X}-{:04X}-{:016X}", self.0, self.1, self.2, self.3)
    }
}

impl std::fmt::Debug for AcsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for AcsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in std::char::decode_utf16(self.chars.iter().cloned()) {
            write!(f, "{}", c.unwrap_or(std::char::REPLACEMENT_CHARACTER))?;
        }

        Ok(())
    }
}