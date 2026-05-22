use std::{str::FromStr};

use bevy::prelude::*;
use ratatui::{style::Color, symbols::border, widgets::{Block, BorderType, Borders}};
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
    pub border_type : BorderType,
    pub border_size : StyleRect,
    pub bg_color : Color,
    pub fg_color : Color,
    pub border_bg_color : Color,
    pub border_fg_color : Color
}

impl Style {
    pub fn from_attr(attributes: &Vec<OwnedAttribute>) -> MischiefResult<Self> {
        let mut styles = Style::default();

        if let Some(style_attr) = attributes.iter().find(|attr| &attr.name.local_name == "style") {
            let Ok(tokens) = tailwind_ast::parse_tailwind(&style_attr.value) else { return Err(MischiefError::TailwindParsingError(style_attr.value.to_string()))};
    
            for token in tokens.iter() {
                set_flex_styles(&mut styles, token);
                //Width and Height Setters
                set_dimention::<'w'>(&mut styles.size.width, &mut styles.min_size.width, &mut styles.max_size.width, token);
                set_dimention::<'h'>(&mut styles.size.height, &mut styles.min_size.height, &mut styles.max_size.height, token);
                //Padding
                set_style_rect("p", &mut styles.padding, token);
                //Borders
                set_borders(&mut styles.borders, &mut styles.border_type, &mut styles.border_bg_color, &mut styles.border_fg_color, token);
            }
        };
        
        Ok(styles)
    }

    pub fn create_block<'a>(&'a self) -> Block<'a> {
        Block::new()
            .borders(self.borders)
            .border_type(self.border_type)
            .border_style(ratatui::prelude::Style::new()
                .bg(self.border_bg_color)
                .fg(self.border_fg_color)
            )
    }

    pub fn minimum_space() -> Self {
        return Style {
            size : StyleSize::AUTO,
            ..Default::default()
        }
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
            border_type: BorderType::Plain,
            border_size: StyleRect::ZERO,
            bg_color: Color::Black,
            fg_color: Color::White,
            border_bg_color : Color::Black,
            border_fg_color : Color::White
        }
    }
}

impl Into<taffy::Style> for Style {
    fn into(mut self) -> taffy::Style {
        // if self.title.is_some() { self.border_size.top.set_px_if_over(1) }
        // if self.bottom_title.is_some() { self.border_size.bottom.set_px_if_over(1) }
        if self.borders.contains(Borders::TOP) { self.border_size.top.set_px_if_over(1) }
        if self.borders.contains(Borders::BOTTOM) { self.border_size.bottom.set_px_if_over(1) }
        if self.borders.contains(Borders::LEFT) { self.border_size.left.set_px_if_over(1) }
        if self.borders.contains(Borders::RIGHT) { self.border_size.right.set_px_if_over(1) }
        taffy::Style {
            border : self.border_size.into(),
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

    pub fn set_px_if_over(&mut self, value : u16) {
        let Self::Px(inner) = self else { return };
        if *inner < value { *inner = value};
    }
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
//        Helper Functions
//==============================================================================================

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

        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
        *value = unit;
    }
}

fn set_borders(
    borders : &mut Borders, 
    border_type : &mut BorderType, 
    border_bg_color : &mut Color, 
    border_fg_color : &mut Color, 
    token : &AstStyle
) {
    if token.elements.eq(&["border"]) {
        *borders = Borders::all();
    }

    if token.elements.starts_with(&["border"]) {
        let Some(extra) = token.elements.get(1) else { return };
        match *extra {
            "t" => *borders |= Borders::TOP,
            "b" => *borders |= Borders::BOTTOM,
            "l" => *borders |= Borders::LEFT,
            "r" => *borders |= Borders::RIGHT,
            "x" => *borders |= Borders::LEFT | Borders::RIGHT,
            "y" => *borders |= Borders::TOP | Borders::BOTTOM,
            "plain" => *border_type = BorderType::Plain,
            "double" => *border_type = BorderType::Double,
            "rounded" => *border_type = BorderType::Rounded,
            "thick" => *border_type = BorderType::Thick,
            "bg" => if let Ok(color) = parse_end_color(token) { *border_bg_color = color}
            "fg" => if let Ok(color) = parse_end_color(token) { *border_fg_color = color}
            _ => {}
        }
    }
}

fn set_style_rect(prefix : &str, value : &mut StyleRect, token : &AstStyle) {
    if token.elements.starts_with(&[&format!("{prefix}l")]) {
        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
        value.left = unit;
    }

    if token.elements.starts_with(&[&format!("{prefix}r")]) {
        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
        value.right = unit;
    }

    if token.elements.starts_with(&[&format!("{prefix}t")]) {
        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
        value.top = unit;
    }

    if token.elements.starts_with(&[&format!("{prefix}b")]) {
        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
        value.bottom = unit;
    }

    if token.elements.starts_with(&[&format!("{prefix}x")]) {
        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
        value.left = unit;
        value.right = unit;
    }

    if token.elements.starts_with(&[&format!("{prefix}y")]) {
        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
        value.top = unit;
        value.bottom = unit;
    }

    if token.elements.starts_with(&[&format!("{prefix}")]) {
        let Ok(unit) = parse_end_number_or_arbitrary(token) else { return };
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

fn parse_end_color(token : &AstStyle) -> MischiefResult<Color> {
    
    if token.elements.ends_with(&["black"]) {return Ok(Color::Black) }
    if token.elements.ends_with(&["red"]) {return Ok(Color::Red) }
    if token.elements.ends_with(&["green"]) {return Ok(Color::Green) }
    if token.elements.ends_with(&["yellow"]) {return Ok(Color::Yellow) }
    if token.elements.ends_with(&["blue"]) {return Ok(Color::Blue) }
    if token.elements.ends_with(&["magenta"]) {return Ok(Color::Magenta) }
    if token.elements.ends_with(&["cyan"]) {return Ok(Color::Cyan) }
    if token.elements.ends_with(&["white"]) {return Ok(Color::White) }
    if token.elements.ends_with(&["gray"]) {return Ok(Color::Gray) }
    if token.elements.ends_with(&["light", "red"]) {return Ok(Color::LightRed) }
    if token.elements.ends_with(&["light", "green"]) {return Ok(Color::LightGreen) }
    if token.elements.ends_with(&["light", "yellow"]) {return Ok(Color::LightYellow) }
    if token.elements.ends_with(&["light", "blue"]) {return Ok(Color::LightBlue) }
    if token.elements.ends_with(&["light", "magenta"]) {return Ok(Color::LightMagenta) }
    if token.elements.ends_with(&["light", "cyan"]) {return Ok(Color::LightCyan) }
    if token.elements.ends_with(&["dark", "gray"]) {return Ok(Color::DarkGray) }

    Err(MischiefError::TailwindParsingError(format!("Unable to parse a color from: {}", token.elements.join("-"))))
}