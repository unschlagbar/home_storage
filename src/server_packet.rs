use iron_oxide::io::ByteReader;

pub trait ServerPacket {
    fn parse(data: &[u8]) -> Self;
    const PACKED_ID: u8;
}


pub struct RequestDirectory {
    pub path: String,
}

impl ServerPacket for RequestDirectory {
    fn parse(data: &[u8]) -> Self {
        let mut reader = ByteReader::new(data);
        let len = reader.read_le_u32();
        let path = reader.read_string(len as usize);

        Self { path }
    }

    const PACKED_ID: u8 = 0;
}

pub struct UploadFilePacket {
    pub file_name: String,
    pub path: String,
    pub file_content_start: usize
}

impl ServerPacket for UploadFilePacket {
    fn parse(data: &[u8]) -> Self {
        let mut reader = ByteReader::new(data);
        let name_len = reader.read_le_u32();
        let file_name = reader.read_string(name_len as usize);
        let path_len = reader.read_le_u32();
        let path = reader.read_string(path_len as usize);
        let file_content_start = reader.position() + 1;

        Self { file_name, path, file_content_start }
    }

    const PACKED_ID: u8 = 3;
}

