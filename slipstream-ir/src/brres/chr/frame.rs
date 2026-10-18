use bitfield_struct::bitfield;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use slipstream_shared::cursor::MutCursor;
use slipstream_shared::error::InvalidInputError;
use slipstream_shared::{RefCursor, SlipstreamError, SlipstreamResult, verify};

/// A 4-byte animation frame.
#[derive(Debug, Clone, PartialEq)]
pub struct I4Frame {
    /// The index of this frame in the animation.
    pub index: u8,
    pub step: f32,
    /// The tangent line to the interpolation slope of the animation.
    pub tangent: f32
}

#[bitfield(u32)]
struct I4FrameBits {
    #[bits(8)]
    pub index: u8,
    #[bits(12)]
    pub step: u16,
    #[bits(12)]
    pub tangent: u16
}

impl I4Frame {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let word = reader.read_u32::<BigEndian>()?;
        let bitfield = I4FrameBits::from_bits(word);

        let index = bitfield.index();
        let step = bitfield.step() as f32;
        let tangent = bitfield.tangent() as f32 / 32.0;

        Ok(Self {
            index,
            step,
            tangent
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        let bits = I4FrameBits::new()
            .with_index_checked(self.index)
            .map_err(|_| {
                SlipstreamError::from(InvalidInputError {
                    reason: format!("Interpolated4 frame index out of range: {}", self.index),
                    ..Default::default()
                })
            })?
            .with_step_checked(self.step as u16)
            .map_err(|_| {
                SlipstreamError::from(InvalidInputError {
                    reason: format!("Interpolated4 frame step out of range: {}", self.step),
                    ..Default::default()
                })
            })?
            .with_tangent_checked((self.tangent * 32.0) as u16)
            .map_err(|_| {
                SlipstreamError::from(InvalidInputError {
                    reason: format!("Interpolated4 frame tangent out of range: {}", self.tangent),
                    ..Default::default()
                })
            })?;

        writer.write_u32::<BigEndian>(bits.into_bits())?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct I6Frame {
    pub index: f32,
    pub step: f32,
    pub tangent: f32
}

impl I6Frame {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let index = (reader.read_u16::<BigEndian>()? as f32) / 32.0;
        let step = reader.read_u16::<BigEndian>()? as f32;
        let tangent = (reader.read_u16::<BigEndian>()? as f32) / 256.0;

        Ok(Self {
            index,
            step,
            tangent
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_u16::<BigEndian>((self.index * 32.0) as u16)?;
        writer.write_u16::<BigEndian>(self.step as u16)?;
        writer.write_u16::<BigEndian>((self.tangent * 256.0) as u16)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct I12Frame {
    pub index: f32,
    pub value: f32,
    pub tangent: f32
}

impl I12Frame {
    pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
        let index = reader.read_f32::<BigEndian>()?;
        let value = reader.read_f32::<BigEndian>()?;
        let tangent = reader.read_f32::<BigEndian>()?;

        Ok(Self {
            index,
            value,
            tangent
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_f32::<BigEndian>(self.index)?;
        writer.write_f32::<BigEndian>(self.value)?;
        writer.write_f32::<BigEndian>(self.tangent)?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct I4Animation {
    pub frame_scale: f32,
    pub step: f32,
    pub base: f32,
    pub frames: Vec<I4Frame>
}

impl I4Animation {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header_frame_count: u16
    ) -> SlipstreamResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        verify!(
            frame_count <= header_frame_count,
            "animation has more frames than header specifies ({frame_count} vs. {header_frame_count})"
        );

        tracing::trace!(
            "Reading {frame_count} I4 frames at location {}",
            reader.position()
        );

        let _unknown1 = reader.read_u16::<BigEndian>()?;
        let frame_scale = reader.read_f32::<BigEndian>()?;
        let step = reader.read_f32::<BigEndian>()?;
        let base = reader.read_f32::<BigEndian>()?;

        let mut frames = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            frames.push(I4Frame::deserialize(reader)?);
        }

        Ok(Self {
            frame_scale,
            step,
            base,
            frames
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        let frame_count = self.frames.len().try_into().map_err(|_| {
            SlipstreamError::from(InvalidInputError {
                reason: format!(
                    "too many frames in animation ({}), max is 65536",
                    self.frames.len()
                ),
                ..Default::default()
            })
        })?;
        writer.write_u16::<BigEndian>(frame_count)?;

        // Unknown
        writer.write_u16::<BigEndian>(0)?;

        writer.write_f32::<BigEndian>(self.frame_scale)?;
        writer.write_f32::<BigEndian>(self.step)?;
        writer.write_f32::<BigEndian>(self.base)?;

        for frame in &self.frames {
            frame.serialize(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct I6Animation {
    pub frame_scale: f32,
    pub step: f32,
    pub base: f32,
    pub frames: Vec<I6Frame>
}

impl I6Animation {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header_frame_count: u16
    ) -> SlipstreamResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        verify!(
            frame_count <= header_frame_count,
            "animation has more frames than header specifies ({frame_count} vs. {header_frame_count})"
        );

        tracing::trace!(
            "Reading {frame_count} I6 frames at location {}",
            reader.position()
        );

        let _unknown1 = reader.read_u16::<BigEndian>()?;
        let frame_scale = reader.read_f32::<BigEndian>()?;
        let step = reader.read_f32::<BigEndian>()?;
        let base = reader.read_f32::<BigEndian>()?;

        let mut frames = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            frames.push(I6Frame::deserialize(reader)?);
        }

        Ok(Self {
            frame_scale,
            step,
            base,
            frames
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        let frame_count = self.frames.len().try_into().map_err(|_| {
            SlipstreamError::from(InvalidInputError {
                reason: format!(
                    "too many frames in animation ({}), max is 65536",
                    self.frames.len()
                ),
                ..Default::default()
            })
        })?;
        writer.write_u16::<BigEndian>(frame_count)?;

        // Unknown
        writer.write_u16::<BigEndian>(0)?;

        writer.write_f32::<BigEndian>(self.frame_scale)?;
        writer.write_f32::<BigEndian>(self.step)?;
        writer.write_f32::<BigEndian>(self.base)?;

        for frame in &self.frames {
            frame.serialize(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct I12Animation {
    pub frame_scale: f32,
    pub frames: Vec<I12Frame>
}

impl I12Animation {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header_frame_count: u16
    ) -> SlipstreamResult<Self> {
        let frame_count = reader.read_u16::<BigEndian>()?;
        verify!(
            frame_count <= header_frame_count,
            "animation has more frames than header specifies ({frame_count} vs. {header_frame_count})"
        );

        tracing::trace!(
            "Reading {frame_count} I12 frames at location {}",
            reader.position()
        );

        let _unknown1 = reader.read_u16::<BigEndian>()?;
        let frame_scale = reader.read_f32::<BigEndian>()?;

        let mut frames = Vec::with_capacity(frame_count as usize);
        for _ in 0..frame_count {
            frames.push(I12Frame::deserialize(reader)?);
        }

        Ok(Self {
            frame_scale,
            frames
        })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        let frame_count = self.frames.len().try_into().map_err(|_| {
            SlipstreamError::from(InvalidInputError {
                reason: format!(
                    "too many frames in animation ({}), max is 65536",
                    self.frames.len()
                ),
                ..Default::default()
            })
        })?;
        writer.write_u16::<BigEndian>(frame_count)?;

        // Unknown
        writer.write_u16::<BigEndian>(0)?;

        writer.write_f32::<BigEndian>(self.frame_scale)?;
        for frame in &self.frames {
            frame.serialize(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct L1Animation {
    pub step: f32,
    pub base: f32,
    pub frames: Vec<u8>
}

impl L1Animation {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header_frame_count: u16
    ) -> SlipstreamResult<Self> {
        tracing::trace!(
            "Reading {header_frame_count} L1 frames at location {}",
            reader.position()
        );

        // reader.set_position(reader.position() - 4);

        let step = reader.read_f32::<BigEndian>()?;
        let base = reader.read_f32::<BigEndian>()?;

        let mut frames = Vec::with_capacity(header_frame_count as usize);
        for _ in 0..header_frame_count {
            frames.push(reader.read_u8()?);
        }

        Ok(Self { step, base, frames })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        writer.write_f32::<BigEndian>(self.step)?;
        writer.write_f32::<BigEndian>(self.base)?;

        for frame in &self.frames {
            writer.write_u8(*frame)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct L4Animation {
    pub frames: Vec<f32>
}

impl L4Animation {
    pub fn deserialize(
        reader: &mut RefCursor<[u8]>,
        header_frame_count: u16
    ) -> SlipstreamResult<Self> {
        tracing::trace!(
            "Reading {header_frame_count} L4 frames at location {}",
            reader.position()
        );

        let mut frames = Vec::with_capacity(header_frame_count as usize);
        for _ in 0..header_frame_count {
            frames.push(reader.read_f32::<BigEndian>()?);
        }

        Ok(Self { frames })
    }

    pub fn serialize(&self, writer: &mut MutCursor) -> SlipstreamResult<()> {
        for frame in &self.frames {
            writer.write_f32::<BigEndian>(*frame)?;
        }

        Ok(())
    }
}
