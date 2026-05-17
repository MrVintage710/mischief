use bevy::prelude::*;
use ratatui::{style::{Color, Modifier}, widgets::Borders};

//==============================================================================================
//        Style
//==============================================================================================

#[derive(Debug, Component, Default, Clone)]
pub struct Style(pub ratatui::style::Style);

impl Style {
    pub fn fg(self, color : Color) -> Self { 
        Self(self.0.fg(color))
    }
    
    pub fn bg(self, color : Color) -> Self { 
        Self(self.0.bg(color))
    }
    
    pub fn underline_color(self, color : Color) -> Self { 
        Self(self.0.underline_color(color))
    }
    
    pub fn patch(self, style : Style) -> Self { 
        Self(self.0.patch(style.0))
    }
    
    pub fn add_modifier(self, modifier : Modifier) -> Self { 
        Self(self.0.add_modifier(modifier))
    }
    
    pub fn remove_modifier(self, modifier : Modifier) -> Self { 
        Self(self.0.remove_modifier(modifier))
    }
}

//==============================================================================================
//        BorderStyle
//==============================================================================================

pub struct BorderStyle {
    borders : Borders,
    coloring : ratatui::style::Style,
    border_style : ratatui::widgets::BorderType
}

