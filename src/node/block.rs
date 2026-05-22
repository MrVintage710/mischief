use bevy::ecs::{component::Component, entity::Entity, system::EntityCommands};
use ratatui::{prelude::Buffer, widgets::{Widget, block}};
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace};

use crate::{layout::Rect, node::Node, style::Style};

#[derive(Debug, Component, Default)]
pub struct Block {
    title : Option<String>,
    title_bottom : Option<String>,
}

impl Block {
    pub fn title(mut self, title : &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
    
    pub fn title_bottom(mut self, title : &str) -> Self {
        self.title_bottom = Some(title.to_string());
        self
    }
}

impl Node for Block {
    fn render<'a>(&'a self, area: & ratatui::layout::Rect, buf: &mut Buffer, _style : ratatui::style::Style, mut block : block::Block<'a>) {
        if let Some(title) = &self.title { block = block.title(title.as_str()) }
        if let Some(title) = &self.title_bottom { block = block.title_bottom(title.as_str()) }
        block.render(*area, buf);
    }

    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, _ : &Namespace) -> Option<Entity> {
        if !(&name.local_name == "Block") { return None }
        
        let styles = Style::from_attr(attributes).ok()?;
        
        let title = attributes.iter().find(|attr| &attr.name.local_name == "title").map(|attr| attr.value.clone());
        let title_bottom = attributes.iter().find(|attr| &attr.name.local_name == "title-bottom").map(|attr| attr.value.clone());
        let block = Block { title, title_bottom };
        
        let mut child = Entity::PLACEHOLDER;
        parent.with_children(|parent| {
            child = parent.spawn((
                block,
                styles,
                Rect::default(),
            )).id();
        });
        Some(child)
    }
}