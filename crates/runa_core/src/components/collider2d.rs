use glam::Vec2;

/// Simple axis-aligned 2D collider represented by half extents.
#[derive(Clone, Copy, Debug, Default)]
pub struct Collider2D {
    pub half_size: Vec2,
    pub enabled: bool,
    pub is_trigger: bool,
}

impl Collider2D {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            enabled: true,
            is_trigger: true,
        }
    }

    pub fn with_half_size(half_size: Vec2) -> Self {
        Self {
            half_size,
            enabled: true,
            is_trigger: true,
        }
    }

    pub fn min(&self, center: Vec2) -> Vec2 {
        center - self.half_size
    }

    pub fn max(&self, center: Vec2) -> Vec2 {
        center + self.half_size
    }

    pub fn contains_point(&self, point: Vec2, center: Vec2) -> bool {
        if !self.enabled {
            return false;
        }

        let min = self.min(center);
        let max = self.max(center);
        point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
    }

    pub fn intersects(&self, center: Vec2, other: &Collider2D, other_center: Vec2) -> bool {
        if !self.enabled || !other.enabled {
            return false;
        }

        let min = self.min(center);
        let max = self.max(center);
        let other_min = other.min(other_center);
        let other_max = other.max(other_center);

        min.x <= other_max.x
            && max.x >= other_min.x
            && min.y <= other_max.y
            && max.y >= other_min.y
    }
}
