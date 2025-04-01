
use std::{fs::{self, File}, io::{self, Write}, net::SocketAddr, sync::{Arc, RwLock}};
use iron_oxide::{io::ByteWriter, net::{MessageDataType, WebSocket, WebSocketInterface}};

use crate::{server::Engine, server_packet::{RequestDirectory, ServerPacket, UploadFilePacket}, SAVE_LOCATION};


#[allow(unused)]
pub struct Client {
    pub ws: WebSocket,
    pub engine: Arc<RwLock<Engine>>,
    pub current_tab: Tab,
    pub current_path: String,
}

impl Client {
    pub fn new(engine: Arc<RwLock<Engine>>, ws: WebSocket) -> Self {
        Self { ws, engine, current_tab: Tab::Dashboard, current_path: String::with_capacity(0) }
    }

    fn get_directory(&mut self, packet: RequestDirectory) {
        println!("requestet_path: {}", packet.path);
        let mut response = ByteWriter::new();
        match fs::read_dir(SAVE_LOCATION.to_string() + &packet.path) {
            Ok(entries) => {
                response.write_byte(1);
                for entry in entries {
                    if let Ok(entry) = entry {
                        let name = entry.file_name().into_string().unwrap();
                        let is_dir = entry.file_type().unwrap().is_dir();
                        response.write_byte(if is_dir { 1 } else { 0 });
                        response.write_u32(name.len() as u32);
                        response.write_string(&name);
                    }
                }
                self.ws.send(response.as_ref(), MessageDataType::Binary);
            },
            Err(e) => println!("error: {e}")
        }
    }

    fn create_new_file(&mut self, packet: &UploadFilePacket) -> io::Result<File> {

        println!("{}", SAVE_LOCATION.to_string() + &packet.file_name + &packet.path);

        let file = fs::File::create(SAVE_LOCATION.to_string() + &packet.path + &packet.file_name)?;
        Ok(file)
    }

    fn upload_file(&mut self, data: Vec<u8>) {
        let packet = UploadFilePacket::parse(&data[1..]);

        match self.create_new_file(&packet) {
            Ok(mut file) => {
                match file.write(&data[packet.file_content_start..]) {
                    Ok(_) => self.validate_file(true, packet.file_name, packet.path,String::with_capacity(0)),
                    Err(e) => self.validate_file(false, packet.file_name, packet.path,e.to_string()),
                }
            },
            Err(e) => self.validate_file(false, packet.file_name, packet.path, e.to_string()),
        }
    }

    fn validate_file(&mut self, succes: bool, name: String, path: String, error: String) {
        let mut writer = ByteWriter::with_capacity(name.len() + 5);
        writer.write_byte(4);
        writer.write_bool(succes);
        writer.write_u32(name.len() as u32);
        writer.write_string(&name);
        writer.write_u32(path.len() as u32);
        writer.write_string(&path);
        if !succes {
            writer.write_u32(error.len() as u32);
            writer.write_string(&error);
        }
        self.ws.send(writer.as_mut(), MessageDataType::Binary);
    }

    fn handle_packet(&mut self, data: Vec<u8>) {
        match data[0] {
            RequestDirectory::PACKED_ID => {
                let packet = RequestDirectory::parse(&data[1..]);
                self.get_directory(packet);
            }
            UploadFilePacket::PACKED_ID => {
                println!("efef");
                self.upload_file(data)
            }
            _ => println!("not supported yet!: {}", String::from_utf8_lossy(&data)),
        }
    }
}

impl WebSocketInterface for Client {
    fn on_message(&mut self, data: Vec<u8>) {
        self.handle_packet(data);
    }

    fn on_closed(&self, ip: SocketAddr) {
        let mut engine = self.engine.write().unwrap();
        engine.remove_client(ip);
    }

    fn websocket_mut(&mut self) -> &mut WebSocket {
        &mut self.ws
    }

    fn websocket(&self) -> &WebSocket {
        &self.ws
    }
}

#[allow(unused)]
pub enum Tab {
    Dashboard,
    FileSystem
}