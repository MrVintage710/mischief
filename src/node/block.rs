use bevy::ecs::{component::Component, entity::Entity, system::EntityCommands};
use ratatui::{layout::Rect, prelude::Buffer, style::Style, widgets::{BorderType, Borders, Widget}};
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace};

use crate::{node::Node, style::get_style_components_from_attributes};

#[derive(Debug, Component, Default)]
pub struct Block {
    title : Option<String>,
    borders : Option<Borders>,
    border_type : BorderType
}

impl Block {
    pub fn title(mut self, title : &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    
    pub fn borders(mut self, borders : Borders) -> Self {
        self.borders = Some(borders);
        self
    }
    
    pub fn border_type(mut self, border_type : BorderType) -> Self {
        self.border_type = border_type;
        self
    }
}

impl Node for Block {
    fn render(&self, area: &Rect, buf: &mut Buffer, style : Style) {
        let mut block = ratatui::widgets::Block::new().style(style);
        
        if let Some(title) = &self.title { block = block.title(title.as_str()) }
        if let Some(borders) = &self.borders { block = block.borders(*borders) }
        block = block.title_bottom(format!("{area:?}"));
        block = block.border_type(self.border_type);
        block.render(*area, buf);
    }

    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, _ : &Namespace) -> Option<Entity> {
        if !(&name.local_name == "Block") { return None }
        
        let style_summary = get_style_components_from_attributes(attributes).unwrap_or_default();
        
        let title = attributes.iter().find(|attr| &attr.name.local_name == "title").map(|attr| attr.value.clone());
        let block = Block { title, ..Default::default() }
            .borders(style_summary.border_style.borders)
            .border_type(style_summary.border_style.border_type)
        ;
        
        let mut child = Entity::PLACEHOLDER;
        parent.with_children(|parent| {
            child = parent.spawn((
                block,
                style_summary.layout,
                style_summary.padding,
                style_summary.style,
                style_summary.rect
            )).id();
        });
        Some(child)
    }
    
}