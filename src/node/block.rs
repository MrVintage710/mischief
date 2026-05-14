use bevy::{ecs::{component::Component, entity::Entity, system::EntityCommands}, reflect::attributes};
use ratatui::{layout::Rect, prelude::Buffer, style::Style, widgets::{Borders, Widget}};
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace, reader::XmlEvent};

use crate::{component::get_style_components_from_attributes, node::Node};

#[derive(Debug, Component, Default)]
pub struct Block {
    title : Option<String>,
    borders : Option<Borders>
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
}

impl Node for Block {
    fn render(&self, area: &Rect, buf: &mut Buffer, style : Style) {
        let mut block = ratatui::widgets::Block::new().style(style);
        
        if let Some(title) = &self.title { block = block.title(title.as_str()) }
        if let Some(borders) = &self.borders { block = block.borders(*borders) }
        block.render(*area, buf);
    }

    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, _ : &Namespace) -> Option<Entity> {
        if !(&name.local_name == "Block") { return None }
        
        let title = attributes.iter().find(|attr| &attr.name.local_name == "title").map(|value| value.value.clone());
        let block = Block {
            title,
            ..Default::default()
        };
        
        let (layout, padding, style) = get_style_components_from_attributes(attributes).unwrap_or_default();
        let mut child = Entity::PLACEHOLDER;
        parent.with_children(|parent| {
            child = parent.spawn((
                block,
                layout,
                padding,
                style
            )).id();
        });
        Some(child)
    }
    
}