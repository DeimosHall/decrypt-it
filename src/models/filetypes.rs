use std::slice::Iter;

#[derive(Clone, Copy, Debug, glib::Enum, PartialEq, Default, Eq, Hash)]
#[enum_type(name = "DecryptItFiletype")]
pub enum FileType {
    #[enum_value(name = "DLC")]
    Dlc,
    #[enum_value(name = "Unknown")]
    #[default]
    Unknown,
}

use FileType::*;

impl FileType {
    pub fn is_input(&self) -> bool {
        matches!(self, Dlc)
    }

    pub fn iterator() -> Iter<'static, Self> {
        static FILETYPES: [FileType; 1] = [Dlc];
        FILETYPES.iter()
    }

    pub fn input_formats() -> Iter<'static, Self> {
        static FILETYPES: [FileType; 1] = [Dlc];
        FILETYPES.iter()
    }

    pub fn as_mime(&self) -> &'static str {
        match self {
            Dlc => "application/x-dlc",
            Unknown => "",
        }
    }

    pub fn from_mimetype(mimetype: &str) -> Option<Self> {
        match mimetype {
            "application/x-dlc" => Some(Dlc),
            _ => None,
        }
    }

    pub fn as_extension(&self) -> &str {
        match self {
            Dlc => "dlc",
            Unknown => "",
        }
    }

    pub fn as_display_string(&self) -> String {
        self.as_extension().to_uppercase()
    }

    pub fn from_string(extension: &str) -> Option<Self> {
        match extension {
            "dlc" => Some(Dlc),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, glib::Enum, PartialEq, Eq, Hash)]
#[enum_type(name = "DecryptItCompressionType")]
pub enum CompressionType {
    #[enum_value(name = "ZIP")]
    Zip,
    #[enum_value(name = "Directory")]
    Directory,
}

use CompressionType::*;

impl CompressionType {
    pub fn is_compression(&self) -> bool {
        matches!(self, Zip)
    }

    pub fn iterator() -> Iter<'static, Self> {
        static COMPRESSION_TYPES: [CompressionType; 2] = [Zip, Directory];
        COMPRESSION_TYPES.iter()
    }

    pub fn compression_formats() -> Iter<'static, Self> {
        static COMPRESSION_TYPES: [CompressionType; 1] = [Zip];
        COMPRESSION_TYPES.iter()
    }

    pub fn possible_output(sandboxed: bool) -> Iter<'static, Self> {
        static COMPRESSION_TYPES: [CompressionType; 1] = [Zip];
        static ALL_TYPES: [CompressionType; 2] = [Zip, Directory];
        match sandboxed {
            true => COMPRESSION_TYPES.iter(),
            false => ALL_TYPES.iter(),
        }
    }

    pub fn as_mime(&self) -> &'static str {
        match self {
            Zip => "application/zip",
            Directory => "inode/directory",
        }
    }

    pub fn as_extension(&self) -> &str {
        match self {
            Zip => "zip",
            Directory => "directory",
        }
    }

    pub fn as_display_string(&self) -> String {
        match self {
            Directory => "Directory".to_owned(),
            x => x.as_extension().to_uppercase(),
        }
    }

    pub fn from_string(extension: &str) -> Option<Self> {
        match extension {
            "zip" => Some(Zip),
            "directory" => Some(Directory),
            _ => None,
        }
    }
}
