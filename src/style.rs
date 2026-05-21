use std::str::FromStr;

use bevy::prelude::*;
use ratatui::widgets::{BorderType, Borders};
use regex::Regex;
use taffy::{Dimension, LengthPercentage, LengthPercentageAuto, style_helpers::{FromLength, FromPercent, TaffyAuto, TaffyZero}};
use tailwind_ast::AstStyle;
use xml::attribute::OwnedAttribute;

use crate::error::{MischiefError, MischiefResult};

//==============================================================================================
//        Style
//==============================================================================================

#[derive(Debug, Component, Clone, Copy)]
pub struct Style{
    pub style : ratatui::style::Style,
    pub padding : StyleRect,
    pub display : taffy::Display,
    pub flex_direction : taffy::FlexDirection,
    pub gap : StyleSize,
    pub size : StyleSize,
    pub min_size : StyleSize,
    pub max_size : StyleSize,
    pub justify_content : Option<taffy::JustifyContent>,
    pub align_content : Option<taffy::AlignContent>,
    pub borders : Borders,
    pub border_type : BorderType
}

impl Style {
    
    pub fn from_attr(attributes: &Vec<OwnedAttribute>) -> MischiefResult<Self> {
        let mut styles = Style::default();

        let Some(style_attr) = attributes.iter().find(|attr| &attr.name.local_name == "style") else { return Err(MischiefError::NoValue)};
        
        let Ok(tokens) = tailwind_ast::parse_tailwind(&style_attr.value) else { return Err(MischiefError::TailwindParsingError(style_attr.value.to_string()))};

        for token in tokens.iter() {
            Self::set_flex_styles(&mut styles, token);
            //Width and Height Setters
            Self::set_dimention::<'w'>(&mut styles.size.width, &mut styles.min_size.width, &mut styles.max_size.width, token);
            Self::set_dimention::<'h'>(&mut styles.size.height, &mut styles.min_size.height, &mut styles.max_size.height, token);
            //Padding
            Self::set_style_rect("p", &mut styles.padding, token);
            //Borders
            Self::set_borders(&mut styles.borders, &mut styles.border_type, token);
        }
        
        Ok(styles)
    }

    fn set_flex_styles(styles : &mut Style, token : &AstStyle) {
        if !token.elements.starts_with(&["flex"]) { return }
        
        if token.elements.eq(&["flex"]) {
            styles.display = taffy::Display::Flex
        }
        
        if token.elements.eq(&["flex", "col"]) {
            styles.flex_direction = taffy::FlexDirection::Column
        }

        if token.elements.eq(&["flex", "col", "reverse"]) {
            styles.flex_direction = taffy::FlexDirection::ColumnReverse
        }

        if token.elements.eq(&["flex", "row"]) {
            styles.flex_direction = taffy::FlexDirection::Row
        }

        if token.elements.eq(&["flex", "row", "reverse"]) {
            styles.flex_direction = taffy::FlexDirection::RowReverse
        }
    }

    fn set_dimention<const DIM : char>(value : &mut Unit, min : &mut Unit, max : &mut Unit, token : &AstStyle) {
        let mut buffer = [0; 4]; // A char can be up to 4 bytes
        let s: &str = DIM.encode_utf8(&mut buffer);
        if !token.elements.starts_with(&[s]) { return }

        if let Some(next) = token.elements.get(1) {
            let value = match *next {
                "max" => max,
                "min" => min,
                _ => value
            };

            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            *value = unit;
        }
    }

    fn set_borders(borders : &mut Borders, border_type : &mut BorderType, token : &AstStyle) {

        if token.elements.eq(&["border"]) {
            *borders = Borders::all();
        }

        if token.elements.eq(&["border"]) {
            *borders = Borders::all();
        }
    }

    fn set_style_rect(prefix : &str, value : &mut StyleRect, token : &AstStyle) {
        if token.elements.starts_with(&[&format!("{prefix}l")]) {
            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            value.left = unit;
        }

        if token.elements.starts_with(&[&format!("{prefix}r")]) {
            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            value.right = unit;
        }

        if token.elements.starts_with(&[&format!("{prefix}t")]) {
            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            value.top = unit;
        }

        if token.elements.starts_with(&[&format!("{prefix}b")]) {
            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            value.bottom = unit;
        }

        if token.elements.starts_with(&[&format!("{prefix}x")]) {
            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            value.left = unit;
            value.right = unit;
        }

        if token.elements.starts_with(&[&format!("{prefix}y")]) {
            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            value.top = unit;
            value.bottom = unit;
        }

        if token.elements.starts_with(&[&format!("{prefix}")]) {
            let Ok(unit) = Self::parse_end_number_or_arbitrary(token) else { return };
            value.top = unit;
            value.bottom = unit;
            value.left = unit;
            value.right = unit;
        }
    }

    fn parse_end_number_or_arbitrary(token : &AstStyle) -> MischiefResult<Unit> {
        if let Some(last) = token.elements.last() && let Ok(unit) = last.parse() {
            return Ok(unit)
        }
        
        if let Some(arbitrary) = token.arbitrary && let Ok(unit) = arbitrary.parse() {
            return Ok(unit);
        };

        Err(MischiefError::TailwindParsingError(format!("Unable to parse a value from: {}", token.elements.join("-"))))
    }
}

impl Default for Style {
    fn default() -> Self {
        Self { 
            style: Default::default(), 
            padding: StyleRect::ZERO, 
            display: taffy::Display::Block, 
            flex_direction: Default::default(), 
            gap: StyleSize::ZERO, 
            size: StyleSize::FULL, 
            min_size: StyleSize::AUTO, 
            max_size: StyleSize::AUTO, 
            justify_content: Default::default(), 
            align_content: Default::default(),
            borders: Borders::empty(),
            border_type: BorderType::Plain
        }
    }
}

impl Into<taffy::Style> for Style {
    fn into(self) -> taffy::Style {
        let border = if self.borders.is_empty() {taffy::Rect::zero()} else {taffy::Rect::length(1.0)};
        taffy::Style {
            border,
            size : self.size.into(),
            min_size : self.min_size.into(),
            max_size : self.max_size.into(),
            gap: self.gap.into(),
            display: self.display,
            flex_direction: self.flex_direction,
            padding : self.padding.into(),
            justify_content : self.justify_content,
            align_content : self.align_content,
            position: taffy::Position::Relative,
            flex_basis: Dimension::AUTO,
            ..Default::default()
        }
    }
}



impl Style {
    // pub fn fg(self, color : Color) -> Self { 
    //     Self(self.0.fg(color))
    // }
    
    // pub fn bg(self, color : Color) -> Self { 
    //     Self(self.0.bg(color))
    // }
    
    // pub fn underline_color(self, color : Color) -> Self { 
    //     Self(self.0.underline_color(color))
    // }
    
    // pub fn patch(self, style : Style) -> Self { 
    //     Self(self.0.patch(style.0))
    // }
    
    // pub fn add_modifier(self, modifier : Modifier) -> Self { 
    //     Self(self.0.add_modifier(modifier))
    // }
    
    // pub fn remove_modifier(self, modifier : Modifier) -> Self { 
    //     Self(self.0.remove_modifier(modifier))
    // }
    
    // pub fn set_style(&mut self, style : ratatui::style::Style) {
    //     self.0 = style;
    // }
    
    // pub fn set_fg(&mut self, color : Color) {
    //     self.0 = self.0.fg(color)
    // }
}

//==============================================================================================
//        Unit
//==============================================================================================

#[derive(Debug, Default, Clone, Copy)]
pub enum Unit {
    Px(u16),
    Percent(f32),
    #[default]
    Auto
}

impl Unit {
    pub const FULL : Self = Self::Percent(1.0);
}

impl TaffyZero for Unit {
    const ZERO: Self = Unit::Px(0);
}

impl TaffyAuto for Unit {
    const AUTO: Self = Unit::Auto;
}

impl FromLength for Unit {
    fn from_length<Input: Into<f32> + Copy>(value: Input) -> Self {
        Unit::Px(value.into() as u16)
    }
}

impl FromPercent for Unit {
    fn from_percent<Input: Into<f32> + Copy>(percent: Input) -> Self {
        Unit::Percent(percent.into())
    }
}

impl FromStr for Unit {
    type Err = MischiefError;

    fn from_str(s: &str) -> MischiefResult<Self> {
        let regex = Regex::new(
            r"^(?<number>\d+(\.\d+)?)(?<unit>[[:alpha:]]+)?$"
        )?;
        if s == "auto" { return Ok(Self::Auto) }
        if s == "full" { return Ok(Self::Percent(1.0)) }
        let test = regex.captures(s).ok_or(MischiefError::UnitParseError("No number matched.".to_string()))?;
        let number = test.name("number").ok_or(MischiefError::UnitParseError("No number matched.".to_string()))?;
        let number : f32 = number.as_str().parse()?;
        let unit = test.name("unit").map(|m| m.as_str()).unwrap_or("px");
        match unit {
            "px" => Ok(Self::Px(number as u16)),
            "%" => Ok(Self::Percent(number)),
            _ => Err(MischiefError::UnitParseError(format!("Did not recognize unit: {}", unit)))
        }
    }
}

impl Into<LengthPercentage> for Unit {
    fn into(self) -> LengthPercentage {
        match self {
            Unit::Px(length) => LengthPercentage::length(length as f32),
            Unit::Percent(percent) => LengthPercentage::percent(percent),
            Unit::Auto => LengthPercentage::percent(1.0),
        }
    }
}

impl Into<LengthPercentageAuto> for Unit {
    fn into(self) -> LengthPercentageAuto {
        match self {
            Unit::Px(length) => LengthPercentageAuto::length(length as f32),
            Unit::Percent(percent) => LengthPercentageAuto::percent(percent),
            Unit::Auto => LengthPercentageAuto::auto(),
        }
    }
}

impl Into<taffy::Dimension> for Unit {
    fn into(self) -> taffy::Dimension {
        match self {
            Unit::Px(length) => taffy::Dimension::length(length as f32),
            Unit::Percent(percent) => taffy::Dimension::percent(percent),
            Unit::Auto => taffy::Dimension::auto(),
        }
    }
}

//==============================================================================================
//        StyleRect
//==============================================================================================

#[derive(Debug, Default, Clone, Copy)]
pub struct StyleRect {
    top : Unit,
    bottom : Unit,
    right : Unit,
    left : Unit
}

impl TaffyZero for StyleRect {
    const ZERO: Self = StyleRect { top: Unit::ZERO, bottom: Unit::ZERO, right: Unit::ZERO, left: Unit::ZERO };
}

impl TaffyAuto for StyleRect {
    const AUTO: Self = StyleRect { top: Unit::AUTO, bottom: Unit::AUTO, right: Unit::AUTO, left: Unit::AUTO };
}

impl Into<taffy::Rect<LengthPercentageAuto>> for StyleRect {
    fn into(self) -> taffy::Rect<LengthPercentageAuto> {
        taffy::Rect { left: self.left.into(), right: self.right.into(), top: self.top.into(), bottom: self.bottom.into() }
    }
}

impl Into<taffy::Rect<LengthPercentage>> for StyleRect {
    fn into(self) -> taffy::Rect<LengthPercentage> {
        taffy::Rect { left: self.left.into(), right: self.right.into(), top: self.top.into(), bottom: self.bottom.into() }
    }
}

//==============================================================================================
//        StyleSize
//==============================================================================================

#[derive(Debug, Default, Clone, Copy)]
pub struct StyleSize {
    width : Unit,
    height : Unit
}

impl StyleSize {
    pub const FULL: Self = StyleSize { width: Unit::FULL, height: Unit::FULL };
}

impl TaffyZero for StyleSize {
    const ZERO: Self = StyleSize {  width: Unit::ZERO, height: Unit::ZERO };
}

impl TaffyAuto for StyleSize {
    const AUTO: Self = StyleSize { width: Unit::AUTO, height: Unit::AUTO };
}

impl Into<taffy::Size<LengthPercentageAuto>> for StyleSize {
    fn into(self) -> taffy::Size<LengthPercentageAuto> {
        taffy::Size {
            width: self.width.into(),
            height: self.height.into(),
        }
    }
}

impl Into<taffy::Size<LengthPercentage>> for StyleSize {
    fn into(self) -> taffy::Size<LengthPercentage> {
        taffy::Size {
            width: self.width.into(),
            height: self.height.into(),
        }
    }
}

impl Into<taffy::Size<taffy::Dimension>> for StyleSize {
    fn into(self) -> taffy::Size<taffy::Dimension> {
        taffy::Size {
            width: self.width.into(),
            height: self.height.into(),
        }
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
    pub border_type : BorderType
}

//==============================================================================================
//        Get Styles from attributes
//==============================================================================================

// #[derive(Debug, Default)]
// pub struct StyleSummary {
//     pub layout : Layout,
//     pub padding : Padding,
//     pub style : Style,
//     pub rect : crate::layout::Rect,
//     pub border_style : BorderStyle
// }

// pub fn get_style_components_from_attributes(attributes : &Vec<OwnedAttribute>) -> Option<StyleSummary> {
//     let style_attr = attributes.iter().find(|attr| &attr.name.local_name == "style")?;
    
//     let tokens = tailwind_ast::parse_tailwind(&style_attr.value)
//         .inspect_err(|e| eprintln!("There was an error while parsing tailwind: {e:?}"))
//         .ok()?;
    
//     let mut style_summary = StyleSummary::default();
    
//     for token in tokens.iter() {
//         get_border_styles(token, &mut style_summary.border_style);
//         get_padding_styles(token, &mut style_summary.padding);
//         get_react_styles(token, &mut style_summary.rect);
//     }
    
//     Some(style_summary)
// }

// fn get_border_styles(token : &AstStyle, border_style : &mut BorderStyle) {
//     if !token.elements.starts_with(&["border"]) { return }
    
//     if let Some(value) = token.elements.get(1) && *value == "t" {border_style.borders |= Borders::TOP}
//     if let Some(value) = token.elements.get(1) && *value == "b" {border_style.borders |= Borders::BOTTOM}
//     if let Some(value) = token.elements.get(1) && *value == "l" {border_style.borders |= Borders::LEFT}
//     if let Some(value) = token.elements.get(1) && *value == "r" {border_style.borders |= Borders::RIGHT}
//     if let Some(value) = token.elements.get(1) && *value == "y" {border_style.borders |= Borders::TOP | Borders::BOTTOM }
//     if let Some(value) = token.elements.get(1) && *value == "x" {border_style.borders |= Borders::RIGHT | Borders::LEFT }
    
//     if let None = token.elements.get(1) { border_style.borders = Borders::all() }
    
//     if let Some(value) = token.elements.get(1) && *value == "plain" {border_style.border_type = BorderType::Plain}
//     if let Some(value) = token.elements.get(1) && *value == "double" {border_style.border_type = BorderType::Double}
//     if let Some(value) = token.elements.get(1) && *value == "qi" {border_style.border_type = BorderType::QuadrantInside}
//     if let Some(value) = token.elements.get(1) && *value == "qo" {border_style.border_type = BorderType::QuadrantOutside}
//     if let Some(value) = token.elements.get(1) && *value == "rounded" {border_style.border_type = BorderType::Rounded}
//     if let Some(value) = token.elements.get(1) && *value == "thick" {border_style.border_type = BorderType::Thick}
// }

// fn get_padding_styles(token : &AstStyle, padding : &mut Padding) {
//     if token.elements.starts_with(&["p"]) {
//         if let Some(value) = token.elements.get(1) {
//             let Ok(value) = value.parse::<u16>() else { return };
//             padding.bottom = value.into();
//             padding.top = value.into();
//             padding.right = value.into();
//             padding.left = value.into();
//         }
//     }
    
//     if token.elements.starts_with(&["pt"]) {
//         if let Some(value) = token.elements.get(1) {
//             let Ok(value) = value.parse::<u16>() else { return };
//             padding.top = value.into();
//         }
//     }
    
//     if token.elements.starts_with(&["pb"]) {
//         if let Some(value) = token.elements.get(1) {
//             let Ok(value) = value.parse::<u16>() else { return };
//             padding.bottom = value.into();
//         }
//     }
    
//     if token.elements.starts_with(&["py"]) {
//         if let Some(value) = token.elements.get(1) {
//             let Ok(value) = value.parse::<u16>() else { return };
//             padding.bottom = value.into();
//             padding.top = value.into();
//         }
//     }
    
//     if token.elements.starts_with(&["pl"]) {
//         if let Some(value) = token.elements.get(1) {
//             let Ok(value) = value.parse::<u16>() else { return };
//             padding.left = value.into();
//         }
//     }
    
//     if token.elements.starts_with(&["pr"]) {
//         if let Some(value) = token.elements.get(1) {
//             let Ok(value) = value.parse::<u16>() else { return };
//             padding.right = value.into();
//         }
//     }
    
//     if token.elements.starts_with(&["px"]) {
//         if let Some(value) = token.elements.get(1) {
//             let Ok(value) = value.parse::<u16>() else { return };
//             padding.left = value.into();
//             padding.right = value.into();
//         }
//     }
// }

// fn get_react_styles(token : &AstStyle, rect : &mut crate::layout::Rect) {
//     if token.elements.starts_with(&["x"]) && let Some(value) = token.elements.get(1) {
//         let Ok(value) = value.parse::<u16>() else { return };
//         rect.x = Value::Px(value)
//     }
    
//     if token.elements.starts_with(&["y"]) && let Some(value) = token.elements.get(1) {
//         let Ok(value) = value.parse::<u16>() else { return };
//         rect.y = Value::Px(value)
//     }
    
//     if token.elements.starts_with(&["w"]) && let Some(value) = token.elements.get(1) {
//         let Ok(value) = value.parse::<u16>() else { return };
//         rect.width = Value::Px(value)
//     }
    
//     if token.elements.starts_with(&["h"]) && let Some(value) = token.elements.get(1) {
//         let Ok(value) = value.parse::<u16>() else { return };
//         rect.height = Value::Px(value)
//     }
// }