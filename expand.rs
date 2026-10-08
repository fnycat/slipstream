mod bones {
    use std::ops::ControlFlow;
    use bitfield_struct::bitfield;
    use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
    use slipstream_derive::{Inspect, inspect_bitfield};
    use slipstream_shared::{
        cursor::{MutCursor, RefCursor},
        error::{CorruptionError, InvalidInputError, SlipstreamError, SlipstreamResult},
        inspect::{FieldConfig, Inspect},
        try_unwrap,
    };
    use crate::{
        encoding::ReadArrayExt, index::IndexGroup,
        node::{
            arena::{IrArena, IrNodeDescriptor, IrNodeKey},
            node::{ContentSlot, IrNodeType},
        },
        visitor::{Visitable, Visitor},
    };
    use crate::{
        util::Box3,
        visitor::{
            VisitorContext, VisitorContextMut, VisitorContextNode, VisitorContextNodeMut,
        },
    };
    #[repr(transparent)]
    pub struct BoneFlags(u32);
    #[automatically_derived]
    impl ::core::marker::Copy for BoneFlags {}
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for BoneFlags {}
    #[automatically_derived]
    impl ::core::clone::Clone for BoneFlags {
        #[inline]
        fn clone(&self) -> BoneFlags {
            let _: ::core::clone::AssertParamIsClone<u32>;
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for BoneFlags {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for BoneFlags {
        #[inline]
        fn eq(&self, other: &BoneFlags) -> bool {
            self.0 == other.0
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for BoneFlags {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {
            let _: ::core::cmp::AssertParamIsEq<u32>;
        }
    }
    #[allow(unused_comparisons)]
    #[allow(clippy::unnecessary_cast)]
    #[allow(clippy::assign_op_pattern)]
    #[allow(clippy::double_parens)]
    impl BoneFlags {
        /// Creates a new default initialized bitfield.
        pub const fn new() -> Self {
            let mut this = Self((0));
            this = this.with_use_identity(false);
            this = this.with_translation_isotropic(false);
            this = this.with_rotation_isotropic(false);
            this = this.with_scale_isotropic(false);
            this = this.with_scale_uniform(false);
            this = this.with_apply_scale_compensate(false);
            this = this.with_apply_child_scale_compensate(false);
            this = this.with_disable_classic_scale(false);
            this = this.with_is_visible(false);
            this = this.with_is_display_matrix(false);
            this = this.with_is_billboard_child(false);
            let mask = u32::MAX >> (u32::BITS - 21u32);
            this.0 = ((this.0) | (((0 as u32) & mask) << 11usize));
            this
        }
        /// Convert from bits.
        pub const fn from_bits(bits: u32) -> Self {
            Self(bits)
        }
        /// Convert into bits.
        pub const fn into_bits(self) -> u32 {
            self.0
        }
        const USE_IDENTITY_BITS: usize = 1usize;
        const USE_IDENTITY_OFFSET: usize = 0usize;
        /**

Bits: 0..1*/
        pub const fn use_identity(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::USE_IDENTITY_BITS as u32);
            let this = ((self.0) >> Self::USE_IDENTITY_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 0..1*/
        pub const fn with_use_identity_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_use_identity_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 0..1*/
        #[track_caller]
        pub const fn with_use_identity(mut self, value: bool) -> Self {
            self.set_use_identity(value);
            self
        }
        /**

Bits: 0..1*/
        pub const fn set_use_identity(&mut self, value: bool) {
            if let Err(_) = self.set_use_identity_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 0..1*/
        pub const fn set_use_identity_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::USE_IDENTITY_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::USE_IDENTITY_OFFSET)
                | (value & mask) << Self::USE_IDENTITY_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const TRANSLATION_ISOTROPIC_BITS: usize = 1usize;
        const TRANSLATION_ISOTROPIC_OFFSET: usize = 1usize;
        /**

Bits: 1..2*/
        pub const fn translation_isotropic(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::TRANSLATION_ISOTROPIC_BITS as u32);
            let this = ((self.0) >> Self::TRANSLATION_ISOTROPIC_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 1..2*/
        pub const fn with_translation_isotropic_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_translation_isotropic_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 1..2*/
        #[track_caller]
        pub const fn with_translation_isotropic(mut self, value: bool) -> Self {
            self.set_translation_isotropic(value);
            self
        }
        /**

Bits: 1..2*/
        pub const fn set_translation_isotropic(&mut self, value: bool) {
            if let Err(_) = self.set_translation_isotropic_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 1..2*/
        pub const fn set_translation_isotropic_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::TRANSLATION_ISOTROPIC_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::TRANSLATION_ISOTROPIC_OFFSET)
                | (value & mask) << Self::TRANSLATION_ISOTROPIC_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const ROTATION_ISOTROPIC_BITS: usize = 1usize;
        const ROTATION_ISOTROPIC_OFFSET: usize = 2usize;
        /**

Bits: 2..3*/
        pub const fn rotation_isotropic(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::ROTATION_ISOTROPIC_BITS as u32);
            let this = ((self.0) >> Self::ROTATION_ISOTROPIC_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 2..3*/
        pub const fn with_rotation_isotropic_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_rotation_isotropic_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 2..3*/
        #[track_caller]
        pub const fn with_rotation_isotropic(mut self, value: bool) -> Self {
            self.set_rotation_isotropic(value);
            self
        }
        /**

Bits: 2..3*/
        pub const fn set_rotation_isotropic(&mut self, value: bool) {
            if let Err(_) = self.set_rotation_isotropic_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 2..3*/
        pub const fn set_rotation_isotropic_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::ROTATION_ISOTROPIC_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::ROTATION_ISOTROPIC_OFFSET)
                | (value & mask) << Self::ROTATION_ISOTROPIC_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const SCALE_ISOTROPIC_BITS: usize = 1usize;
        const SCALE_ISOTROPIC_OFFSET: usize = 3usize;
        /**

Bits: 3..4*/
        pub const fn scale_isotropic(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::SCALE_ISOTROPIC_BITS as u32);
            let this = ((self.0) >> Self::SCALE_ISOTROPIC_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 3..4*/
        pub const fn with_scale_isotropic_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_scale_isotropic_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 3..4*/
        #[track_caller]
        pub const fn with_scale_isotropic(mut self, value: bool) -> Self {
            self.set_scale_isotropic(value);
            self
        }
        /**

Bits: 3..4*/
        pub const fn set_scale_isotropic(&mut self, value: bool) {
            if let Err(_) = self.set_scale_isotropic_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 3..4*/
        pub const fn set_scale_isotropic_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::SCALE_ISOTROPIC_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::SCALE_ISOTROPIC_OFFSET)
                | (value & mask) << Self::SCALE_ISOTROPIC_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const SCALE_UNIFORM_BITS: usize = 1usize;
        const SCALE_UNIFORM_OFFSET: usize = 4usize;
        /**

Bits: 4..5*/
        pub const fn scale_uniform(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::SCALE_UNIFORM_BITS as u32);
            let this = ((self.0) >> Self::SCALE_UNIFORM_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 4..5*/
        pub const fn with_scale_uniform_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_scale_uniform_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 4..5*/
        #[track_caller]
        pub const fn with_scale_uniform(mut self, value: bool) -> Self {
            self.set_scale_uniform(value);
            self
        }
        /**

Bits: 4..5*/
        pub const fn set_scale_uniform(&mut self, value: bool) {
            if let Err(_) = self.set_scale_uniform_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 4..5*/
        pub const fn set_scale_uniform_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::SCALE_UNIFORM_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::SCALE_UNIFORM_OFFSET)
                | (value & mask) << Self::SCALE_UNIFORM_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const APPLY_SCALE_COMPENSATE_BITS: usize = 1usize;
        const APPLY_SCALE_COMPENSATE_OFFSET: usize = 5usize;
        /**

Bits: 5..6*/
        pub const fn apply_scale_compensate(&self) -> bool {
            let mask = u32::MAX
                >> (u32::BITS - Self::APPLY_SCALE_COMPENSATE_BITS as u32);
            let this = ((self.0) >> Self::APPLY_SCALE_COMPENSATE_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 5..6*/
        pub const fn with_apply_scale_compensate_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_apply_scale_compensate_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 5..6*/
        #[track_caller]
        pub const fn with_apply_scale_compensate(mut self, value: bool) -> Self {
            self.set_apply_scale_compensate(value);
            self
        }
        /**

Bits: 5..6*/
        pub const fn set_apply_scale_compensate(&mut self, value: bool) {
            if let Err(_) = self.set_apply_scale_compensate_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 5..6*/
        pub const fn set_apply_scale_compensate_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX
                >> (u32::BITS - Self::APPLY_SCALE_COMPENSATE_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::APPLY_SCALE_COMPENSATE_OFFSET)
                | (value & mask) << Self::APPLY_SCALE_COMPENSATE_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const APPLY_CHILD_SCALE_COMPENSATE_BITS: usize = 1usize;
        const APPLY_CHILD_SCALE_COMPENSATE_OFFSET: usize = 6usize;
        /**

Bits: 6..7*/
        pub const fn apply_child_scale_compensate(&self) -> bool {
            let mask = u32::MAX
                >> (u32::BITS - Self::APPLY_CHILD_SCALE_COMPENSATE_BITS as u32);
            let this = ((self.0) >> Self::APPLY_CHILD_SCALE_COMPENSATE_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 6..7*/
        pub const fn with_apply_child_scale_compensate_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_apply_child_scale_compensate_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 6..7*/
        #[track_caller]
        pub const fn with_apply_child_scale_compensate(mut self, value: bool) -> Self {
            self.set_apply_child_scale_compensate(value);
            self
        }
        /**

Bits: 6..7*/
        pub const fn set_apply_child_scale_compensate(&mut self, value: bool) {
            if let Err(_) = self.set_apply_child_scale_compensate_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 6..7*/
        pub const fn set_apply_child_scale_compensate_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX
                >> (u32::BITS - Self::APPLY_CHILD_SCALE_COMPENSATE_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::APPLY_CHILD_SCALE_COMPENSATE_OFFSET)
                | (value & mask) << Self::APPLY_CHILD_SCALE_COMPENSATE_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const DISABLE_CLASSIC_SCALE_BITS: usize = 1usize;
        const DISABLE_CLASSIC_SCALE_OFFSET: usize = 7usize;
        /**

Bits: 7..8*/
        pub const fn disable_classic_scale(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::DISABLE_CLASSIC_SCALE_BITS as u32);
            let this = ((self.0) >> Self::DISABLE_CLASSIC_SCALE_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 7..8*/
        pub const fn with_disable_classic_scale_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_disable_classic_scale_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 7..8*/
        #[track_caller]
        pub const fn with_disable_classic_scale(mut self, value: bool) -> Self {
            self.set_disable_classic_scale(value);
            self
        }
        /**

Bits: 7..8*/
        pub const fn set_disable_classic_scale(&mut self, value: bool) {
            if let Err(_) = self.set_disable_classic_scale_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 7..8*/
        pub const fn set_disable_classic_scale_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::DISABLE_CLASSIC_SCALE_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::DISABLE_CLASSIC_SCALE_OFFSET)
                | (value & mask) << Self::DISABLE_CLASSIC_SCALE_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const IS_VISIBLE_BITS: usize = 1usize;
        const IS_VISIBLE_OFFSET: usize = 8usize;
        /**

Bits: 8..9*/
        pub const fn is_visible(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::IS_VISIBLE_BITS as u32);
            let this = ((self.0) >> Self::IS_VISIBLE_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 8..9*/
        pub const fn with_is_visible_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_is_visible_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 8..9*/
        #[track_caller]
        pub const fn with_is_visible(mut self, value: bool) -> Self {
            self.set_is_visible(value);
            self
        }
        /**

Bits: 8..9*/
        pub const fn set_is_visible(&mut self, value: bool) {
            if let Err(_) = self.set_is_visible_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 8..9*/
        pub const fn set_is_visible_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::IS_VISIBLE_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::IS_VISIBLE_OFFSET)
                | (value & mask) << Self::IS_VISIBLE_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const IS_DISPLAY_MATRIX_BITS: usize = 1usize;
        const IS_DISPLAY_MATRIX_OFFSET: usize = 9usize;
        /**

Bits: 9..10*/
        pub const fn is_display_matrix(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::IS_DISPLAY_MATRIX_BITS as u32);
            let this = ((self.0) >> Self::IS_DISPLAY_MATRIX_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 9..10*/
        pub const fn with_is_display_matrix_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_is_display_matrix_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 9..10*/
        #[track_caller]
        pub const fn with_is_display_matrix(mut self, value: bool) -> Self {
            self.set_is_display_matrix(value);
            self
        }
        /**

Bits: 9..10*/
        pub const fn set_is_display_matrix(&mut self, value: bool) {
            if let Err(_) = self.set_is_display_matrix_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 9..10*/
        pub const fn set_is_display_matrix_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::IS_DISPLAY_MATRIX_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::IS_DISPLAY_MATRIX_OFFSET)
                | (value & mask) << Self::IS_DISPLAY_MATRIX_OFFSET;
            self.0 = (bits);
            Ok(())
        }
        const IS_BILLBOARD_CHILD_BITS: usize = 1usize;
        const IS_BILLBOARD_CHILD_OFFSET: usize = 10usize;
        /**

Bits: 10..11*/
        pub const fn is_billboard_child(&self) -> bool {
            let mask = u32::MAX >> (u32::BITS - Self::IS_BILLBOARD_CHILD_BITS as u32);
            let this = ((self.0) >> Self::IS_BILLBOARD_CHILD_OFFSET) & mask;
            this != 0
        }
        /**

Bits: 10..11*/
        pub const fn with_is_billboard_child_checked(
            mut self,
            value: bool,
        ) -> core::result::Result<Self, ()> {
            match self.set_is_billboard_child_checked(value) {
                Ok(_) => Ok(self),
                Err(_) => Err(()),
            }
        }
        /**

Bits: 10..11*/
        #[track_caller]
        pub const fn with_is_billboard_child(mut self, value: bool) -> Self {
            self.set_is_billboard_child(value);
            self
        }
        /**

Bits: 10..11*/
        pub const fn set_is_billboard_child(&mut self, value: bool) {
            if let Err(_) = self.set_is_billboard_child_checked(value) {
                {
                    ::core::panicking::panic_fmt(
                        format_args!("value out of bounds [0, 1]"),
                    );
                }
            }
        }
        /**

Bits: 10..11*/
        pub const fn set_is_billboard_child_checked(
            &mut self,
            value: bool,
        ) -> core::result::Result<(), ()> {
            let this = value;
            let value: u32 = this as _;
            let mask = u32::MAX >> (u32::BITS - Self::IS_BILLBOARD_CHILD_BITS as u32);
            if value > mask {
                return Err(());
            }
            let bits = (self.0) & !(mask << Self::IS_BILLBOARD_CHILD_OFFSET)
                | (value & mask) << Self::IS_BILLBOARD_CHILD_OFFSET;
            self.0 = (bits);
            Ok(())
        }
    }
    #[allow(unused_comparisons)]
    #[allow(clippy::unnecessary_cast)]
    #[allow(clippy::assign_op_pattern)]
    #[allow(clippy::double_parens)]
    impl Default for BoneFlags {
        fn default() -> Self {
            let mut this = Self((0));
            this = this.with_use_identity(false);
            this = this.with_translation_isotropic(false);
            this = this.with_rotation_isotropic(false);
            this = this.with_scale_isotropic(false);
            this = this.with_scale_uniform(false);
            this = this.with_apply_scale_compensate(false);
            this = this.with_apply_child_scale_compensate(false);
            this = this.with_disable_classic_scale(false);
            this = this.with_is_visible(false);
            this = this.with_is_display_matrix(false);
            this = this.with_is_billboard_child(false);
            let mask = u32::MAX >> (u32::BITS - 21u32);
            this.0 = ((this.0) | (((0 as u32) & mask) << 11usize));
            this
        }
    }
    impl From<u32> for BoneFlags {
        fn from(v: u32) -> Self {
            Self(v)
        }
    }
    impl From<BoneFlags> for u32 {
        fn from(v: BoneFlags) -> Self {
            v.0
        }
    }
    impl core::fmt::Debug for BoneFlags {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("BoneFlags")
                .field("use_identity", &self.use_identity())
                .field("translation_isotropic", &self.translation_isotropic())
                .field("rotation_isotropic", &self.rotation_isotropic())
                .field("scale_isotropic", &self.scale_isotropic())
                .field("scale_uniform", &self.scale_uniform())
                .field("apply_scale_compensate", &self.apply_scale_compensate())
                .field(
                    "apply_child_scale_compensate",
                    &self.apply_child_scale_compensate(),
                )
                .field("disable_classic_scale", &self.disable_classic_scale())
                .field("is_visible", &self.is_visible())
                .field("is_display_matrix", &self.is_display_matrix())
                .field("is_billboard_child", &self.is_billboard_child())
                .finish()
        }
    }
    /// Automatically generated by the [`Inspect`] derive macro.
    ///
    /// This function generates an abstract representation of the current struct
    impl slipstream_shared::inspect::Inspect for BoneFlags {
        fn draw_inspect(
            &mut self,
            ui: &mut egui::Ui,
            cfg: &slipstream_shared::inspect::FieldConfig,
        ) -> slipstream_shared::inspect::Changes {
            use slipstream_shared::widgets;
            let mut changes = slipstream_shared::inspect::Changes::default();
            widgets::draw_collapsing_state(
                widgets::CollapseDescriptor::new(
                    ui.id().with("CollapsingState"),
                    "header".into(),
                    None,
                    widgets::HeaderAlignment::Right,
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Use Identity: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "use_identity",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.use_identity(),
                                        on_update: |v| self.set_use_identity(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Translation Isotropic: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "translation_isotropic",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.translation_isotropic(),
                                        on_update: |v| self.set_translation_isotropic(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Rotation Isotropic: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "rotation_isotropic",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.rotation_isotropic(),
                                        on_update: |v| self.set_rotation_isotropic(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Scale Isotropic: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "scale_isotropic",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.scale_isotropic(),
                                        on_update: |v| self.set_scale_isotropic(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Scale Uniform: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "scale_uniform",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.scale_uniform(),
                                        on_update: |v| self.set_scale_uniform(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Apply Scale Compensate: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "apply_scale_compensate",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.apply_scale_compensate(),
                                        on_update: |v| self.set_apply_scale_compensate(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Apply Child Scale Compensate: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "apply_child_scale_compensate",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.apply_child_scale_compensate(),
                                        on_update: |v| self.set_apply_child_scale_compensate(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Disable Classic Scale: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "disable_classic_scale",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.disable_classic_scale(),
                                        on_update: |v| self.set_disable_classic_scale(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Is Visible: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "is_visible",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.is_visible(),
                                        on_update: |v| self.set_is_visible(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Is Display Matrix: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "is_display_matrix",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.is_display_matrix(),
                                        on_update: |v| self.set_is_display_matrix(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Is Billboard Child: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "is_billboard_child",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = slipstream_shared::inspect::BitFieldWrapper {
                                        value: self.is_billboard_child(),
                                        on_update: |v| self.set_is_billboard_child(v),
                                    }
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        Ok(())
                    },
                ),
                ui,
            );
            changes
        }
        fn draw_value(
            &mut self,
            ui: &mut egui::Ui,
            cfg: &slipstream_shared::inspect::FieldConfig,
        ) -> slipstream_shared::inspect::Changes {
            let egui::InnerResponse { inner, .. } = egui::CollapsingHeader::new(
                    "Bone Flags",
                )
                .show(
                    ui,
                    |ui| {
                        let mut changes = slipstream_shared::inspect::Changes::default();
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Use Identity: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "use_identity",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.use_identity(),
                                            on_update: |v| self.set_use_identity(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Translation Isotropic: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "translation_isotropic",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.translation_isotropic(),
                                            on_update: |v| self.set_translation_isotropic(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Rotation Isotropic: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "rotation_isotropic",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.rotation_isotropic(),
                                            on_update: |v| self.set_rotation_isotropic(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Scale Isotropic: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "scale_isotropic",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.scale_isotropic(),
                                            on_update: |v| self.set_scale_isotropic(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Scale Uniform: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "scale_uniform",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.scale_uniform(),
                                            on_update: |v| self.set_scale_uniform(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Apply Scale Compensate: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "apply_scale_compensate",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.apply_scale_compensate(),
                                            on_update: |v| self.set_apply_scale_compensate(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Apply Child Scale Compensate: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "apply_child_scale_compensate",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.apply_child_scale_compensate(),
                                            on_update: |v| self.set_apply_child_scale_compensate(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Disable Classic Scale: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "disable_classic_scale",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.disable_classic_scale(),
                                            on_update: |v| self.set_disable_classic_scale(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Is Visible: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "is_visible",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.is_visible(),
                                            on_update: |v| self.set_is_visible(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Is Display Matrix: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "is_display_matrix",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.is_display_matrix(),
                                            on_update: |v| self.set_is_display_matrix(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Is Billboard Child: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "is_billboard_child",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = slipstream_shared::inspect::BitFieldWrapper {
                                            value: self.is_billboard_child(),
                                            on_update: |v| self.set_is_billboard_child(v),
                                        }
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                        });
                        changes
                    },
                );
            inner
        }
    }
    /// Configures the way billboarding is used for this object.
    ///
    /// This can be used to make something always face the camera.
    pub enum BillboardSetting {
        /// No influence.
        Disabled,
        /// Influenced by rotation of parent node. Z-axis is parallel to camera lens axis.
        Billboard,
        /// Influenced by rotation of parent node. Z-axis points toward camera direction.
        PerspectiveBillboard,
        /// Not influenced by rotation of parent node, restricted by camera's up vector.
        /// Z-axis is parallel to camera lens axis.
        CameraBillboard,
        /// Not influenced by rotation of parent node, restricted by camera's up vector.
        /// Z-axis points toward camera direction.
        CameraPerspectiveBillboard,
        /// Influenced by rotation of parent node and rotates only around Y-axis.
        /// Z-axis is parallel to camera lens axis.
        YBillboard,
        /// Influenced by rotation of parent node and rotates only around Y-axis.
        /// Z-axis points toward camera direction.
        YPerspectiveBillboard,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for BillboardSetting {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::write_str(
                f,
                match self {
                    BillboardSetting::Disabled => "Disabled",
                    BillboardSetting::Billboard => "Billboard",
                    BillboardSetting::PerspectiveBillboard => "PerspectiveBillboard",
                    BillboardSetting::CameraBillboard => "CameraBillboard",
                    BillboardSetting::CameraPerspectiveBillboard => {
                        "CameraPerspectiveBillboard"
                    }
                    BillboardSetting::YBillboard => "YBillboard",
                    BillboardSetting::YPerspectiveBillboard => "YPerspectiveBillboard",
                },
            )
        }
    }
    #[automatically_derived]
    impl ::core::marker::Copy for BillboardSetting {}
    #[automatically_derived]
    #[doc(hidden)]
    unsafe impl ::core::clone::TrivialClone for BillboardSetting {}
    #[automatically_derived]
    impl ::core::clone::Clone for BillboardSetting {
        #[inline]
        fn clone(&self) -> BillboardSetting {
            *self
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for BillboardSetting {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for BillboardSetting {
        #[inline]
        fn eq(&self, other: &BillboardSetting) -> bool {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            __self_discr == __arg1_discr
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Eq for BillboardSetting {
        #[inline]
        #[doc(hidden)]
        #[coverage(off)]
        fn assert_fields_are_eq(&self) {}
    }
    #[automatically_derived]
    impl ::core::cmp::PartialOrd for BillboardSetting {
        #[inline]
        fn partial_cmp(
            &self,
            other: &BillboardSetting,
        ) -> ::core::option::Option<::core::cmp::Ordering> {
            ::core::option::Option::Some(::core::cmp::Ord::cmp(self, other))
        }
    }
    #[automatically_derived]
    impl ::core::cmp::Ord for BillboardSetting {
        #[inline]
        fn cmp(&self, other: &BillboardSetting) -> ::core::cmp::Ordering {
            let __self_discr = ::core::intrinsics::discriminant_value(self);
            let __arg1_discr = ::core::intrinsics::discriminant_value(other);
            ::core::cmp::Ord::cmp(&__self_discr, &__arg1_discr)
        }
    }
    impl slipstream_shared::inspect::AsEnumLabel for BillboardSetting {
        #[inline]
        fn as_index(&self) -> usize {
            match self {
                Self::Disabled => 0usize,
                Self::Billboard => 1usize,
                Self::PerspectiveBillboard => 2usize,
                Self::CameraBillboard => 3usize,
                Self::CameraPerspectiveBillboard => 4usize,
                Self::YBillboard => 5usize,
                Self::YPerspectiveBillboard => 6usize,
            }
        }
        #[inline]
        fn as_label(&self) -> &'static str {
            match self {
                Self::Disabled => "Disabled",
                Self::Billboard => "Billboard",
                Self::PerspectiveBillboard => "Perspective Billboard",
                Self::CameraBillboard => "Camera Billboard",
                Self::CameraPerspectiveBillboard => "Camera Perspective Billboard",
                Self::YBillboard => "Y Billboard",
                Self::YPerspectiveBillboard => "Y Perspective Billboard",
            }
        }
    }
    impl slipstream_shared::inspect::Inspect for BillboardSetting {
        fn draw_inspect(
            &mut self,
            ui: &mut egui::Ui,
            cfg: &slipstream_shared::inspect::FieldConfig,
        ) -> slipstream_shared::inspect::Changes {
            ::core::panicking::panic("not yet implemented");
        }
        fn draw_value(
            &mut self,
            ui: &mut egui::Ui,
            cfg: &slipstream_shared::inspect::FieldConfig,
        ) -> slipstream_shared::inspect::Changes {
            let curr_label = slipstream_shared::inspect::AsEnumLabel::as_label(self);
            let mut changes = slipstream_shared::inspect::Changes::default();
            egui::ComboBox::new(ui.id().with("ComboBox"), "")
                .selected_text(curr_label)
                .show_ui(
                    ui,
                    |ui| {
                        let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(
                            self,
                        ) == 0usize;
                        let response = ui.selectable_label(is_selected, "Disabled");
                        if response.clicked() {
                            *self = Self::Disabled;
                            changes.changed = true;
                        }
                        let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(
                            self,
                        ) == 1usize;
                        let response = ui.selectable_label(is_selected, "Billboard");
                        if response.clicked() {
                            *self = Self::Billboard;
                            changes.changed = true;
                        }
                        let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(
                            self,
                        ) == 2usize;
                        let response = ui
                            .selectable_label(is_selected, "Perspective Billboard");
                        if response.clicked() {
                            *self = Self::PerspectiveBillboard;
                            changes.changed = true;
                        }
                        let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(
                            self,
                        ) == 3usize;
                        let response = ui
                            .selectable_label(is_selected, "Camera Billboard");
                        if response.clicked() {
                            *self = Self::CameraBillboard;
                            changes.changed = true;
                        }
                        let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(
                            self,
                        ) == 4usize;
                        let response = ui
                            .selectable_label(
                                is_selected,
                                "Camera Perspective Billboard",
                            );
                        if response.clicked() {
                            *self = Self::CameraPerspectiveBillboard;
                            changes.changed = true;
                        }
                        let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(
                            self,
                        ) == 5usize;
                        let response = ui.selectable_label(is_selected, "Y Billboard");
                        if response.clicked() {
                            *self = Self::YBillboard;
                            changes.changed = true;
                        }
                        let is_selected = slipstream_shared::inspect::AsEnumLabel::as_index(
                            self,
                        ) == 6usize;
                        let response = ui
                            .selectable_label(is_selected, "Y Perspective Billboard");
                        if response.clicked() {
                            *self = Self::YPerspectiveBillboard;
                            changes.changed = true;
                        }
                    },
                );
            changes
        }
    }
    impl BillboardSetting {
        /// Returns the amount of items in this enum.
        pub fn len() -> usize {
            Self::YPerspectiveBillboard as usize + 1
        }
    }
    impl TryFrom<u32> for BillboardSetting {
        type Error = SlipstreamError;
        fn try_from(value: u32) -> Result<Self, Self::Error> {
            Ok(
                match value {
                    0 => Self::Disabled,
                    1 => Self::Billboard,
                    2 => Self::PerspectiveBillboard,
                    3 => Self::CameraBillboard,
                    4 => Self::CameraPerspectiveBillboard,
                    5 => Self::YBillboard,
                    6 => Self::YPerspectiveBillboard,
                    v => {
                        return Err(
                            CorruptionError {
                                reason: ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "invalid bone flag billboard setting: {0} (expected 0-6)",
                                            v,
                                        ),
                                    )
                                }),
                                ..Default::default()
                            }
                                .into(),
                        );
                    }
                },
            )
        }
    }
    impl BillboardSetting {
        fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
            let word = reader.read_u32::<BigEndian>()?;
            Self::try_from(word)
        }
        fn serialize(self, writer: &mut MutCursor) -> SlipstreamResult<()> {
            writer.write_u32::<BigEndian>(self as u32)?;
            Ok(())
        }
    }
    /// A bone that already has all its data deserialized but without resolved references.
    ///
    /// When constructing the skeleton, a new [`Bone`] is created that contains proper references
    /// to other bones.
    pub struct UnresolvedBone {
        pub bone_start: u32,
        pub index: u32,
        pub id: u32,
        pub flags: BoneFlags,
        /// Configures how billboarding is used for this bone.
        pub billboard_setting: BillboardSetting,
        pub billboard_transform: u32,
        pub scaling_vector: glam::Vec3,
        pub rotation_vector: glam::Vec3,
        pub translation_vector: glam::Vec3,
        pub bounding_volume: Box3,
        /// The offset in bytes to the parent of this bone.
        pub parent_offset: i32,
        pub first_child_offset: i32,
        pub next_sibling_offset: i32,
        pub previous_sibling_offset: i32,
        pub user_data_offset: i32,
        pub transform_matrix: glam::Mat4,
        pub inverse_matrix: glam::Mat4,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for UnresolvedBone {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "bone_start",
                "index",
                "id",
                "flags",
                "billboard_setting",
                "billboard_transform",
                "scaling_vector",
                "rotation_vector",
                "translation_vector",
                "bounding_volume",
                "parent_offset",
                "first_child_offset",
                "next_sibling_offset",
                "previous_sibling_offset",
                "user_data_offset",
                "transform_matrix",
                "inverse_matrix",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.bone_start,
                &self.index,
                &self.id,
                &self.flags,
                &self.billboard_setting,
                &self.billboard_transform,
                &self.scaling_vector,
                &self.rotation_vector,
                &self.translation_vector,
                &self.bounding_volume,
                &self.parent_offset,
                &self.first_child_offset,
                &self.next_sibling_offset,
                &self.previous_sibling_offset,
                &self.user_data_offset,
                &self.transform_matrix,
                &&self.inverse_matrix,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(
                f,
                "UnresolvedBone",
                names,
                values,
            )
        }
    }
    #[automatically_derived]
    impl ::core::clone::Clone for UnresolvedBone {
        #[inline]
        fn clone(&self) -> UnresolvedBone {
            UnresolvedBone {
                bone_start: ::core::clone::Clone::clone(&self.bone_start),
                index: ::core::clone::Clone::clone(&self.index),
                id: ::core::clone::Clone::clone(&self.id),
                flags: ::core::clone::Clone::clone(&self.flags),
                billboard_setting: ::core::clone::Clone::clone(&self.billboard_setting),
                billboard_transform: ::core::clone::Clone::clone(
                    &self.billboard_transform,
                ),
                scaling_vector: ::core::clone::Clone::clone(&self.scaling_vector),
                rotation_vector: ::core::clone::Clone::clone(&self.rotation_vector),
                translation_vector: ::core::clone::Clone::clone(
                    &self.translation_vector,
                ),
                bounding_volume: ::core::clone::Clone::clone(&self.bounding_volume),
                parent_offset: ::core::clone::Clone::clone(&self.parent_offset),
                first_child_offset: ::core::clone::Clone::clone(
                    &self.first_child_offset,
                ),
                next_sibling_offset: ::core::clone::Clone::clone(
                    &self.next_sibling_offset,
                ),
                previous_sibling_offset: ::core::clone::Clone::clone(
                    &self.previous_sibling_offset,
                ),
                user_data_offset: ::core::clone::Clone::clone(&self.user_data_offset),
                transform_matrix: ::core::clone::Clone::clone(&self.transform_matrix),
                inverse_matrix: ::core::clone::Clone::clone(&self.inverse_matrix),
            }
        }
    }
    #[automatically_derived]
    impl ::core::marker::StructuralPartialEq for UnresolvedBone {}
    #[automatically_derived]
    impl ::core::cmp::PartialEq for UnresolvedBone {
        #[inline]
        fn eq(&self, other: &UnresolvedBone) -> bool {
            self.bone_start == other.bone_start && self.index == other.index
                && self.id == other.id
                && self.billboard_transform == other.billboard_transform
                && self.parent_offset == other.parent_offset
                && self.first_child_offset == other.first_child_offset
                && self.next_sibling_offset == other.next_sibling_offset
                && self.previous_sibling_offset == other.previous_sibling_offset
                && self.user_data_offset == other.user_data_offset
                && self.flags == other.flags
                && self.billboard_setting == other.billboard_setting
                && self.scaling_vector == other.scaling_vector
                && self.rotation_vector == other.rotation_vector
                && self.translation_vector == other.translation_vector
                && self.bounding_volume == other.bounding_volume
                && self.transform_matrix == other.transform_matrix
                && self.inverse_matrix == other.inverse_matrix
        }
    }
    impl UnresolvedBone {
        pub fn deserialize(reader: &mut RefCursor<[u8]>) -> SlipstreamResult<Self> {
            let start = reader.position();
            let _length = reader.read_u32::<BigEndian>()?;
            let _mdl0_offset = reader.read_i32::<BigEndian>()?;
            let _name_offset = reader.read_i32::<BigEndian>()?;
            let index = reader.read_u32::<BigEndian>()?;
            let id = reader.read_u32::<BigEndian>()?;
            let flags = BoneFlags::from_bits(reader.read_u32::<BigEndian>()?);
            let billboard_setting = BillboardSetting::deserialize(reader)?;
            let billboard_transform = reader.read_u32::<BigEndian>()?;
            let scaling_vector = glam::Vec3::from_array(
                reader.read_f32_array::<3, BigEndian>()?,
            );
            let rotation_vector = glam::Vec3::from_array(
                reader.read_f32_array::<3, BigEndian>()?,
            );
            let translation_vector = glam::Vec3::from_array(
                reader.read_f32_array::<3, BigEndian>()?,
            );
            let bounding_volume = Box3::deserialize(reader)?;
            let parent_offset = reader.read_i32::<BigEndian>()?;
            let first_child_offset = reader.read_i32::<BigEndian>()?;
            let next_sibling_offset = reader.read_i32::<BigEndian>()?;
            let previous_sibling_offset = reader.read_i32::<BigEndian>()?;
            let user_data_offset = reader.read_i32::<BigEndian>()?;
            let m = reader.read_f32_array::<12, BigEndian>()?;
            let transform_matrix = glam::mat4(
                glam::vec4(m[0], m[4], m[8], 0.0),
                glam::vec4(m[1], m[5], m[9], 0.0),
                glam::vec4(m[2], m[6], m[10], 0.0),
                glam::vec4(m[3], m[7], m[11], 1.0),
            );
            let m = reader.read_f32_array::<12, BigEndian>()?;
            let inverse_matrix = glam::mat4(
                glam::vec4(m[0], m[4], m[8], 0.0),
                glam::vec4(m[1], m[5], m[9], 0.0),
                glam::vec4(m[2], m[6], m[10], 0.0),
                glam::vec4(m[3], m[7], m[11], 1.0),
            );
            Ok(Self {
                bone_start: start as u32,
                index,
                id,
                flags,
                billboard_setting,
                billboard_transform,
                scaling_vector,
                rotation_vector,
                translation_vector,
                bounding_volume,
                parent_offset,
                first_child_offset,
                next_sibling_offset,
                previous_sibling_offset,
                user_data_offset,
                transform_matrix,
                inverse_matrix,
            })
        }
    }
    /// A bone combined with its name.
    pub struct LabeledBone {
        pub label: String,
        pub data: UnresolvedBone,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for LabeledBone {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_struct_field2_finish(
                f,
                "LabeledBone",
                "label",
                &self.label,
                "data",
                &&self.data,
            )
        }
    }
    pub struct Bone {
        /// Regular MDL0 index of this section. As bones are stored as a linear array of "files" within the MDL0 file,
        /// these indices correspond to the index of this bone into this array.
        pub index: u32,
        /// The ID stored inside the bone.
        pub id: u32,
        pub flags: BoneFlags,
        pub billboard_setting: BillboardSetting,
        pub billboard_reference: Option<IrNodeKey>,
        pub translation: glam::Vec3,
        #[inspect(suffix = " °", rename = "ROTATION")]
        pub rotation: glam::Vec3,
        pub scale: glam::Vec3,
        pub bounding_volume: Box3,
        #[inspect(ignore)]
        pub parent: Option<IrNodeKey>,
        #[inspect(ignore)]
        pub user_data_offset: i32,
        #[inspect(ignore)]
        pub transform_matrix: glam::Mat4,
        #[inspect(ignore)]
        pub inverse_matrix: glam::Mat4,
    }
    #[automatically_derived]
    impl ::core::fmt::Debug for Bone {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            let names: &'static _ = &[
                "index",
                "id",
                "flags",
                "billboard_setting",
                "billboard_reference",
                "translation",
                "rotation",
                "scale",
                "bounding_volume",
                "parent",
                "user_data_offset",
                "transform_matrix",
                "inverse_matrix",
            ];
            let values: &[&dyn ::core::fmt::Debug] = &[
                &self.index,
                &self.id,
                &self.flags,
                &self.billboard_setting,
                &self.billboard_reference,
                &self.translation,
                &self.rotation,
                &self.scale,
                &self.bounding_volume,
                &self.parent,
                &self.user_data_offset,
                &self.transform_matrix,
                &&self.inverse_matrix,
            ];
            ::core::fmt::Formatter::debug_struct_fields_finish(f, "Bone", names, values)
        }
    }
    /// Automatically generated by the [`Inspect`] derive macro.
    ///
    /// This function generates an abstract representation of the current struct
    impl slipstream_shared::inspect::Inspect for Bone {
        fn draw_inspect(
            &mut self,
            ui: &mut egui::Ui,
            cfg: &slipstream_shared::inspect::FieldConfig,
        ) -> slipstream_shared::inspect::Changes {
            use slipstream_shared::widgets;
            let mut changes = slipstream_shared::inspect::Changes::default();
            widgets::draw_collapsing_state(
                widgets::CollapseDescriptor::new(
                    ui.id().with("CollapsingState"),
                    "header".into(),
                    None,
                    widgets::HeaderAlignment::Right,
                    |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Index: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "index",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self.index.draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Id: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "id",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self.id.draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Flags: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "flags",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self.flags.draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Billboard Setting: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "billboard_setting",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self
                                        .billboard_setting
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Billboard Reference: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "billboard_reference",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self
                                        .billboard_reference
                                        .draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Translation: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "translation",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self.translation.draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Rotation: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "ROTATION",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: Some(" °"),
                                    };
                                    let response = self.rotation.draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Scale: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "scale",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self.scale.draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        ui.horizontal(|ui| {
                            ui.label("Bounding Volume: ");
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Min),
                                |ui| {
                                    const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                        label: "bounding_volume",
                                        category: None,
                                        read_only: false,
                                        range: None,
                                        suffix: None,
                                    };
                                    let response = self.bounding_volume.draw_value(ui, &CONFIG);
                                    changes |= response;
                                },
                            );
                        });
                        ui.end_row();
                        Ok(())
                    },
                ),
                ui,
            );
            changes
        }
        fn draw_value(
            &mut self,
            ui: &mut egui::Ui,
            cfg: &slipstream_shared::inspect::FieldConfig,
        ) -> slipstream_shared::inspect::Changes {
            let egui::InnerResponse { inner, .. } = egui::CollapsingHeader::new("Bone")
                .show(
                    ui,
                    |ui| {
                        let mut changes = slipstream_shared::inspect::Changes::default();
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("Index: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "index",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self.index.draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Id: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "id",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self.id.draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Flags: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "flags",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self.flags.draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Billboard Setting: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "billboard_setting",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self
                                            .billboard_setting
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Billboard Reference: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "billboard_reference",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self
                                            .billboard_reference
                                            .draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Translation: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "translation",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self.translation.draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Rotation: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "ROTATION",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: Some(" °"),
                                        };
                                        let response = self.rotation.draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Scale: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "scale",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self.scale.draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.label("Bounding Volume: ");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Min),
                                    |ui| {
                                        const CONFIG: slipstream_shared::inspect::FieldConfig = slipstream_shared::inspect::FieldConfig {
                                            label: "bounding_volume",
                                            category: None,
                                            read_only: false,
                                            range: None,
                                            suffix: None,
                                        };
                                        let response = self.bounding_volume.draw_value(ui, &CONFIG);
                                        changes |= response;
                                    },
                                );
                            });
                            ui.end_row();
                        });
                        changes
                    },
                );
            inner
        }
    }
    impl Bone {
        /// Converts raw bone data into a more usable format by resolving the file offsets
        /// to proper node references. This ensures the editor knows which other files this one refers to.
        pub fn from_unresolved(
            bone: &UnresolvedBone,
            billboard_id: Option<IrNodeKey>,
            parent_id: Option<IrNodeKey>,
        ) -> Self {
            Self {
                index: bone.index,
                id: bone.id,
                flags: bone.flags.clone(),
                billboard_setting: bone.billboard_setting,
                billboard_reference: billboard_id,
                scale: bone.scaling_vector,
                rotation: bone.rotation_vector,
                translation: bone.translation_vector,
                bounding_volume: bone.bounding_volume,
                parent: parent_id,
                user_data_offset: bone.user_data_offset,
                transform_matrix: bone.transform_matrix,
                inverse_matrix: bone.inverse_matrix,
            }
        }
    }
    impl Visitable for Bone {
        fn accept(
            &self,
            node: VisitorContextNode<'_>,
            visitor: &mut dyn Visitor,
        ) -> ControlFlow<()> {
            visitor.visit_bone(VisitorContext::new(node, self))
        }
        fn accept_mut(
            &mut self,
            node: VisitorContextNodeMut<'_>,
            visitor: &mut dyn Visitor,
        ) -> ControlFlow<()> {
            visitor.visit_bone_mut(VisitorContextMut::new(node, self))
        }
    }
    /// Builds a nested tree of bones as nodes and returns the root node of the skeleton.
    fn build_skeleton_tree(
        reader: &mut RefCursor<[u8]>,
        _root_key: IrNodeKey,
        bones: Vec<LabeledBone>,
        arena: &IrArena,
    ) -> SlipstreamResult<IrNodeKey> {
        /// The offset between the start of the bone and the bone's index.
        const BONE_INDEX_OFFSET: u64 = 3 * 4;
        let virtual_bones = bones
            .iter()
            .map(|bone| {
                arena
                    .insert(IrNodeDescriptor {
                        label: bone.label.clone(),
                        ty: IrNodeType::Bone { end: true },
                        parent: None,
                        ..Default::default()
                    })
            })
            .collect::<Vec<_>>();
        let mut found_root = None;
        for (i, bone) in bones.iter().enumerate() {
            let curr_key = virtual_bones[i];
            if bone.data.parent_offset == 0 {
                found_root = Some(i);
                arena
                    .update(
                        curr_key,
                        |bone_node| {
                            bone_node.ty = IrNodeType::Bone { end: false };
                            bone_node.contents = ContentSlot::eager(
                                Box::new(Bone::from_unresolved(&bone.data, None, None)),
                            );
                        },
                    );
                continue;
            }
            let parent_start = bone.data.bone_start as i64
                + bone.data.parent_offset as i64;
            reader.set_position(parent_start as u64 + BONE_INDEX_OFFSET);
            let parent_index = reader.read_u32::<BigEndian>()?;
            if parent_index == i as u32 {
                return Err(
                    InvalidInputError {
                        reason: ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("bone `{0}` is its own parent", bone.label),
                            )
                        }),
                        ..Default::default()
                    }
                        .into(),
                );
            }
            let parent_key = virtual_bones[parent_index as usize];
            arena
                .update(
                    parent_key,
                    |parent_node| {
                        parent_node.ty = IrNodeType::Bone { end: false };
                        parent_node.children.push(curr_key);
                        parent_node.children.len() as u16 - 1
                    },
                )
                .ok_or_else(|| ::slipstream_shared::error::SlipstreamError::from(::slipstream_shared::error::AssertFailed {
                    reason: ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!(
                                "parent node {0:?} was not found during skeleton resolution",
                                parent_key,
                            ),
                        )
                    }),
                    location: None,
                }))?;
            arena
                .update(
                    curr_key,
                    |curr_node| {
                        curr_node.parent = Some(parent_key);
                        curr_node.contents = ContentSlot::eager(
                            Box::new(
                                Bone::from_unresolved(&bone.data, None, Some(parent_key)),
                            ),
                        );
                    },
                );
        }
        let Some(root_index) = found_root else {
            return Err(
                InvalidInputError {
                    reason: String::from("skeleton contained no root bone"),
                    ..Default::default()
                }
                    .into(),
            );
        };
        {
            use ::tracing::__macro_support::Callsite as _;
            static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                static META: ::tracing::Metadata<'static> = {
                    ::tracing_core::metadata::Metadata::new(
                        "event slipstream-ir\\src\\mdl0\\bones.rs:362",
                        "slipstream_ir::mdl0::bones",
                        ::tracing::Level::TRACE,
                        ::tracing_core::__macro_support::Option::Some(
                            "slipstream-ir\\src\\mdl0\\bones.rs",
                        ),
                        ::tracing_core::__macro_support::Option::Some(362u32),
                        ::tracing_core::__macro_support::Option::Some(
                            "slipstream_ir::mdl0::bones",
                        ),
                        ::tracing_core::field::FieldSet::new(
                            &["message"],
                            ::tracing_core::callsite::Identifier(&__CALLSITE),
                        ),
                        ::tracing::metadata::Kind::EVENT,
                    )
                };
                ::tracing::callsite::DefaultCallsite::new(&META)
            };
            let enabled = ::tracing::Level::TRACE
                <= ::tracing::level_filters::STATIC_MAX_LEVEL
                && ::tracing::Level::TRACE
                    <= ::tracing::level_filters::LevelFilter::current()
                && {
                    let interest = __CALLSITE.interest();
                    !interest.is_never()
                        && ::tracing::__macro_support::__is_enabled(
                            __CALLSITE.metadata(),
                            interest,
                        )
                };
            if enabled {
                (|value_set: ::tracing::field::ValueSet| {
                    let meta = __CALLSITE.metadata();
                    ::tracing::Event::dispatch(meta, &value_set);
                })({
                    #[allow(unused_imports)]
                    use ::tracing::field::{debug, display, Value};
                    __CALLSITE
                        .metadata()
                        .fields()
                        .value_set_all(
                            &[
                                (::tracing::__macro_support::Option::Some(
                                    &format_args!("Skeleton constructed")
                                        as &dyn ::tracing::field::Value,
                                )),
                            ],
                        )
                });
            } else {
            }
        };
        let root = virtual_bones[root_index];
        Ok(root)
    }
    /// Deserializes the entire `Bones` section of an MDL0 file.
    ///
    /// The parser automatically builds a proper file tree of bones that the outliner
    /// can display. References between bones are resolved to use node IDs instead.
    pub fn deserialize_skeleton(
        reader: &mut RefCursor<[u8]>,
        parent_id: IrNodeKey,
        arena: &IrArena,
    ) -> SlipstreamResult<IrNodeKey> {
        {}
        #[allow(clippy::suspicious_else_formatting)]
        {
            let __tracing_attr_span;
            let __tracing_attr_guard;
            if ::tracing::Level::INFO <= ::tracing::level_filters::STATIC_MAX_LEVEL
                && ::tracing::Level::INFO
                    <= ::tracing::level_filters::LevelFilter::current() || { false }
            {
                __tracing_attr_span = {
                    use ::tracing::__macro_support::Callsite as _;
                    static __CALLSITE: ::tracing::callsite::DefaultCallsite = {
                        static META: ::tracing::Metadata<'static> = {
                            ::tracing_core::metadata::Metadata::new(
                                "deserialize_skeleton",
                                "slipstream_ir::mdl0::bones",
                                ::tracing::Level::INFO,
                                ::tracing_core::__macro_support::Option::Some(
                                    "slipstream-ir\\src\\mdl0\\bones.rs",
                                ),
                                ::tracing_core::__macro_support::Option::Some(373u32),
                                ::tracing_core::__macro_support::Option::Some(
                                    "slipstream_ir::mdl0::bones",
                                ),
                                ::tracing_core::field::FieldSet::new(
                                    &[
                                        {
                                            const NAME: ::tracing::__macro_support::FieldName<
                                                { ::tracing::__macro_support::FieldName::len("parent_id") },
                                            > = ::tracing::__macro_support::FieldName::new("parent_id");
                                            NAME.as_str()
                                        },
                                    ],
                                    ::tracing_core::callsite::Identifier(&__CALLSITE),
                                ),
                                ::tracing::metadata::Kind::SPAN,
                            )
                        };
                        ::tracing::callsite::DefaultCallsite::new(&META)
                    };
                    let mut interest = ::tracing::subscriber::Interest::never();
                    if ::tracing::Level::INFO
                        <= ::tracing::level_filters::STATIC_MAX_LEVEL
                        && ::tracing::Level::INFO
                            <= ::tracing::level_filters::LevelFilter::current()
                        && {
                            interest = __CALLSITE.interest();
                            !interest.is_never()
                        }
                        && ::tracing::__macro_support::__is_enabled(
                            __CALLSITE.metadata(),
                            interest,
                        )
                    {
                        let meta = __CALLSITE.metadata();
                        ::tracing::Span::new(
                            meta,
                            &{
                                #[allow(unused_imports)]
                                use ::tracing::field::{debug, display, Value};
                                meta.fields()
                                    .value_set_all(
                                        &[
                                            (::tracing::__macro_support::Option::Some(
                                                &::tracing::field::Empty as &dyn ::tracing::field::Value,
                                            )),
                                        ],
                                    )
                            },
                        )
                    } else {
                        let span = ::tracing::__macro_support::__disabled_span(
                            __CALLSITE.metadata(),
                        );
                        {};
                        span
                    }
                };
                __tracing_attr_guard = __tracing_attr_span.enter();
            }
            #[warn(clippy::suspicious_else_formatting)]
            {
                #[allow(
                    unknown_lints,
                    unreachable_code,
                    clippy::diverging_sub_expression,
                    clippy::empty_loop,
                    clippy::let_unit_value,
                    clippy::let_with_type_underscore,
                    clippy::needless_return,
                    clippy::unreachable
                )]
                if false {
                    let __tracing_attr_fake_return: SlipstreamResult<IrNodeKey> = loop {};
                    return __tracing_attr_fake_return;
                }
                {
                    let section_index = IndexGroup::deserialize(reader)?;
                    let mut bones = Vec::with_capacity(section_index.entries.len() - 1);
                    for entry in &section_index.entries[1..] {
                        let label = section_index.get_entry_name(reader, entry)?;
                        let data_start = section_index.get_entry_data_start(entry);
                        reader.set_position(data_start);
                        let bone = UnresolvedBone::deserialize(reader)?;
                        bones.push(LabeledBone { label, data: bone });
                    }
                    let node = build_skeleton_tree(reader, parent_id, bones, arena)?;
                    let root_key = arena
                        .insert(IrNodeDescriptor {
                            label: "Bones".to_owned(),
                            ty: IrNodeType::Bone { end: false },
                            parent: Some(parent_id),
                            children: ::alloc::boxed::box_assume_init_into_vec_unsafe(
                                ::alloc::intrinsics::write_box_via_move(
                                    ::alloc::boxed::Box::new_uninit(),
                                    [node],
                                ),
                            ),
                            ..Default::default()
                        });
                    Ok(root_key)
                }
            }
        }
    }
}
