use slipstream_ir::{
    mdl0::{
        Bone, BoneBind, BoneIndex, BoneWeight, DefCommand, Definitions, MatrixId, Polygon, WeightId,
    },
    node::arena::IrArena,
    visitor::{Visitor, VisitorContext},
};
use slipstream_shared::{SlipstreamResult, try_unwrap};
use std::{collections::HashMap, ops::ControlFlow};

use crate::viewer::translation::{
    IntermediateModel, IntermediatePolygon, MAX_BONE_INFLUENCES, ModelContents, VertexKey,
};

/// Maps bone IDs to matrices in the matrix table
#[derive(Default, Debug)]
pub struct BoneMap {
    /// Maps bones to their corresponding matrices.
    matrix_map: HashMap<BoneIndex, MatrixId>,
}

impl BoneMap {
    /// Obtains the matrix corresponding to the given bone.
    pub fn get_matrix(&self, bone_index: BoneIndex) -> Option<MatrixId> {
        self.matrix_map.get(&bone_index).copied()
    }

    /// Generates a bone map from a `Definitions` entry. All commands other than
    /// [`MapNode`] will be ignored.
    ///
    /// [`MapNode`]: slipstream_ir::mdl0::definitions::BytecodeCommand
    pub fn from_definitions(definitions: &Definitions) -> SlipstreamResult<Self> {
        let mut map = HashMap::with_capacity(definitions.commands.len());
        for cmd in &definitions.commands {
            match cmd {
                DefCommand::MapNode(mapping) => {
                    map.insert(mapping.bone_index, mapping.matrix_index);
                }
                _ => tracing::warn!("Unexpected command `{cmd:?}` in `NodeTree`, skipping it"),
            }
        }

        Ok(Self { matrix_map: map })
    }
}

/// Contains the model's bone weights.
///
/// There are two different [`Definitions`] commands that control the bone weights.
/// [`Weights`] contains the actual data while [`WeightIndex`] contains indices into this
/// block of data.
///
/// The method is similar to how vertex buffers and index buffers work in graphics APIs.
///
/// [`Definitions`]: slipstream_ir::mdl0::definitions::Definitions
/// [`Weights`]: slipstream_ir::mdl0::definitions::Weights
/// [`WeightIndex`]: slipstream_ir::mdl0::definitions::WeightIndex
#[derive(Default, Debug)]
pub struct BoneWeights {
    // Maps a matrix to its corresponding entry in `weights`.
    indices: HashMap<MatrixId, WeightId>,
    // Actual weight data.
    weights: HashMap<WeightId, Vec<BoneWeight>>,
}

impl BoneWeights {
    /// Obtains the matrix's bone weights.
    pub fn get_by_matrix_id(&self, matrix: MatrixId) -> Option<&[BoneWeight]> {
        let index = self.indices.get(&matrix)?;
        self.weights.get(index).map(|v| v.as_slice())
    }

    /// Obtains the bone weights by their weight ID.
    pub fn get_by_weight_id(&self, weight: WeightId) -> Option<&[BoneWeight]> {
        self.weights.get(&weight).map(|v| v.as_slice())
    }

    /// Generates `BoneWeights` from a `Definitions` entry. All commands other than
    /// [`Weights`] and [`WeightIndex`] will be ignored.
    ///
    /// [`Weights`]: slipstream_ir::mdl0::definitions::Weights
    /// [`WeightIndex`]: slipstream_ir::mdl0::definitions::WeightIndex
    pub fn from_definitions(definitions: &Definitions) -> SlipstreamResult<Self> {
        let mut index_map = HashMap::new();
        let mut weight_map = HashMap::new();

        for cmd in &definitions.commands {
            match cmd {
                DefCommand::Weights(weights) => {
                    weight_map.insert(weights.id, weights.weights.clone());

                    let weight_sum = weights.weights.iter().fold(0.0, |acc, w| acc + w.weight);

                    if (weight_sum - 1.0).abs() > 0.001 {
                        tracing::error!(
                            "Weights of ID {} do not add up to 1.0 ({weight_sum})",
                            weights.id.0
                        );
                    }
                }
                DefCommand::WeightIndex(index) => {
                    index_map.insert(index.matrix_id, index.weight_id);
                }
                _ => tracing::warn!("Unexpected command `{cmd:?}` in `NodeMix`, skipping it"),
            }
        }

        Ok(Self {
            indices: index_map,
            weights: weight_map,
        })
    }
}

pub(super) struct VertexBoneData {
    pub ids: [u32; MAX_BONE_INFLUENCES],
    pub weights: [f32; MAX_BONE_INFLUENCES],
}

impl ModelContents<'_> {
    pub(super) fn translate_bones(
        &self,
        model: &IntermediateModel,
        scratch: &IntermediatePolygon,
        polygon: &Polygon,
        vertex_key: &VertexKey,
    ) -> SlipstreamResult<VertexBoneData> {
        const WEIGHTS_DEFAULT: [f32; MAX_BONE_INFLUENCES] = {
            let mut def = [0.0; MAX_BONE_INFLUENCES];
            def[0] = 1.0;
            def
        };

        Ok(match &polygon.bone_bind {
            BoneBind::Rigid(rigid) => {
                // If the bone binding is rigid, we don't even have to look at the vertex's
                // pn index.

                // `rigid` refers to the bone's ID field.

                tracing::debug!("RIGID {rigid}");

                // Find the matrix corresponding to this bone.
                let matrix_id = model.bone_map.get_matrix(BoneIndex(*rigid as u16)).unwrap();

                let mut bone_ids = [0; MAX_BONE_INFLUENCES];
                bone_ids[0] = matrix_id.0 as u32;

                let mut weights = WEIGHTS_DEFAULT;
                weights[3] = -42.0;

                VertexBoneData {
                    ids: bone_ids,
                    weights,
                }
            }
            BoneBind::Mixed(mixed) => {
                // The polygon has a bone table.

                // The PNMTXID (position-normal matrix ID) should be present in the vertex when
                // the polygon has a bone table.
                let pn_id = vertex_key.mtx_id.unwrap_or_else(|| {
                    tracing::error!("Missing PNMTXIDX for vertex, attaching it to bone 0. This might mess up animations and model structure.");
                    0
                });

                let matrix_id = scratch.xf_registers.positions[pn_id as usize];
                tracing::debug!(
                    pn_id = ?pn_id,
                    matrix_id = ?matrix_id
                );

                //                 // Using the bone table, we map the vertex's PNMTXID to a bone index.
                //                 let bone_index = *try_unwrap!(
                //                     mixed.entries.get(pn_id as usize),
                //                     "bone table index out of range: {pn_id}"
                //                 )?;
                //
                //                 // then map the bone index to a matrix id.
                //                 let matrix_id = model.bone_map.get_matrix(BoneIndex(bone_index)).unwrap();

                // Check if weights are involved
                match &model.bone_weights {
                    Some(weights) => {
                        // The bone influences are weighted. This is used to control how a vertex
                        // moves when it's influenced by multiple bones.

                        tracing::debug!("matrix ID {matrix_id:?}");

                        let mut bone_indices = [0; MAX_BONE_INFLUENCES];
                        bone_indices[0] = matrix_id.0 as u32;

                        VertexBoneData {
                            ids: bone_indices,
                            weights: [1.0, 0.0, 0.0, -1.0],
                        }
                    }
                    None => {
                        // The polygon has multiple different bones affecting it.
                        // As the model has no weights, this usually means that still only
                        // one bone affects this vertex.

                        let mut bone_indices = [0; MAX_BONE_INFLUENCES];
                        bone_indices[0] = matrix_id.0 as u32;

                        VertexBoneData {
                            ids: bone_indices,
                            weights: [1.0, 0.0, 0.0, -2.0],
                        }
                    }
                }
            }
        })
    }

    pub(super) fn traverse_skeleton(
        &self,
        out: &mut IntermediateModel,
        arena: &IrArena,
    ) -> SlipstreamResult<()> {
        struct RecursiveBoneVisitor<'a> {
            arena: &'a IrArena,
            transforms: HashMap<BoneIndex, glam::Mat4>,
            result: SlipstreamResult<()>,
        }

        impl Visitor for RecursiveBoneVisitor<'_> {
            fn visit_bone(&mut self, bone: VisitorContext<'_, Bone>) -> ControlFlow<()> {
                // let [a, b, c] = bone.rotation_vector;
                // let mat = glam::Mat4::from_scale_rotation_translation(
                //     glam::Vec3::from_array(bone.scaling_vector),
                //     glam::Quat::from_euler(
                //         glam::EulerRot::XYZEx,
                //         a.to_radians(),
                //         b.to_radians(),
                //         c.to_radians(),
                //     ),
                //     glam::Vec3::from_array(bone.translation_vector),
                // );

                let m = &bone.transform_matrix;
                let mat = glam::mat4(
                    glam::vec4(m[0], m[4], m[8], 0.0),
                    glam::vec4(m[1], m[5], m[9], 0.0),
                    glam::vec4(m[2], m[6], m[10], 0.0),
                    glam::vec4(m[3], m[7], m[11], 1.0),
                );

                self.transforms.insert(BoneIndex(bone.index as u16), mat);

                for child in bone.meta.children {
                    let mut child_visitor = Self {
                        arena: self.arena,
                        transforms: HashMap::new(),
                        result: Ok(()),
                    };

                    self.arena
                        .visit(*child, &mut child_visitor)
                        .expect("failed to iterate over bone children");

                    self.transforms.extend(child_visitor.transforms.iter());
                }

                ControlFlow::Break(())
            }
        }

        let skeleton_root = try_unwrap!(self.skeleton_root, "skeleton root was not found")?;

        let mut visitor = RecursiveBoneVisitor {
            arena,
            transforms: HashMap::new(),
            result: Ok(()),
        };
        arena.visit(skeleton_root, &mut visitor)?;
        visitor.result?;

        // Find max bone ID to resize the vector.
        let max_bone_index = visitor
            .transforms
            .keys()
            .max()
            .copied()
            .unwrap_or(BoneIndex(0));

        out.matrix_table
            .resize(max_bone_index.0 as usize + 1, glam::Mat4::IDENTITY);

        for (bone_index, transform) in visitor.transforms {
            let matrix_id = out.bone_map.get_matrix(bone_index).unwrap();

            if out.matrix_table.len() <= matrix_id.0 as usize {
                out.matrix_table
                    .resize(matrix_id.0 as usize + 1, glam::Mat4::IDENTITY);
            }

            tracing::trace!("Set {matrix_id:?} for {bone_index:?}");
            out.matrix_table[matrix_id.0 as usize] = transform;
        }

        Ok(())
    }
}
