use std::ops::ControlFlow;

use crate::kmp::enph::EnemyRoutes;
use crate::kmp::enpt::EnemyRoutePoints;
use crate::kmp::{
    Areas, CheckpointGroups, Checkpoints, GeoObjects, ItemRoutePoints, ItemRoutes, KmpHeader, PointInformationPaths, Respawns, StartingPositions
};
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

    fn visit_enemy_routes(&self, context: VisitorCtx<'_, EnemyRoutes>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_enemy_routes_mut(
        &mut self,
        context: VisitorCtxMut<'_, EnemyRoutes>
    ) -> ControlFlow<()> {
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

    fn visit_item_route_points(&self, context: VisitorCtx<'_, ItemRoutePoints>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_item_route_points_mut(
        &mut self,
        context: VisitorCtxMut<'_, ItemRoutePoints>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_item_routes(&mut self, context: VisitorCtx<'_, ItemRoutes>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_item_routes_mut(&mut self, context: VisitorCtxMut<'_, ItemRoutes>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_checkpoints(&mut self, context: VisitorCtx<'_, Checkpoints>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_checkpoints_mut(
        &mut self,
        context: VisitorCtxMut<'_, Checkpoints>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_checkpoint_groups(
        &mut self,
        context: VisitorCtx<'_, CheckpointGroups>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_checkpoint_groups_mut(
        &mut self,
        context: VisitorCtxMut<'_, CheckpointGroups>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_geo_objects(&mut self, context: VisitorCtx<'_, GeoObjects>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_geo_objects_mut(&mut self, context: VisitorCtxMut<'_, GeoObjects>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_point_information(
        &mut self,
        context: VisitorCtx<'_, PointInformationPaths>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_point_information_mut(
        &mut self,
        context: VisitorCtxMut<'_, PointInformationPaths>
    ) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_areas(&mut self, context: VisitorCtx<'_, Areas>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_areas_mut(&mut self, context: VisitorCtxMut<'_, Areas>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_respawns(&mut self, context: VisitorCtx<'_, Respawns>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn visit_respawns_mut(&mut self, context: VisitorCtxMut<'_, Respawns>) -> ControlFlow<()> {
        if self.stop_when_uninterested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}
