use crate::math::interp::{interpolate_f32, InterpRange, InterpolateMode};
use crate::params::{Param, ParamCtx, ParamUuid};
use std::collections::HashMap;

/// Animation loop mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationLoopMode {
	#[default]
	Once,
	Loop,
	PingPong,
}

/// A keyframe in an animation track.
#[derive(Debug, Clone, Copy)]
pub struct Keyframe {
	pub time: f32,
	pub value: f32,
	pub interpolation: InterpolateMode,
}

/// Axis of a parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationAxis {
	X,
	Y,
}

/// A track in an animation clip, targeting a specific parameter axis.
#[derive(Debug, Clone)]
pub struct AnimationTrack {
	pub param_uuid: ParamUuid,
	pub axis: AnimationAxis,
	pub keyframes: Vec<Keyframe>,
}

impl AnimationTrack {
	pub fn sample(&self, time: f32) -> f32 {
		if self.keyframes.is_empty() {
			return 0.0;
		}

		let first = self.keyframes.first().unwrap();
		if time <= first.time {
			return first.value;
		}

		let last = self.keyframes.last().unwrap();
		if time >= last.time {
			return last.value;
		}

		// Find the two keyframes to interpolate between
		let idx = self
			.keyframes
			.binary_search_by(|k| k.time.total_cmp(&time))
			.unwrap_or_else(|i| i);

		let k1 = &self.keyframes[idx - 1];
		let k2 = &self.keyframes[idx];

		interpolate_f32(
			time,
			InterpRange::new(k1.time, k2.time),
			InterpRange::new(k1.value, k2.value),
			k2.interpolation,
		)
	}
}

/// An animation clip.
#[derive(Debug, Clone)]
pub struct Animation {
	pub name: String,
	pub length: f32,
	pub loop_mode: AnimationLoopMode,
	pub tracks: Vec<AnimationTrack>,
}

impl Animation {
	pub fn sample(&self, time: f32) -> HashMap<(ParamUuid, AnimationAxis), f32> {
		let mut values = HashMap::new();
		for track in &self.tracks {
			values.insert((track.param_uuid, track.axis), track.sample(time));
		}
		values
	}
}

/// Context for playing animations on a puppet.
pub struct AnimationCtx {
	pub current_animations: HashMap<String, AnimationState>,
	/// Mapping from UUID to name and is_vec2 for faster lookup during apply.
	pub param_info: HashMap<ParamUuid, (String, bool)>,
}

pub struct AnimationState {
	pub time: f32,
	pub weight: f32,
	pub playing: bool,
}

impl AnimationCtx {
	pub fn new(params: &HashMap<String, Param>) -> Self {
		let mut param_info = HashMap::new();
		for (name, param) in params {
			param_info.insert(param.uuid, (name.clone(), param.is_vec2));
		}

		Self {
			current_animations: HashMap::new(),
			param_info,
		}
	}

	pub fn play(&mut self, name: &str, weight: f32) {
		self.current_animations.insert(
			name.to_owned(),
			AnimationState {
				time: 0.0,
				weight,
				playing: true,
			},
		);
	}

	pub fn clear(&mut self) {
		self.current_animations.clear();
	}

	pub fn update(&mut self, dt: f32) {
		for state in self.current_animations.values_mut() {
			if state.playing {
				state.time += dt;
			}
		}
	}

	pub fn apply(&self, animations: &[Animation], param_ctx: &mut ParamCtx) {
		// Collect all parameter values
		let mut param_values = HashMap::<ParamUuid, glam::Vec2>::new();

		for anim in animations {
			if let Some(state) = self.current_animations.get(&anim.name) {
				let effective_time = state.get_effective_time(anim);
				let sampled = anim.sample(effective_time);

				for ((uuid, axis), value) in sampled {
					let entry = param_values.entry(uuid).or_default();
					match axis {
						AnimationAxis::X => entry.x = value * state.weight,
						AnimationAxis::Y => entry.y = value * state.weight,
					}
				}
			}
		}

		// Apply to param_ctx
		for (uuid, value) in param_values {
			if let Some((name, _is_vec2)) = self.param_info.get(&uuid) {
				let _ = param_ctx.set(name, value);
			}
		}
	}
}

impl AnimationState {
	pub fn get_effective_time(&self, anim: &Animation) -> f32 {
		match anim.loop_mode {
			AnimationLoopMode::PingPong => {
				let double_len = anim.length * 2.0;
				let t = if double_len > 0.0 { self.time % double_len } else { 0.0 };
				if t > anim.length {
					2.0 * anim.length - t
				} else {
					t
				}
			}
			AnimationLoopMode::Loop => {
				if anim.length > 0.0 {
					self.time % anim.length
				} else {
					0.0
				}
			}
			AnimationLoopMode::Once => self.time.min(anim.length),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::params::AxisPoints;
	use crate::puppet::Puppet;

	#[test]
	fn test_animation_sampling() {
		let track = AnimationTrack {
			param_uuid: ParamUuid(1),
			axis: AnimationAxis::X,
			keyframes: vec![
				Keyframe {
					time: 0.0,
					value: 0.0,
					interpolation: InterpolateMode::Linear,
				},
				Keyframe {
					time: 1.0,
					value: 10.0,
					interpolation: InterpolateMode::Linear,
				},
			],
		};

		assert_eq!(track.sample(0.0), 0.0);
		assert_eq!(track.sample(0.5), 5.0);
		assert_eq!(track.sample(1.0), 10.0);
		assert_eq!(track.sample(1.5), 10.0);
	}

	#[test]
	fn test_ping_pong_loop() {
		let anim = Animation {
			name: "test".to_owned(),
			length: 1.0,
			loop_mode: AnimationLoopMode::PingPong,
			tracks: vec![],
		};
		let mut state = AnimationState {
			time: 0.0,
			weight: 1.0,
			playing: true,
		};

		assert_eq!(state.get_effective_time(&anim), 0.0);
		state.time = 0.5;
		assert_eq!(state.get_effective_time(&anim), 0.5);
		state.time = 1.0;
		assert_eq!(state.get_effective_time(&anim), 1.0);
		state.time = 1.5;
		assert_eq!(state.get_effective_time(&anim), 0.5);
		state.time = 2.0;
		assert_eq!(state.get_effective_time(&anim), 0.0);
		state.time = 2.5;
		assert_eq!(state.get_effective_time(&anim), 0.5);
	}

	#[test]
	fn test_animation_context_apply() {
		let mut params = HashMap::new();
		params.insert(
			"Head".to_owned(),
			Param {
				uuid: ParamUuid(1),
				name: "Head".to_owned(),
				is_vec2: true,
				min: glam::Vec2::splat(-1.0),
				max: glam::Vec2::splat(1.0),
				defaults: glam::Vec2::ZERO,
				axis_points: AxisPoints {
					x: vec![0.0, 0.5, 1.0],
					y: vec![0.0, 0.5, 1.0],
				},
				bindings: vec![],
			},
		);

		let anim = Animation {
			name: "Idle".to_owned(),
			length: 1.0,
			loop_mode: AnimationLoopMode::Once,
			tracks: vec![AnimationTrack {
				param_uuid: ParamUuid(1),
				axis: AnimationAxis::X,
				keyframes: vec![
					Keyframe {
						time: 0.0,
						value: 0.0,
						interpolation: InterpolateMode::Linear,
					},
					Keyframe {
						time: 1.0,
						value: 1.0,
						interpolation: InterpolateMode::Linear,
					},
				],
			}],
		};

		let mut animation_ctx = AnimationCtx::new(&params);
		animation_ctx.play("Idle", 1.0);

		let mut puppet = Puppet::new(
			crate::puppet::meta::PuppetMeta::default(),
			crate::physics::PuppetPhysics::default(),
			crate::node::InoxNode {
				uuid: crate::node::InoxNodeUuid(0),
				name: "Root".to_owned(),
				enabled: true,
				zsort: 0.0,
				trans_offset: Default::default(),
				lock_to_root: false,
			},
			params,
			vec![anim.clone()],
		);
		puppet.init_transforms();
		puppet.init_rendering();
		puppet.init_params();

		let mut param_ctx = puppet.param_ctx.take().unwrap();

		// Sample at t=0.5
		animation_ctx.current_animations.get_mut("Idle").unwrap().time = 0.5;
		animation_ctx.apply(&[anim], &mut param_ctx);

		assert_eq!(param_ctx.values.get("Head").unwrap().x, 0.5);

		puppet.param_ctx = Some(param_ctx);
	}
}
