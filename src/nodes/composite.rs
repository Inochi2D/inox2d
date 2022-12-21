use super::drawable::Drawable;

#[derive(Debug, Clone)]
pub struct Composite {
    pub(crate) draw_state: Drawable,
}
