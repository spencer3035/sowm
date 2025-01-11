const PACKET_VERSION: [u8; 4] = ['v' as u8, '0' as u8, '.' as u8, '1' as u8];

/// A simple packet to be sent over a socket.
///
/// A packet contains an 8 byte header followed by an N bit data section.
///
/// The header is 8 bytes defined as the following:
///
///   0   1   2   3   4   5   6   7
///  'v' '0' '.' '1'  L   H   R   R
///
///  Where bytes 0-3 are version string 'v1.0' in ascii.
///
///  Bytes 4 and 5 contain the low and high bytes of a u16 integer describing the length of the
///  data part of the packet, assumed to be in little endian. N = L | (H << 8)
///
///  Bytes 7 and 8 are currently reserved and should be set to 0.
///
///  The remainder of the packet is the data, which should be exactly equal in length defined in
///  the header
pub struct Packet {
    header: [u8; 8],
    data: Vec<u8>,
}

impl Packet {
    pub fn into_bytes(self) -> Vec<u8> {
        let (mut header, mut data) = (self.header.to_vec(), self.data);
        header.append(&mut data);
        header
    }

    pub fn new(data: Vec<u8>) -> Self {
        if data.len() > u16::MAX as usize {}

        // TODO: Make achievement for exceeding this limit
        let len: u16 = data
            .len()
            .try_into()
            .expect("Data is too much to be stored as u16");

        // TODO: Make achievement for running on big endian system
        // Assumes little endian
        let header = [
            PACKET_VERSION[0],
            PACKET_VERSION[1],
            PACKET_VERSION[2],
            PACKET_VERSION[3],
            (len & 0x00FF) as u8,
            ((len & 0xFF00) >> 8) as u8,
            0,
            0,
        ];

        Packet { header, data }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn len_from_header(header: &[u8; 8]) -> Result<usize, PacketError> {
        let good_version = header[0] == PACKET_VERSION[0]
            && header[1] == PACKET_VERSION[1]
            && header[2] == PACKET_VERSION[2]
            && header[3] == PACKET_VERSION[3];

        if !good_version {
            return Err(PacketError::BadVersion);
        }

        if header[6] != 0 || header[7] != 0 {
            return Err(PacketError::ReservedNotZero);
        }

        let len_low = header[4] as usize;
        let len_high = header[5] as usize;
        Ok(len_low | len_high << 8)
    }
}

#[derive(Debug)]
pub enum PacketError {
    BadVersion,
    ReservedNotZero,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn packet_lengths() {
        let len = 200;
        let data = vec![0; len];
        let packet = Packet::new(data);
        let len_after = Packet::len_from_header(&packet.header).expect("Header wasn't valid");
        assert_eq!(len, len_after);

        let len = 402;
        let data = vec![0; len];
        let packet = Packet::new(data);
        let len_after = Packet::len_from_header(&packet.header).expect("Header wasn't valid");
        assert_eq!(len, len_after);

        let len = 400;
        let data = vec![0; len];
        let packet = Packet::new(data);
        let len_after = Packet::len_from_header(&packet.header).expect("Header wasn't valid");
        assert_eq!(len, len_after);
    }

    #[test]
    #[should_panic]
    fn bad_packet_lengths() {
        let len = 10000000;
        let data = vec![0; len];
        let packet = Packet::new(data);
        let len_after = Packet::len_from_header(&packet.header).expect("Header wasn't valid");
        assert_eq!(len, len_after);
    }
}
