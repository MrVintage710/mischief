use bevy::prelude::*;
use ratatui::{style::{Color, Modifier}, widgets::Borders};
use tailwind_ast::AstStyle;
use xml::attribute::OwnedAttribute;

use crate::layout::{Layout, Padding};

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

#[derive(Debug, Default)]
pub struct BorderStyle {
    pub title : Option<String>,
    pub borders : Borders,
    pub coloring : ratatui::style::Style,
    pub border_type : ratatui::widgets::BorderType
}

//==============================================================================================
//        Get Styles from attributes
//==============================================================================================

#[derive(Debug, Default)]
pub struct StyleSummary {
    pub layout : Layout,
    pub padding : Padding,
    pub style : Style,
    pub rect : crate::layout::Rect,
    pub border_style : BorderStyle
}

pub fn get_style_components_from_attributes(attributes : &Vec<OwnedAttribute>) -> Option<StyleSummary> {
    let style_attr = attributes.iter().find(|attr| &attr.name.local_name == "style")?;
    
    let tokens = tailwind_ast::parse_tailwind(&style_attr.value)
        .inspect_err(|e| eprintln!("There was an error while parsing tailwind: {e:?}"))
        .ok()?;
    
    let mut style_summary = StyleSummary::default();
    
    for token in tokens.iter() {
        get_border_styles(token, &mut style_summary.border_style);
        get_padding_styles(token, &mut style_summary.padding);
    }
    
    Some(style_summary)
}

fn get_border_styles(token : &AstStyle, border_style : &mut BorderStyle) {
    if !token.elements.starts_with(&["border"]) { return }
    
    if let Some(value) = token.elements.get(1) && *value == "t" {border_style.borders |= Borders::TOP}
    if let Some(value) = token.elements.get(1) && *value == "b" {border_style.borders |= Borders::BOTTOM}
    if let Some(value) = token.elements.get(1) && *value == "l" {border_style.borders |= Borders::LEFT}
    if let Some(value) = token.elements.get(1) && *value == "r" {border_style.borders |= Borders::RIGHT}
    if let Some(value) = token.elements.get(1) && *value == "y" {border_style.borders |= Borders::TOP | Borders::BOTTOM }
    if let Some(value) = token.elements.get(1) && *value == "x" {border_style.borders |= Borders::RIGHT | Borders::LEFT }
    
    if let None = token.elements.get(1) { border_style.borders = Borders::all() }
    
    if let Some(value) = token.elements.get(1) && *value == "plain" {border_style.border_type = ratatui::widgets::BorderType::Plain}
    if let Some(value) = token.elements.get(1) && *value == "double" {border_style.border_type = ratatui::widgets::BorderType::Double}
    if let Some(value) = token.elements.get(1) && *value == "qi" {border_style.border_type = ratatui::widgets::BorderType::QuadrantInside}
    if let Some(value) = token.elements.get(1) && *value == "qo" {border_style.border_type = ratatui::widgets::BorderType::QuadrantOutside}
    if let Some(value) = token.elements.get(1) && *value == "rounded" {border_style.border_type = ratatui::widgets::BorderType::Rounded}
    if let Some(value) = token.elements.get(1) && *value == "thick" {border_style.border_type = ratatui::widgets::BorderType::Thick}
}

fn get_padding_styles(token : &AstStyle, padding : &mut Padding) {
    if token.elements.starts_with(&["p"]) {
        if let Some(value) = token.elements.get(1) {
            let Ok(value) = value.parse::<u16>() else { return };
            padding.bottom = value;
            padding.top = value;
            padding.right = value;
            padding.left = value;
        }
    }
    
}