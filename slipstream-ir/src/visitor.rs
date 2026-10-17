use crate::arc::{ArcDirectory, UnknownFile};
use crate::brres::BrresFile;
use crate::chr::{self, Chr0Visitor};
use crate::mdl::{
    self, Bone, ColorBuffer, Definitions, MaterialBuffer, Mdl0Visitor, NormalBuffer, PaletteLinks,
    Polygon, Tev, TextureLinks, UvBuffer, VertexBuffer
};
use crate::node::arena::IrNodeKey;
use crate::node::node::{IrNode, IrNodeType};
use crate::tex::Texture;
use crate::tpl::{TplHeader, TplImage};
use downcast_rs::{Downcast, impl_downcast};
use std::ops::{ControlFlow, Deref, DerefMut};

pub struct VisitorCtxMeta<'a> {
    pub label: &'a str,
    pub ty: IrNodeType,
    pub key: IrNodeKey,
    pub children: &'a [IrNodeKey]
}

impl<'a> From<&'a IrNode> for VisitorCtxMeta<'a> {
    #[inline]
    fn from(node: &'a IrNode) -> Self {
        Self {
            label: &node.label,
            ty: node.ty,
            key: node.key(),
            children: &node.children
        }
    }
}

pub struct VisitorCtx<'a, T> {
    pub meta: VisitorCtxMeta<'a>,
    pub content: &'a T
}

impl<'a, T> VisitorCtx<'a, T> {
    #[inline]
    pub(crate) fn new(node: VisitorCtxMeta<'a>, content: &'a T) -> Self {
        Self {
            meta: node,
            content
        }
    }
}

impl<'a, T> Deref for VisitorCtx<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.content
    }
}

pub struct VisitorCtxMetaMut<'a> {
    pub label: &'a mut String,
    pub ty: IrNodeType,
    pub key: IrNodeKey,
    pub children: &'a mut [IrNodeKey]
}

impl<'a> From<&'a mut IrNode> for VisitorCtxMetaMut<'a> {
    fn from(node: &'a mut IrNode) -> Self {
        Self {
            key: node.key(),
            label: &mut node.label,
            ty: node.ty,
            children: &mut node.children
        }
    }
}

/// The contents can be accessed via the deref implementations.
pub struct VisitorCtxMut<'a, T> {
    pub meta: VisitorCtxMetaMut<'a>,
    pub content: &'a mut T
}

impl<'a, T> VisitorCtxMut<'a, T> {
    #[inline]
    pub(crate) fn new(node: VisitorCtxMetaMut<'a>, content: &'a mut T) -> Self {
        Self {
            meta: node,
            content
        }
    }
}

impl<'a, T: 'static> Deref for VisitorCtxMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.content
    }
}

impl<'a, T: 'static> DerefMut for VisitorCtxMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.content
    }
}

/// Visitor for node-level data. See [`PropertyVisitor`] for a more fine-grained visitor
/// that can operate on struct fields themselves.
///
/// Returning `ControlFlow::Break` from the visitor will end the walk in the current branch.
/// This means that further children of the node will not be visited.
///
/// [`PropertyVisitor`]: slipstream_shared::inspect::PropertyVisitor
#[allow(unused_variables)]
pub trait Visitor {
    /// Whether to stop traversing the tree when an unimplemented visitor method is encountered.
    ///
    /// This is used by ARC files to discover how many nodes they have without having to manually
    /// implementing breaking control flow on every other possible type.
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    /// Turns this visitor into an MDL visitor.
    fn as_mdl(&mut self) -> Option<&mut dyn Mdl0Visitor> {
        None
    }

    /// Turns this visitor into a CHR visitor.
    fn as_chr(&mut self) -> Option<&mut dyn Chr0Visitor> {
        None
    }

    // Trivial files
    // ===================================================================================================

    fn visit_arc(&mut self, arc: VisitorCtx<'_, ArcDirectory>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_arc_mut(&mut self, arc: VisitorCtxMut<'_, ArcDirectory>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_brres(&mut self, brres: VisitorCtx<'_, BrresFile>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_brres_mut(&mut self, brres: VisitorCtxMut<'_, BrresFile>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    /// Immutably visits the header of a TPL file.
    fn visit_tpl(&mut self, tpl: VisitorCtx<'_, TplHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    /// Mutably visits the header of a TPL file.
    fn visit_tpl_mut(&mut self, tpl: VisitorCtxMut<'_, TplHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    /// Immutably visits an image in a TPL file.
    fn visit_tpl_image(&mut self, image: VisitorCtx<'_, TplImage>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    /// Mutably visits an image in a TPL file.
    fn visit_tpl_image_mut(&mut self, image: VisitorCtxMut<'_, TplImage>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_unknown(&mut self, unknown: VisitorCtx<'_, UnknownFile>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_unknown_mut(&mut self, unknown: VisitorCtxMut<'_, UnknownFile>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    // TEX methods
    // ==================================================================================================

    fn visit_tex(&mut self, texture: VisitorCtx<'_, Texture>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_tex_mut(&mut self, texture: VisitorCtxMut<'_, Texture>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

pub trait Visitable: Downcast {
    fn accept(&self, node: VisitorCtxMeta, visitor: &mut dyn Visitor) -> ControlFlow<()>;
    fn accept_mut(&mut self, node: VisitorCtxMetaMut, visitor: &mut dyn Visitor)
    -> ControlFlow<()>;
}
impl_downcast!(Visitable);
