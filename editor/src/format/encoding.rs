use std::{io::Cursor, rc::Rc};

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};

use crate::format::error::{CorruptionError, EncodingError, EncodingResult};

macro_rules! impl_byteorder_arrays {
    ($($ty: ty),*) => {
        paste::paste! {
            pub trait WriteArrayExt: byteorder::WriteBytesExt {
                fn write_u8_array<const N: usize>(&mut self, values: [u8; N]) -> std::io::Result<()>;
                fn write_i8_array<const N: usize>(&mut self, values: [i8; N]) -> std::io::Result<()>;

                $(
                    fn [<write_ $ty _array>]<const N: usize, B: byteorder::ByteOrder>(&mut self, values: [$ty; N])
                        -> std::io::Result<()>;
                )*
            }

            impl<W: byteorder::WriteBytesExt> WriteArrayExt for W {
                fn write_u8_array<const N: usize>(&mut self, values: [u8; N]) -> std::io::Result<()> {
                    for b in values {
                        self.write_u8(b)?;
                    }

                    Ok(())
                }

                fn write_i8_array<const N: usize>(&mut self, values: [i8; N]) -> std::io::Result<()> {
                    for b in values {
                        self.write_i8(b)?;
                    }

                    Ok(())
                }

                $(
                    fn [< write_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self, values: [$ty; N])
                        -> std::io::Result<()>
                    {
                        for b in values {
                            self.[<write_ $ty>]::<B>(b)?;
                        }

                        Ok(())
                    }
                )*
            }

            pub trait ReadArrayExt: byteorder::ReadBytesExt {
                fn read_u8_array<const N: usize>(&mut self) -> std::io::Result<[u8; N]>;
                fn read_i8_array<const N: usize>(&mut self) -> std::io::Result<[i8; N]>;

                $(
                    fn [< read_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self)
                        -> std::io::Result<[$ty; N]>;
                )*
            }

            impl<R: byteorder::ReadBytesExt> ReadArrayExt for R {
                fn read_u8_array<const N: usize>(&mut self) -> std::io::Result<[u8; N]> {
                    let mut array = [0; N];
                    for i in 0..N {
                        array[i] = self.read_u8()?;
                    }

                    Ok(array)
                }

                fn read_i8_array<const N: usize>(&mut self) -> std::io::Result<[i8; N]> {
                    let mut array = [0; N];
                    for i in 0..N {
                        array[i] = self.read_i8()?;
                    }

                    Ok(array)
                }

                $(
                    fn [< read_ $ty _array >]<const N: usize, B: byteorder::ByteOrder>(&mut self)
                        -> std::io::Result<[$ty; N]>
                    {
                        let mut array = [Default::default(); N];
                        for i in 0..N {
                            array[i] = self.[< read_ $ty >]::<B>()?;
                        }

                        Ok(array)
                    }
                )*
            }
        }
    }
}

impl_byteorder_arrays!(u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);

pub trait ReadStringExt<'buf>: ReadBytesExt {
    /// Reads a `&str` with a `u32` length prefix.
    fn read_u32_str<B: byteorder::ByteOrder>(&mut self) -> EncodingResult<&'buf str>;
    /// Reads a `&str` with a null terminator.
    fn read_null_str<B: byteorder::ByteOrder>(&mut self) -> EncodingResult<&'buf str>;

    /// Reads a `String` with a `u32` length prefix.
    fn read_u32_string<B: byteorder::ByteOrder>(&mut self) -> EncodingResult<String> {
        let str = self.read_u32_str::<B>()?;
        Ok(str.to_owned())
    }

    /// Reads a `String` with a null terminator.
    fn read_null_string<B: byteorder::ByteOrder>(&mut self) -> EncodingResult<String> {
        let str = self.read_null_str::<B>()?;
        Ok(str.to_owned())
    }
}

impl<'buf> ReadStringExt<'buf> for Cursor<&'buf [u8]> {
    fn read_u32_str<B: byteorder::ByteOrder>(&mut self) -> EncodingResult<&'buf str> {
        let str_len = self.read_u32::<B>()?;
        let str_buffer =
            &self.get_ref()[self.position() as usize..str_len as usize + self.position() as usize];

        Ok(str::from_utf8(str_buffer)?)
    }

    fn read_null_str<B: byteorder::ByteOrder>(&mut self) -> EncodingResult<&'buf str> {
        let start_buffer = &self.get_ref()[self.position() as usize..];
        let null_position = start_buffer
            .iter()
            .position(|&b| b == 0x00)
            .ok_or_else(|| {
                EncodingError::from(CorruptionError {
                    reason: "did not find string null terminator before EOF".to_owned(),
                    ..Default::default()
                })
            })?;

        Ok(str::from_utf8(&start_buffer[..null_position as usize])?)
    }
}

pub trait Deserialize: Sized {
    fn deserialize(reader: &mut Cursor<Rc<[u8]>>) -> EncodingResult<Self>;
}

pub trait Serialize {
    fn serialize(&self) -> EncodingResult<Vec<u8>> {
        let mut out = Vec::new();
        self.serialize_into(&mut out)?;

        Ok(out)
    }

    fn serialize_into(&self, writer: &mut Vec<u8>) -> EncodingResult<()>;
}
