use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpStream},
    sync::mpsc::Receiver,
    thread::JoinHandle,
    time::Duration,
};

use blowfish::cipher::{generic_array::GenericArray, typenum::U8, BlockEncrypt, KeyInit};
use log::info;

use crate::rover::media::StreamPacket;

use self::request::Request;

pub mod adpcm;
mod command;
pub mod media;
mod request;

pub use command::*;

const IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 100);
const PORT: u16 = 80;

const TARGET_ID: &str = "AC13";
const TARGET_PASSWORD: &str = "AC13";

pub struct Rover {
    command_socket: TcpStream,
    heartbeat_thread: JoinHandle<()>,
    media_socket: TcpStream,
    media_thread: JoinHandle<()>,
}

impl Rover {
    pub fn init() -> anyhow::Result<(Self, Receiver<StreamPacket>)> {
        let mut command_socket = TcpStream::connect((IP, PORT))
            .map_err(|e| anyhow::anyhow!("failed to connect socket: {}", e))?;
        command_socket.set_read_timeout(Some(Duration::from_secs_f32(5.0)))?;
        command_socket.set_write_timeout(Some(Duration::from_secs_f32(5.0)))?;

        socket_send(&mut command_socket, Request::from_u32s(0, &[0, 0, 0, 0]))?;

        //let reply = receive_command_reply(&mut socket, 82).unwrap();
        let reply = socket_receive(&mut command_socket, 82)?;
        let camera_id = String::from_utf8(reply[25..37].to_vec()).unwrap();

        let key = format!("{TARGET_ID}:{camera_id}-save-private:{TARGET_PASSWORD}");

        let [l1, r1, l2, r2]: [i32; 4] = reply[66..]
            .chunks_exact(4)
            .map(|c| i32::from_le_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        let blowfish = blowfish::BlowfishLE::new_from_slice(key.as_bytes()).unwrap();

        let l1 = l1.to_le_bytes();
        let r1 = r1.to_le_bytes();
        let mut l1r1 = [l1[0], l1[1], l1[2], l1[3], r1[0], r1[1], r1[2], r1[3]];
        let x: &mut GenericArray<u8, U8> = l1r1.as_mut_slice().try_into().unwrap();
        blowfish.encrypt_block(x);
        let l1 = u32::from_le_bytes(l1r1[0..4].try_into().unwrap());
        let r1 = u32::from_le_bytes(l1r1[4..].try_into().unwrap());

        let l2 = l2.to_le_bytes();
        let r2 = r2.to_le_bytes();
        let mut l2r2 = [l2[0], l2[1], l2[2], l2[3], r2[0], r2[1], r2[2], r2[3]];
        let x: &mut GenericArray<u8, U8> = l2r2.as_mut_slice().try_into().unwrap();
        blowfish.encrypt_block(x);
        let l2 = u32::from_le_bytes(l2r2[0..4].try_into().unwrap());
        let r2 = u32::from_le_bytes(l2r2[4..].try_into().unwrap());

        socket_send(
            &mut command_socket,
            Request::from_u32s(2, &[l1, r1, l2, r2]),
        )?;

        let _ = socket_receive(&mut command_socket, 26)?;

        let heartbeat_thread = {
            let mut command_socket = command_socket.try_clone().unwrap();
            std::thread::spawn(move || {
                socket_send(&mut command_socket, Request::heartbeat()).unwrap();
                std::thread::sleep(std::time::Duration::from_secs(60))
            })
        };

        // video start request
        socket_send(&mut command_socket, Request::video_start())
            .map_err(|e| anyhow::anyhow!("Failed to send video start request: {}", e))?;

        let video_start_reply = socket_receive(&mut command_socket, 29)?;

        let mut media_socket = TcpStream::connect((IP, PORT))
            .map_err(|e| anyhow::anyhow!("failed to connect socket: {}", e))?;
        media_socket.set_read_timeout(Some(Duration::from_secs_f32(5.0)))?;
        media_socket.set_write_timeout(Some(Duration::from_secs_f32(5.0)))?;

        socket_send(
            &mut media_socket,
            Request {
                c: 0x56,
                id: 0,
                n: 4,
                bytes: video_start_reply[25..].to_vec(),
            },
        )?;

        let (tx, rx) = std::sync::mpsc::channel();
        let media_thread = {
            let media_socket = media_socket.try_clone().unwrap();
            std::thread::spawn(media::media_loop(media_socket, tx))
        };

        socket_send(&mut command_socket, Request::audio_start())?;

        let _ = socket_receive(&mut command_socket, 25)?;

        Ok((
            Rover {
                command_socket,
                heartbeat_thread,
                media_socket,
                media_thread,
            },
            rx,
        ))
    }

    pub fn send_command(&mut self, command: Command) -> anyhow::Result<()> {
        info!("sending {command:?}");
        socket_send(&mut self.command_socket, command.to_request())
    }
}

fn socket_receive(sock: &mut TcpStream, len: usize) -> anyhow::Result<Vec<u8>> {
    let mut buf = Vec::new();
    buf.resize(len, 0);
    sock.read_exact(&mut buf)
        .map_err(|e| anyhow::anyhow!("Failed to read from socket: {e}"))?;
    Ok(buf)
}

fn socket_send(sock: &mut TcpStream, request: Request) -> anyhow::Result<()> {
    sock.write_all(request.to_bytes().as_slice())
        .map_err(|e| anyhow::anyhow!("Failed to write to socket: {e}"))
}
