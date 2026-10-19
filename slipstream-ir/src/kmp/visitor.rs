use std::ops::ControlFlow;

use crate::brres::{
    Bone, ColorBuffer, Definitions, Material, MdlHeader, NormalBuffer, PaletteLinks, Polygon, Tev,
    TextureLinks, UvBuffer, VertexBuffer
};
use crate::kmp::enph::EnemyRoute;
use crate::kmp::enpt::EnemyRoutePoints;
use crate::kmp::{KmpHeader, StartingPositions};
use crate::visitor::{VisitorCtx, VisitorCtxMut};

#[allow(unused_variables)]
pub trait KmpVisitor {
    fn stop_when_uninterested(&self) -> bool {
        false
    }

    fn visit_kmp(&mut self, context: VisitorCtx<'_, KmpHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_kmp_mut(&mut self, context: VisitorCtxMut<'_, KmpHeader>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_starting_positions(
        &self,
        context: VisitorCtx<'_, StartingPositions>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_starting_positions_mut(
        &mut self,
        context: VisitorCtxMut<'_, StartingPositions>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_enemy_route(&self, context: VisitorCtx<'_, EnemyRoute>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_enemy_route_mut(&mut self, context: VisitorCtxMut<'_, EnemyRoute>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_enemy_route_points(
        &self,
        context: VisitorCtx<'_, EnemyRoutePoints>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_enemy_route_points_mut(
        &mut self,
        context: VisitorCtxMut<'_, EnemyRoutePoints>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
