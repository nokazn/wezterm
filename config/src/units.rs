use wezterm_dynamic::{FromDynamic, FromDynamicOptions, ToDynamic, Value};

#[derive(Debug, Copy, Clone)]
pub struct PixelUnit(Dimension);

impl Into<Dimension> for PixelUnit {
    fn into(self) -> Dimension {
        self.0
    }
}

impl FromDynamic for PixelUnit {
    fn from_dynamic(
        value: &Value,
        _options: FromDynamicOptions,
    ) -> Result<Self, wezterm_dynamic::Error> {
        Ok(Self(DefaultUnit::Pixels.from_dynamic_impl(value)?))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DefaultUnit {
    Points,
    Pixels,
    Percent,
    Cells,
}

impl DefaultUnit {
    fn to_dimension(self, value: f32) -> Dimension {
        match self {
            Self::Points => Dimension::Points(value),
            Self::Pixels => Dimension::Pixels(value),
            Self::Percent => Dimension::Percent(value / 100.),
            Self::Cells => Dimension::Cells(value),
        }
    }
}

impl DefaultUnit {
    fn from_dynamic_impl(self, value: &Value) -> Result<Dimension, String> {
        match value {
            Value::F64(f) => Ok(self.to_dimension(f.into_inner() as f32)),
            Value::I64(i) => Ok(self.to_dimension(*i as f32)),
            Value::U64(u) => Ok(self.to_dimension(*u as f32)),
            Value::String(s) => {
                if let Ok(value) = s.parse::<f32>() {
                    Ok(self.to_dimension(value))
                } else {
                    fn is_unit(s: &str, unit: &'static str) -> Option<f32> {
                        let s = s.strip_suffix(unit)?.trim();
                        s.parse().ok()
                    }

                    if let Some(v) = is_unit(s, "px") {
                        Ok(DefaultUnit::Pixels.to_dimension(v))
                    } else if let Some(v) = is_unit(s, "%") {
                        Ok(DefaultUnit::Percent.to_dimension(v))
                    } else if let Some(v) = is_unit(s, "pt") {
                        Ok(DefaultUnit::Points.to_dimension(v))
                    } else if let Some(v) = is_unit(s, "cell") {
                        Ok(DefaultUnit::Cells.to_dimension(v))
                    } else {
                        Err(format!(
                            "expected either a number or a string of \
                        the form '123px' where 'px' is a unit and \
                        can be one of 'px', '%', 'pt' or 'cell', \
                        but got {}",
                            s
                        ))
                    }
                }
            }
            other => Err(format!(
                "expected either a number or a string of \
                        the form '123px' where 'px' is a unit and \
                        can be one of 'px', '%', 'pt' or 'cell', \
                        but got {}",
                other.variant_name()
            )),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Dimension {
    /// A value expressed in points, where 72 points == 1 inch.
    Points(f32),

    /// A value expressed in raw pixels
    Pixels(f32),

    /// A value expressed in terms of a fraction of the maximum
    /// value in the same direction.  For example, left padding
    /// of 10% depends on the pixel width of that element.
    /// The value is 1.0 == 100%.  It is possible to express
    /// eg: 2.0 for 200%.
    Percent(f32),

    /// A value expressed in terms of a fraction of the cell
    /// size computed from the configured font size.
    /// 1.0 == the cell size.
    Cells(f32),
}

impl Dimension {
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Points(n) | Self::Pixels(n) | Self::Percent(n) | Self::Cells(n) => *n == 0.,
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Pixels(0.)
    }
}

impl ToDynamic for Dimension {
    fn to_dynamic(&self) -> Value {
        let s = match self {
            Self::Points(n) => format!("{}pt", n),
            Self::Pixels(n) => format!("{}px", n),
            Self::Percent(n) => format!("{}%", n * 100.),
            Self::Cells(n) => format!("{}cell", n),
        };
        Value::String(s)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DimensionContext {
    pub dpi: f32,
    /// Width/Height or other upper bound on the dimension,
    /// measured in pixels.
    pub pixel_max: f32,
    /// Width/Height of the font metrics cell size in the appropriate
    /// dimension, measured in pixels.
    pub pixel_cell: f32,
}

impl Dimension {
    pub fn evaluate_as_pixels(&self, context: DimensionContext) -> f32 {
        match self {
            Self::Pixels(n) => n.floor(),
            Self::Points(pt) => (pt * context.dpi / 72.0).floor(),
            Self::Percent(p) => (p * context.pixel_max).floor(),
            Self::Cells(c) => (c * context.pixel_cell).floor(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryOrigin {
    /// x,y relative to overall screen coordinate system.
    /// Selected position might be outside of the regions covered
    /// by the user's selected monitor placement.
    ScreenCoordinateSystem,
    MainScreen,
    ActiveScreen,
    Named(String),
}

impl Default for GeometryOrigin {
    fn default() -> Self {
        Self::ScreenCoordinateSystem
    }
}
