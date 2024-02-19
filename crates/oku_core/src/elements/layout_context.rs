use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};
use taffy::{AvailableSpace, Size};

pub struct CosmicTextContent {
    pub buffer: Buffer,
}

impl CosmicTextContent {
    pub(crate) fn new(metrics: Metrics, text: &str, attrs: Attrs, font_system: &mut FontSystem) -> Self {
        let mut buffer = Buffer::new_empty(metrics);
        buffer.set_size(font_system, f32::INFINITY, f32::INFINITY);
        buffer.set_text(font_system, text, attrs, Shaping::Advanced);
        Self { buffer }
    }

    fn measure(&mut self, known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>, font_system: &mut FontSystem) -> Size<f32> {
        // Set width constraint
        let width_constraint = known_dimensions.width.unwrap_or(match available_space.width {
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => f32::INFINITY,
            AvailableSpace::Definite(width) => width,
        });

        let _height_constraint = known_dimensions.height.unwrap_or(match available_space.height {
            AvailableSpace::MinContent => 0.0,
            AvailableSpace::MaxContent => f32::INFINITY,
            AvailableSpace::Definite(height) => height,
        });
        // self.buffer.set_size(font_system, width_constraint, height_constraint);
        self.buffer.set_size(font_system, width_constraint, f32::INFINITY);

        // Compute layout
        self.buffer.shape_until_scroll(font_system, true);

        // Determine measured size of text
        let (width, total_lines) = self.buffer.layout_runs().fold((0.0, 0usize), |(width, total_lines), run| (run.line_w.max(width), total_lines + 1));
        let height = total_lines as f32 * self.buffer.metrics().line_height;

        Size { width, height }
    }
}

pub enum LayoutContext {
    Text(CosmicTextContent),
}

pub fn measure_content(known_dimensions: Size<Option<f32>>, available_space: Size<AvailableSpace>, node_context: Option<&mut LayoutContext>, font_system: &mut FontSystem) -> Size<f32> {
    if let Size {
        width: Some(width),
        height: Some(height),
    } = known_dimensions
    {
        return Size { width, height };
    }

    match node_context {
        None => Size::ZERO,
        Some(LayoutContext::Text(text_context)) => text_context.measure(known_dimensions, available_space, font_system),
    }
}
