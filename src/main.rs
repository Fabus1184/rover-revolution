use log::{trace, Level};
use openh264::decoder::Decoder;
use sdl2::{
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    render::TextureAccess,
};

use crate::rover::{
    media::StreamPacket, Camera, Command, Direction, HorizontalDirection, Rover, Speed,
    VerticalDirection,
};

mod rover;

fn main() {
    simple_logger::init_with_level(Level::Trace).unwrap();

    let (mut rover, frame_receiver) = Rover::init().unwrap();
    let mut steer = HorizontalDirection::Neutral;
    let mut direction = Direction::Neutral;
    let mut stealth = false;
    let speed = Speed::Fast;

    rover
        .send_command(Command::Drive(direction, steer, speed))
        .unwrap();

    let context = sdl2::init().unwrap();
    let video = context.video().unwrap();
    let window = video
        .window("Rover Revolution", 320 * 4, 240 * 4)
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let textuer_creator = canvas.texture_creator();

    let mut event_pump = context.event_pump().unwrap();

    let mut texture1 = textuer_creator
        .create_texture(PixelFormatEnum::YV12, TextureAccess::Streaming, 640, 480)
        .unwrap();
    let mut texture2 = textuer_creator
        .create_texture(PixelFormatEnum::YV12, TextureAccess::Streaming, 320, 240)
        .unwrap();

    let mut decoder = Decoder::new().unwrap();

    'lop: loop {
        if let Ok(packet) = frame_receiver.try_recv() {
            trace!("packet: {:?}", packet);

            match packet {
                StreamPacket::Audio {
                    data,
                    offset,
                    index,
                    ..
                } => {
                    //let x = rover::adpcm::adpcm_to_pcm(data.as_slice(), offset, index);
                    //println!("{x:?}")
                }
                StreamPacket::Video {
                    video_type, data, ..
                } => {
                    if let Some(frame) = decoder.decode(data.as_slice()).unwrap() {
                        let texture = match video_type {
                            1 => &mut texture1,
                            2 => &mut texture2,
                            _ => todo!(),
                        };

                        texture
                            .update_yuv(
                                None,
                                frame.y_with_stride(),
                                frame.strides_yuv().0,
                                frame.u_with_stride(),
                                frame.strides_yuv().1,
                                frame.v_with_stride(),
                                frame.strides_yuv().2,
                            )
                            .unwrap();

                        canvas.set_draw_color(Color::RGB(20, 20, 20));
                        canvas.clear();

                        canvas.copy(texture, None, None).unwrap();

                        canvas.present();
                    }
                }
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => match keycode {
                    Keycode::Q => break 'lop,
                    Keycode::Num1 => {
                        rover
                            .send_command(Command::UseCamera(Camera::Driving))
                            .unwrap();
                    }
                    Keycode::Num2 => {
                        rover
                            .send_command(Command::UseCamera(Camera::Turret))
                            .unwrap();
                    }
                    Keycode::W => {
                        direction = Direction::Forward;
                        rover
                            .send_command(Command::Drive(direction, steer, speed))
                            .unwrap();
                    }
                    Keycode::S => {
                        direction = Direction::Backward;
                        rover
                            .send_command(Command::Drive(direction, steer, speed))
                            .unwrap();
                    }
                    Keycode::A => {
                        steer = HorizontalDirection::Left;
                        rover
                            .send_command(Command::Drive(direction, steer, speed))
                            .unwrap();
                    }
                    Keycode::D => {
                        steer = HorizontalDirection::Right;
                        rover
                            .send_command(Command::Drive(direction, steer, speed))
                            .unwrap();
                    }
                    Keycode::Up => {
                        rover
                            .send_command(Command::CameraMoveVertical(VerticalDirection::Up))
                            .unwrap();
                    }
                    Keycode::Down => {
                        rover
                            .send_command(Command::CameraMoveVertical(VerticalDirection::Down))
                            .unwrap();
                    }
                    Keycode::Left => {
                        rover
                            .send_command(Command::CameraMoveHorizontal(HorizontalDirection::Left))
                            .unwrap();
                    }
                    Keycode::Right => {
                        rover
                            .send_command(Command::CameraMoveHorizontal(HorizontalDirection::Right))
                            .unwrap();
                    }
                    Keycode::E => {
                        stealth ^= true;
                        rover.send_command(Command::StealthMode(stealth)).unwrap();
                    }

                    _ => {}
                },
                sdl2::event::Event::KeyUp {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => match keycode {
                    Keycode::W | Keycode::S => {
                        direction = Direction::Neutral;
                        rover
                            .send_command(Command::Drive(
                                direction,
                                HorizontalDirection::Neutral,
                                speed,
                            ))
                            .unwrap();
                        rover
                            .send_command(Command::Drive(direction, steer, speed))
                            .unwrap();
                    }
                    Keycode::A if steer == HorizontalDirection::Left => {
                        steer = HorizontalDirection::Neutral;
                        rover.send_command(Command::SteerStop(speed)).unwrap();
                    }
                    Keycode::D if steer == HorizontalDirection::Right => {
                        steer = HorizontalDirection::Neutral;
                        rover.send_command(Command::SteerStop(speed)).unwrap();
                    }
                    Keycode::Up | Keycode::Down => {
                        rover
                            .send_command(Command::CameraMoveVertical(VerticalDirection::Neutral))
                            .unwrap();
                    }
                    Keycode::Left | Keycode::Right => {
                        rover
                            .send_command(Command::CameraMoveHorizontal(
                                HorizontalDirection::Neutral,
                            ))
                            .unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
