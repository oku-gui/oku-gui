use crate::user::components::component::{ComponentId, GenericUserState};
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};
use std::collections::HashMap;
use std::time::Instant;
use taffy::{AvailableSpace, Size};

pub struct TaffyTextContext {
    pub id: ComponentId,
    pub metrics: Metrics,
    pub text: String,
}

impl TaffyTextContext {
    pub fn new(id: ComponentId, metrics: Metrics, text: String) -> Self {
        Self { id, metrics, text }
    }
}

pub struct CosmicTextContent {
    pub id: ComponentId,
    pub buffer: Buffer,
    pub metrics: Metrics,
}

impl CosmicTextContent {
    pub(crate) fn new(
        id: ComponentId,
        metrics: Metrics,
        text: &str,
        attrs: Attrs,
        font_system: &mut FontSystem,
    ) -> Self {
        //let mut buffer = Buffer::new(font_system, metrics);
        //buffer.set_text(font_system, text, attrs, Shaping::Basic);
        Self {
            id,
            metrics,
            buffer: Buffer::new(font_system, metrics),
        }
    }

    fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
        font_system: &mut FontSystem,
    ) -> Size<f32> {
        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });

        let height_constraint = known_dimensions.height.or(match available_space.height {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(height) => Some(height),
        });

        self.buffer.set_size(font_system, width_constraint, height_constraint);
        // Compute layout
        self.buffer.shape_until_scroll(font_system, true);

        // Determine measured size of text
        let (width, total_lines) = self
            .buffer
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
        let height = total_lines as f32 * self.buffer.metrics().line_height;

        Size { width, height }
    }
}

pub struct ImageContext {
    pub width: f32,
    pub height: f32,
}

impl ImageContext {
    pub fn measure(
        &mut self,
        known_dimensions: Size<Option<f32>>,
        _available_space: Size<AvailableSpace>,
    ) -> Size<f32> {
        match (known_dimensions.width, known_dimensions.height) {
            (Some(width), Some(height)) => Size { width, height },
            (Some(width), None) => Size {
                width,
                height: (width / self.width) * self.height,
            },
            (None, Some(height)) => Size {
                width: (height / self.height) * self.width,
                height,
            },
            (None, None) => Size {
                width: self.width,
                height: self.height,
            },
        }
    }
}

pub enum LayoutContext {
    Text(TaffyTextContext),
    Image(ImageContext),
}

pub fn measure_content(
    element_state: &mut HashMap<ComponentId, Box<GenericUserState>>,
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    node_context: Option<&mut LayoutContext>,
    font_system: &mut FontSystem,
) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = known_dimensions
    {
        return Size { width, height };
    }

    match node_context {
        None => Size::ZERO,
        Some(LayoutContext::Text(taffy_text_context)) => {
            let cosmic_text_content: &mut CosmicTextContent = if let Some(cosmic_text_content) =
                element_state.get_mut(&taffy_text_context.id).unwrap().downcast_mut()
            {
                cosmic_text_content
            } else {
                let mut buffer = Buffer::new(font_system, taffy_text_context.metrics);
                let x = CosmicTextContent {
                    id: taffy_text_context.id,
                    buffer,
                    metrics: taffy_text_context.metrics,
                };

                element_state.insert(taffy_text_context.id.clone(), Box::new(x));
                element_state.get_mut(&taffy_text_context.id).unwrap().downcast_mut().unwrap()
            };

            cosmic_text_content.buffer.set_text(font_system, &taffy_text_context.text, cosmic_text::Attrs::new(), Shaping::Advanced);
            cosmic_text_content.measure(known_dimensions, available_space, font_system)
        }
        Some(LayoutContext::Image(image_context)) => image_context.measure(known_dimensions, available_space),
    }
}
