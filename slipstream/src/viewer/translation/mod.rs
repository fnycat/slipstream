mod skeleton;
mod vertex;

pub use skeleton::*;
use slipstream_ir::mdl0::{
    Bone, ColorBuffer, DRAW_OPA_NAME, Definitions, MatrixId, Mdl0Header, Mdl0Root, NODE_MIX_NAME,
    NODE_TREE_NAME, NormalBuffer, Polygon, UvBuffer, VertexBuffer,
};
use slipstream_ir::util::Box3;
pub use vertex::*;

use slipstream_ir::node::arena::{IrArena, IrNodeKey};
use slipstream_ir::visitor::{Visitable, Visitor, VisitorContext};
use slipstream_shared::{SlipstreamResult, try_unwrap};
use std::ops::ControlFlow;

#[derive(Default, Debug)]
pub struct IntermediateModel {
    pub bone_map: BoneMap,
    pub bone_weights: Option<BoneWeights>,

    pub xf_slots: XfRegisters,

    pub bounding_volume: Box3,

    pub bone_translations: Vec<glam::Mat4>,
    pub polygons: Vec<IntermediatePolygon>,
}

#[derive(Clone)]
pub struct ModelContents<'a> {
    arena: &'a IrArena,

    bounding_volume: Box3,

    node_tree: Option<IrNodeKey>,
    node_mix: Option<IrNodeKey>,
    draw_xlu: Option<IrNodeKey>,
    draw_opaque: Option<IrNodeKey>,

    skeleton_root: Option<IrNodeKey>,

    vertices: Vec<IrNodeKey>,
    normals: Vec<IrNodeKey>,
    colors: Vec<IrNodeKey>,
    uvs: Vec<IrNodeKey>,
    polygons: Vec<IrNodeKey>,
}

impl<'a> ModelContents<'a> {
    /// Retrieves the given key from the map, downcasts it to `U`
    /// and runs `inspect_fn` on it.
    pub(super) fn try_inspect_inner<F, T, U>(
        &self,
        key: IrNodeKey,
        inspect_fn: F,
    ) -> SlipstreamResult<T>
    where
        U: Visitable,
        F: FnOnce(&U) -> SlipstreamResult<T>,
    {
        let out = self
            .arena
            .inspect(key, |node| {
                let buf = try_unwrap!(node.contents.get_or_try_init()?, "buffer had no content")?;

                let buf = try_unwrap!(
                    buf.as_any().downcast_ref::<U>(),
                    "buffer had an incorrect `Visitable` type"
                )?;

                inspect_fn(buf)
            })
            .transpose()?;

        try_unwrap!(out, "buffer {key:?} did not exist")
    }

    /// Populates the `bone -> matrix` map.
    fn translate_node_tree(
        &self,
        out: &mut IntermediateModel,
        arena: &IrArena,
    ) -> SlipstreamResult<()> {
        struct DefinitionVisitor {
            result: SlipstreamResult<BoneMap>,
        }

        impl Visitor for DefinitionVisitor {
            fn visit_definitions(
                &mut self,
                context: VisitorContext<'_, Definitions>,
            ) -> ControlFlow<()> {
                tracing::trace!("Translating definition `{}`", context.meta.label);
                self.result = BoneMap::from_definitions(context.content);

                ControlFlow::Continue(())
            }
        }

        if let Some(node_tree) = self.node_tree {
            let mut visitor = DefinitionVisitor {
                result: Ok(BoneMap::default()),
            };
            arena.visit(node_tree, &mut visitor)?;
            out.bone_map = visitor.result?;
        }

        Ok(())
    }

    /// Loads all of the bone weights.
    fn translate_node_mix(
        &self,
        out: &mut IntermediateModel,
        arena: &IrArena,
    ) -> SlipstreamResult<()> {
        struct DefinitionVisitor<'a> {
            model: &'a ModelContents<'a>,
            result: SlipstreamResult<BoneWeights>,
        }

        impl Visitor for DefinitionVisitor<'_> {
            fn visit_definitions(
                &mut self,
                context: VisitorContext<'_, Definitions>,
            ) -> ControlFlow<()> {
                tracing::trace!("Translating definition `{}`", context.meta.label);
                self.result = BoneWeights::from_definitions(context.content);

                ControlFlow::Continue(())
            }
        }

        if let Some(node_mix) = self.node_mix {
            let mut visitor = DefinitionVisitor {
                model: self,
                result: Ok(BoneWeights::default()),
            };
            arena.visit(node_mix, &mut visitor)?;
            out.bone_weights = Some(visitor.result?);
        }

        Ok(())
    }

    fn translate_polygons(
        &self,
        out: &mut IntermediateModel,
        arena: &IrArena,
    ) -> SlipstreamResult<()> {
        struct PolygonVisitor<'a> {
            out: &'a mut IntermediateModel,
            input: &'a ModelContents<'a>,
            scratch: &'a mut IntermediatePolygon,
            result: SlipstreamResult<()>,
        }

        impl Visitor for PolygonVisitor<'_> {
            fn visit_polygon(&mut self, context: VisitorContext<'_, Polygon>) -> ControlFlow<()> {
                tracing::trace!("Translating polygon `{}`", context.meta.label);
                self.result = self
                    .input
                    .translate_polygon(self.out, self.scratch, context.content);

                ControlFlow::Break(())
            }
        }

        out.polygons.reserve(self.polygons.len());
        for &polygon in &self.polygons {
            let mut intermediate = IntermediatePolygon::default();
            let mut visitor = PolygonVisitor {
                out,
                input: self,
                scratch: &mut intermediate,
                result: Ok(()),
            };
            let _ = arena.visit(polygon, &mut visitor)?;
            visitor.result?;

            out.polygons.push(intermediate);
        }

        Ok(())
    }

    /// Converts the raw buffers to an [`IntermediateModel`].
    ///
    /// This intermediate model can then be converted into wgpu buffers and commands
    /// in the next step.
    pub fn to_intermediate(&self, arena: &IrArena) -> SlipstreamResult<IntermediateModel> {
        let mut model = IntermediateModel::default();

        model.bounding_volume = self.bounding_volume;
        self.translate_node_tree(&mut model, arena)?;
        self.translate_node_mix(&mut model, arena)?;
        self.populate_primary_influences(&mut model, arena)?;
        self.translate_polygons(&mut model, arena)?;

        Ok(model)
    }

    pub fn from_root(root: IrNodeKey, arena: &'a IrArena) -> SlipstreamResult<Self> {
        let mut visitor = Self {
            arena,

            bounding_volume: Box3::default(),

            node_tree: None,
            node_mix: None,
            draw_opaque: None,
            draw_xlu: None,

            skeleton_root: None,

            vertices: Vec::new(),
            normals: Vec::new(),
            colors: Vec::new(),
            uvs: Vec::new(),
            polygons: Vec::new(),
        };

        arena.walk(root, &mut visitor)?;
        Ok(visitor)
    }
}

impl Visitor for ModelContents<'_> {
    fn visit_mdl0(&mut self, model: VisitorContext<'_, Mdl0Root>) -> ControlFlow<()> {
        self.bounding_volume = model.header.bounding_volume;

        ControlFlow::Continue(())
    }

    fn visit_definitions(
        &mut self,
        definitions: VisitorContext<'_, Definitions>,
    ) -> ControlFlow<()> {
        match definitions.meta.label {
            DRAW_OPA_NAME => self.draw_opaque = Some(definitions.meta.key),
            DRAW_XLU_NAME => self.draw_xlu = Some(definitions.meta.key),
            NODE_TREE_NAME => self.node_tree = Some(definitions.meta.key),
            NODE_MIX_NAME => self.node_mix = Some(definitions.meta.key),
            _ => tracing::warn!("Unknown definitions file: `{}`", definitions.meta.label),
        }

        ControlFlow::Break(())
    }

    fn visit_bone(&mut self, bone: VisitorContext<'_, Bone>) -> ControlFlow<()> {
        // First bone we find should be the root, but still verify
        if bone.parent.is_some() {
            tracing::error!(
                "First bone in the model has an assigned parent while it should be the root"
            );
        }

        self.skeleton_root = Some(bone.meta.key);
        ControlFlow::Break(())
    }

    fn visit_vertices(&mut self, vertices: VisitorContext<'_, VertexBuffer>) -> ControlFlow<()> {
        self.vertices.push(vertices.meta.key);
        ControlFlow::Break(())
    }

    fn visit_normals(&mut self, normals: VisitorContext<'_, NormalBuffer>) -> ControlFlow<()> {
        self.normals.push(normals.meta.key);
        ControlFlow::Break(())
    }

    fn visit_colors(&mut self, colors: VisitorContext<'_, ColorBuffer>) -> ControlFlow<()> {
        self.colors.push(colors.meta.key);
        ControlFlow::Break(())
    }

    fn visit_uvs(&mut self, uvs: VisitorContext<'_, UvBuffer>) -> ControlFlow<()> {
        self.uvs.push(uvs.meta.key);
        ControlFlow::Break(())
    }

    fn visit_polygon(&mut self, polygon: VisitorContext<'_, Polygon>) -> ControlFlow<()> {
        self.polygons.push(polygon.meta.key);
        ControlFlow::Break(())
    }
}
