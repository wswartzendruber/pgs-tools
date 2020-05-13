mod seg {

    use std::{
        io::{Error as IoError, Read},
        result::Result,
    };
    use byteorder::{BigEndian, ReadBytesExt};
    use thiserror::Error as ThisError;

    pub type SegResult<T> = Result<T, SegError>;

    #[derive(ThisError, Debug)]
    pub enum SegError {
        #[error("segment IO error")]
        PrematureEof {
            #[from]
            source: IoError,
        },
        #[error("segment has unrecognized magic number")]
        UnrecognizedMagicNumber,
        #[error("segment has unrecognized kind")]
        UnrecognizedKind,
    }

    pub enum SegType {
        Pds,
        Ods,
        Pcs,
        Wds,
        End,
    }

    pub struct Seg {
        pts: u32,
        dts: u32,
        kind: SegType,
        size: usize,
        payload: Vec<u8>,
    }

    pub trait ReadExt {
        fn read_seg(&mut self) -> SegResult<Seg>;
    }

    impl ReadExt for dyn Read {

        fn read_seg(&mut self) -> SegResult<Seg> {

            if self.read_u16::<BigEndian>()? != 0x5047 {
                return Err(SegError::UnrecognizedMagicNumber)
            }

            let pts = self.read_u32::<BigEndian>()?;
            let dts = self.read_u32::<BigEndian>()?;
            let kind = match self.read_u8()? {
                0x14 => SegType::Pds,
                0x15 => SegType::Ods,
                0x16 => SegType::Pcs,
                0x17 => SegType::Wds,
                0x80 => SegType::End,
                _ => return Err(SegError::UnrecognizedKind),
            };
            let size = self.read_u16::<BigEndian>()? as usize;
            let mut payload = vec![0x0; size];

            self.read_exact(&mut payload)?;

            Ok(Seg { pts, dts, kind, size, payload })
        }
    }
}
