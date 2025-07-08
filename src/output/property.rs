use crate::output::{TableFormat, PlainFormat};
use colored::*;

/// A device property key-value pair
#[derive(Debug, Clone, serde::Serialize)]
pub struct Property {
    pub key: String,
    pub value: String,
}

#[allow(dead_code)]
impl Property {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Formatter for device properties
#[allow(dead_code)]
pub struct PropertyFormatter {
    color_enabled: bool,
}

#[allow(dead_code)]
impl PropertyFormatter {
    pub fn new() -> Self {
        Self {
            color_enabled: true,
        }
    }
    
    pub fn with_color(mut self, enabled: bool) -> Self {
        self.color_enabled = enabled;
        self
    }
    
    /// Format properties as colored key-value pairs
    pub fn format_properties(&self, properties: &[Property]) -> Vec<String> {
        properties.iter()
            .map(|prop| self.format_property(prop))
            .collect()
    }
    
    /// Format a single property
    pub fn format_property(&self, prop: &Property) -> String {
        if self.color_enabled {
            format!("{}: {}", prop.key.bright_cyan(), prop.value.bright_white())
        } else {
            format!("{}: {}", prop.key, prop.value)
        }
    }
}

impl TableFormat for Property {
    fn headers() -> Vec<&'static str> {
        vec!["PROPERTY", "VALUE"]
    }
    
    fn row(&self) -> Vec<String> {
        vec![self.key.clone(), self.value.clone()]
    }
}

impl PlainFormat for Property {
    fn plain(&self) -> String {
        format!("[{}]: [{}]", self.key, self.value)
    }
}

/// A collection of properties
#[derive(Debug, Clone, serde::Serialize)]
pub struct PropertyCollection {
    pub properties: Vec<Property>,
}

#[allow(dead_code)]
impl PropertyCollection {
    pub fn new() -> Self {
        Self {
            properties: Vec::new(),
        }
    }
    
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.push(Property::new(key, value));
    }
    
    pub fn from_vec(props: Vec<(String, String)>) -> Self {
        Self {
            properties: props.into_iter()
                .map(|(k, v)| Property::new(k, v))
                .collect(),
        }
    }
}

impl TableFormat for PropertyCollection {
    fn headers() -> Vec<&'static str> {
        Property::headers()
    }
    
    fn row(&self) -> Vec<String> {
        // This doesn't make sense for a collection
        vec![]
    }
}