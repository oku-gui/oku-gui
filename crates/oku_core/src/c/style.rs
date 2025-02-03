pub use crate::style::Display;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum UnitType {
    Px,
    Percentage,
    Auto,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Unit {
    pub unit_type: UnitType,
    pub value: f32,
}

impl Unit {
    fn to_rust(&self) -> crate::style::Unit {
        match self.unit_type {
            UnitType::Px => crate::style::Unit::Px(self.value),
            UnitType::Percentage => crate::style::Unit::Percentage(self.value),
            UnitType::Auto => crate::style::Unit::Auto,
        }
    }
    
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[no_mangle]
    pub extern "C" fn color_rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {
            r: r as f32,
            g: g as f32,
            b: b as f32,
            a: a as f32,
        }
    }

}

#[repr(C)]
pub struct Style {
    color: Color,
    margin: [Unit; 4],
    display: Display
}

#[no_mangle]
pub extern "C" fn default_styles() -> Style {
    Style {
        color: Color::color_rgba(0, 0, 0, 255),
        margin: [Unit { unit_type: UnitType::Px, value: 0.0 }; 4],
        display: Display::Block
    }
}

impl Style {

    pub(crate) fn to_rust(&self) -> crate::style::Style {
        crate::style::Style {
            color: crate::Color::from_rgba8(self.color.r as u8, self.color.g as u8, self.color.b as u8, self.color.a as u8),
            display: self.display,
            margin: [self.margin[0].to_rust(), self.margin[1].to_rust(), self.margin[2].to_rust(), self.margin[3].to_rust()],
            ..Default::default()
        }
    }

}