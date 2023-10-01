#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub c: u8,
    pub id: u8,
    pub n: u8,
    pub bytes: Vec<u8>,
}

impl Request {
    pub fn heartbeat() -> Self {
        Self::from_command_byte(0xFF, &[])
    }

    pub fn video_start() -> Self {
        Self::from_u32s(4, [1])
    }

    pub fn audio_start() -> Self {
        Self::from_command_byte(8, [1])
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        [
            0x4D, 0x4F, 0x5F, self.c, self.id, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, self.n, 0, 0, 0, 0, 0,
            0, 0,
        ]
        .into_iter()
        .chain(self.bytes.iter().copied())
        .collect::<Vec<_>>()
    }

    pub fn from_command_byte<'a, B: AsRef<[u8]>>(id: u8, bytes: B) -> Self {
        Self {
            c: 0x4F,
            id,
            n: bytes.as_ref().len() as u8,
            bytes: bytes.as_ref().to_vec(),
        }
    }

    pub fn from_device_control(a: u8, b: u8) -> Self {
        Self::from_command_byte(0xFA, &[a, b])
    }

    pub fn from_camera_request(request: u8) -> Self {
        Self::from_command_byte(14, [request])
    }

    pub fn from_u32s<'a, B: AsRef<[u32]>>(id: u8, ints: B) -> Self {
        Self::from_command_byte(
            id,
            ints.as_ref()
                .into_iter()
                .flat_map(|i| i.to_le_bytes())
                .collect::<Vec<_>>(),
        )
    }
}
