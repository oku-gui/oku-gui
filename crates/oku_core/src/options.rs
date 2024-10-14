use std::fmt::{Display, Formatter};

#[derive(Default)]
pub struct OkuOptions {
    pub renderer: RendererType,
}

#[derive(Default, Copy, Clone, Debug)]
pub enum RendererType {
    Software,
    #[default]
    Wgpu,
}

impl Display for RendererType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RendererType::Software => write!(f, "software(tiny-skia)"),
            RendererType::Wgpu => write!(f, "wgpu"),
        }
    }
}
