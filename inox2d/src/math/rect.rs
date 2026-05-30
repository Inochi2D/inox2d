use glam::Vec2;

#[derive(Clone, Copy)]
pub struct RectBounds {
	top_left: Vec2,
	bottom_right: Vec2,
}

impl RectBounds {
	pub fn from_point(point: Vec2) -> Self {
		RectBounds {
			top_left: point,
			bottom_right: point,
		}
	}

	pub fn with_union_point(&self, point: Vec2) -> Self {
		let mut child = *self;

		if point.x < child.top_left.x {
			child.top_left.x = point.x;
		} else if point.x > child.bottom_right.x {
			child.bottom_right.x = point.x;
		}

		if point.y < child.top_left.y {
			child.top_left.y = point.y;
		} else if point.y > child.bottom_right.y {
			child.bottom_right.y = point.y;
		}

		child
	}

	pub fn width(&self) -> f32 {
		self.bottom_right.x - self.top_left.x
	}

	pub fn height(&self) -> f32 {
		self.bottom_right.y - self.top_left.y
	}

	pub fn top_left_point(&self) -> Vec2 {
		self.top_left
	}

	pub fn bottom_right_point(&self) -> Vec2 {
		self.bottom_right
	}
}
