use std::net::TcpStream;

use super::socket_receive;

#[derive(Clone, PartialEq, Eq)]
pub enum StreamPacket {
    Video {
        length: i32,
        video_type: u8,
        video_length: i32,
        timestamp: u32,
        data: Vec<u8>,
    },
    Audio {
        length: i32,
        audio_length: i32,
        offset: i16,
        index: u8,
        timestamp: u32,
        data: Vec<u8>,
    },
}

impl std::fmt::Debug for StreamPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Video {
                length,
                video_type,
                video_length,
                timestamp,
                data,
            } => f
                .debug_struct("Video")
                .field("length", length)
                .field("video_type", video_type)
                .field("video_length", video_length)
                .field("timestamp", timestamp)
                .field("data.len()", &data.len())
                .finish(),
            Self::Audio {
                length,
                audio_length,
                offset,
                index,
                timestamp,
                data,
            } => f
                .debug_struct("Audio")
                .field("length", length)
                .field("audio_length", audio_length)
                .field("offset", offset)
                .field("index", index)
                .field("timestamp", timestamp)
                .field("data.len()", &data.len())
                .finish(),
        }
    }
}

pub fn media_loop(
    mut media_socket: TcpStream,
    tx: std::sync::mpsc::Sender<StreamPacket>,
) -> impl FnMut() {
    move || {
        let mut buf1 = vec![0_u8; 204800];
        let buf3 = [0x4D, 0x4F, 0x5F, 0x56];

        'label: loop {
            let bytes = socket_receive(&mut media_socket, 23).unwrap();

            let mut length = 1;
            for mut k in 0_i16.. {
                if k as i32 >= 4 {
                    if length != 0 {
                        k = i16::from_le_bytes(bytes[4..6].try_into().unwrap());
                        length = i32::from_le_bytes(bytes[15..19].try_into().unwrap());

                        if length > 204800 {
                            buf1.resize(length as usize, 0);
                        }

                        let bytes = socket_receive(&mut media_socket, length as usize).unwrap();

                        let timestamp = u32::from_le_bytes(bytes[..4].try_into().unwrap());
                        let packet = match k {
                            1 => {
                                let video_type = bytes[8];
                                let video_length =
                                    i32::from_le_bytes(bytes[9..13].try_into().unwrap());
                                StreamPacket::Video {
                                    length,
                                    video_type,
                                    video_length,
                                    timestamp,
                                    data: bytes[13..13 + video_length as usize].to_vec(),
                                }
                            }
                            2 => {
                                let audio_length =
                                    i32::from_le_bytes(bytes[13..17].try_into().unwrap());
                                /*let sampend = 40 + audio_length;
                                let offset = i16::from_le_bytes(
                                    bytes[sampend as usize..sampend as usize + 2]
                                        .try_into()
                                        .unwrap(),
                                );
                                let index = bytes[sampend as usize + 2];*/
                                StreamPacket::Audio {
                                    length,
                                    audio_length,
                                    offset: 0,
                                    index: 0,
                                    timestamp,
                                    data: bytes[17..17 + audio_length as usize].to_vec(),
                                }
                            }
                            _ => todo!(),
                        };

                        tx.send(packet).unwrap();

                        // System.arraycopy(bytes, p2 + 13, this.bArrayImage, 0, this.Video_Data_iVideoLen);
                        // AppDecodeH264.sessionDataCallBack(this.bArrayImage, this.Video_Data_iVideoLen, this.CurrentVideoType);

                        continue 'label;
                    }

                    continue 'label;
                }

                if bytes[k as usize] != buf3[k as usize] {
                    length = 0;
                }
            }

            break;
        }
    }
}
