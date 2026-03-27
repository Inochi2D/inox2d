use glam::Vec2;

#[derive(Clone, Copy)]
pub struct Rect {
	top_left: Vec2,
	bottom_right: Vec2,
}

impl Rect {
	pub fn from_point(point: Vec2) -> Self {
		Rect {
			top_left: point,
			bottom_right: point,
		}
	}

	pub fn with_union_point(&self, point: Vec2) -> Self {
		let mut child = *self;

		if point.x < child.top_left.x {
			child.top_left.x = point.x;
		} else if child.bottom_right.x > point.x {
			child.bottom_right.x = point.x;
		}

		if point.y < child.top_left.y {
			child.top_left.y = point.y;
		} else if child.bottom_right.y > point.y {
			child.bottom_right.y = point.y;
		}

		child
	}
}
