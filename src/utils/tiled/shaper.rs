//! Don't look like Bevy until load
use std::io::Cursor;
use std::path::Path;
use std::sync::Arc;

use avian2d::{
    parry::{
        math::Isometry,
        na::{Const, Isometry2, OPoint},
        shape::{Compound, SharedShape},
        transformation::vhacd::{VHACD, VHACDParameters},
    },
    prelude::*,
};
use bevy::{
    asset::{AssetLoader, io::Reader},
    log::{info, warn},
    platform::collections::HashMap,
    prelude::*,
    reflect::TypePath,
};
use bevy_ecs_tilemap::prelude::*;
use thiserror::Error;
use tiled::{ObjectData, ObjectShape};

use crate::{
    // TODO: asset_tracking::LoadResource,
    game::player::PLAYER_Z_TRANSLATION,
};

type Point2 = OPoint<f32, Const<2>>;

pub fn shaper(shape: &ObjectShape) -> SharedShape {
    use ObjectShape::*;
    match shape {
        Rect { width, height } => SharedShape::cuboid(width / 2.0, height / 2.0),
        // https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#ellipse
        Ellipse { width, height } => {
            unimplemented!();
        }
        // (Editor) lining with the "Polygon" option, do not finish it
        Polyline { points } => {
            if points.is_empty() {
                panic!("The data (tmx) is corrupted");
            } else if points.len() == 2 {
                // Single segment
                SharedShape::segment(
                    Point2::new(points[0].0, points[0].1),
                    Point2::new(points[1].0, points[1].1),
                )
            } else {
                let vertices: Vec<Point2> =
                    points.iter().map(|(x, y)| Point2::new(*x, *y)).collect();
                let n = vertices.len();
                let indices: Vec<[u32; 2]> = (0..n).map(|i| [i as u32, (i + 1) as u32]).collect();
                SharedShape::polyline(vertices, Some(indices))
            }
        }
        Polygon { points } => {
            if points.len() < 3 {
                panic!("The data (tmx) is corrupted");
            }

            let vertices: Vec<Point2> = points.iter().map(|(x, y)| Point2::new(*x, *y)).collect();
            let n = vertices.len();
            let indices: Vec<[u32; 2]> = (0..n).map(|i| [i as u32, ((i + 1) % n) as u32]).collect();
            // let decomposition = VHACD::decompose(&VHACDParameters::default(), &vertices, &indices, false);
            SharedShape::convex_decomposition_with_params(
                &vertices,
                &indices,
                &VHACDParameters::default(),
            )
        }
        Point(_x, _y) => {
            unimplemented!();
        }
        Text { .. } => {
            unimplemented!();
        }
    }
}

#[allow(dead_code)]
pub fn get_shared_shape(shape: &ObjectShape) -> Option<SharedShape> {
    use ObjectShape::*;
    match shape {
        Rect { width, height } => Some(SharedShape::cuboid(width / 2.0, height / 2.0)),
        // https://doc.mapeditor.org/en/stable/reference/tmx-map-format/#ellipse
        Ellipse { width, height } => {
            unimplemented!();
        }
        // (Editor) lining with the "Polygon" option, do not finish it
        Polyline { points } => {
            if points.is_empty() {
                panic!("The data (tmx) is corrupted");
            } else if points.len() == 2 {
                // Single segment
                Some(SharedShape::segment(
                    Point2::new(points[0].0, points[0].1),
                    Point2::new(points[1].0, points[1].1),
                ))
            } else {
                let vertices: Vec<Point2> =
                    points.iter().map(|(x, y)| Point2::new(*x, *y)).collect();
                let n = vertices.len();
                let indices: Vec<[u32; 2]> = (0..n).map(|i| [i as u32, (i + 1) as u32]).collect();
                Some(SharedShape::polyline(vertices, Some(indices)))
            }
        }
        Polygon { points } => {
            if points.len() < 3 {
                panic!("The data (tmx) is corrupted");
            }

            let vertices: Vec<Point2> = points.iter().map(|(x, y)| Point2::new(*x, *y)).collect();
            let n = vertices.len();
            let indices: Vec<[u32; 2]> = (0..n).map(|i| [i as u32, ((i + 1) % n) as u32]).collect();
            // let decomposition = VHACD::decompose(&VHACDParameters::default(), &vertices, &indices, false);
            Some(SharedShape::convex_decomposition_with_params(
                &vertices,
                &indices,
                &VHACDParameters::default(),
            ))
        }
        Point(_x, _y) => {
            unimplemented!();
        }
        Text { .. } => {
            unimplemented!();
        }
    }
}

#[allow(dead_code)]
pub struct PreSharedShape {
    tile_id: tiled::TileId,
    objects: Vec<ObjectData>,
}
#[allow(dead_code)]
impl PreSharedShape {
    pub fn new(tile_id: tiled::TileId, objects: Vec<ObjectData>) -> Self {
        Self { tile_id, objects }
    }

    // from `ObjectLayerData::object_data()`
    pub fn from_object_data(tile_id: tiled::TileId, object_datas: &[ObjectData]) -> Self {
        Self {
            tile_id,
            objects: object_datas.to_vec(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ObjectData> {
        self.objects.iter()
    }

    pub fn to_shared_shape(&self) -> Option<SharedShape> {
        if self.objects.is_empty() {
            return None;
        }
        if self.objects.len() == 1 {
            return get_shared_shape(&self.objects[0].shape);
        }
        let mut compound = Vec::<(Isometry2<f32>, SharedShape)>::new();

        for obj in &self.objects {
            if let Some(shared_shape) = get_shared_shape(&obj.shape) {
                if shared_shape.as_composite_shape().is_some() {
                    panic!("Nested composite shapes are not allowed.");
                }
                println!("obj x,y: {} {}", obj.x, obj.y);
                compound.push((Isometry2::translation(obj.x, obj.y), shared_shape));
            }
        }

        if compound.is_empty() {
            None
        } else {
            Some(SharedShape::new(Compound::new(compound)))
            //Some(SharedShape::compound(compound))
        }
    }
}
