#![allow(unused)]
use iron_oxide::io::ByteWriter;

pub trait ClientPacket {
    fn serialize(&self) -> Vec<u8>;
    const PACKED_ID: u8;
}

pub struct FileManipulation {
    pub path: String,
    pub file_name: String,
    pub action: FileAction
}

#[repr(u8)]
pub enum FileAction {
    Rename(String) = 0,
    Delete = 1,
    Create = 2,
}

impl FileAction {
    fn to_u8(&self) -> u8 {
        match self {
            FileAction::Rename(_) => 0,
            FileAction::Delete => 1,
            FileAction::Create => 2,
        }
    }
}

impl ClientPacket for FileManipulation {
    fn serialize(&self) -> Vec<u8> {
        let mut writer = ByteWriter::new();
        writer.write_byte(Self::PACKED_ID as u8);

        writer.write_u32(self.path.len() as u32);
        writer.write_string(&self.path);

        writer.write_u32(self.file_name.len() as u32);
        writer.write_string(&self.file_name);

        writer.write_byte(self.action.to_u8());

        match &self.action {
            FileAction::Rename(new_name) => {
                writer.write_u32(new_name.len() as u32);
                writer.write_string(&new_name);
            },
            FileAction::Delete => todo!(),
            FileAction::Create => todo!(),
        }

        writer.finish()


    }

    const PACKED_ID: u8 = 5;
}