use super::request::Request;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Neutral,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalDirection {
    Left,
    Neutral,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Slow,
    Fast,
}

impl Speed {
    fn encode(self) -> u8 {
        match self {
            Speed::Slow => 1,
            Speed::Fast => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Camera {
    Driving,
    Turret,
}

impl Camera {
    fn encode(self) -> u8 {
        match self {
            Camera::Driving => 2,
            Camera::Turret => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalDirection {
    Up,
    Neutral,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Drive(Direction, HorizontalDirection, Speed),
    SteerStop(Speed),
    UseCamera(Camera),
    CameraMoveHorizontal(HorizontalDirection),
    CameraMoveVertical(VerticalDirection),
    StealthMode(bool),
}

impl Command {
    pub fn to_request(self) -> Request {
        match self {
            Command::Drive(dir, steer, speed) => {
                let cmd = match dir {
                    Direction::Forward => match steer {
                        HorizontalDirection::Left => 6,
                        HorizontalDirection::Neutral => 1,
                        HorizontalDirection::Right => 7,
                    },
                    Direction::Neutral => match steer {
                        HorizontalDirection::Left => 5,
                        HorizontalDirection::Neutral => 0,
                        HorizontalDirection::Right => 4,
                    },
                    Direction::Backward => match steer {
                        HorizontalDirection::Left => 8,
                        HorizontalDirection::Neutral => 2,
                        HorizontalDirection::Right => 9,
                    },
                };

                Request::from_device_control(cmd, speed.encode())
            }
            Command::SteerStop(speed) => Request::from_device_control(3, speed.encode()),
            Command::UseCamera(camera) => Request::from_command_byte(19, [6, camera.encode()]),
            Command::CameraMoveHorizontal(steer) => Request::from_camera_request(match steer {
                HorizontalDirection::Left => 4,
                HorizontalDirection::Neutral => 5,
                HorizontalDirection::Right => 6,
            }),
            Command::CameraMoveVertical(steer) => Request::from_camera_request(match steer {
                VerticalDirection::Up => 0,
                VerticalDirection::Neutral => 1,
                VerticalDirection::Down => 2,
            }),
            Command::StealthMode(enable) => {
                Request::from_camera_request(if enable { 94 } else { 95 })
            }
        }
    }
}

/*
fn send_command_int_request(sock: &mut TcpStream, id: u8, ints: &[u32]) -> anyhow::Result<()> {
    let mut bytevals = vec![];
    for &i in ints {
        bytevals.extend(i.to_le_bytes());
    }
    send_command_request(sock, id, (4 * ints.len()) as u8, bytevals.as_slice())
}
*/
