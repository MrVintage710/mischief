use bevy::ecs::{component::Component, entity::Entity, system::EntityCommands};
use ratatui::{prelude::Buffer, widgets::{BorderType, Borders, Widget}};
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace};

use crate::{layout::Rect, node::Node, style::Style};

#[derive(Debug, Component, Default)]
pub struct Block {
    title : Option<String>,
    title_bottom : Option<String>,
    borders : Option<Borders>,
    border_type : BorderType
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
    fn render(&self, area: & ratatui::layout::Rect, buf: &mut Buffer, style : ratatui::style::Style) {
        let mut block = ratatui::widgets::Block::new().border_style(style).title_style(style);
        
        if let Some(title) = &self.title { block = block.title(title.as_str()) }
        if let Some(title) = &self.title_bottom { block = block.title_bottom(title.as_str()) }
        if let Some(borders) = &self.borders { block = block.borders(*borders) }
        block = block
            .border_type(self.border_type)
            .title_bottom(format!("{:?}", style.fg))
            .title_bottom(format!("({} {} {} {})", area.x, area.y, area.width, area.height))
        ;
        block.render(*area, buf);
    }

    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, _ : &Namespace) -> Option<Entity> {
        if !(&name.local_name == "Block") { return None }
        
        let styles = Style::from_attr(attributes).ok()?;
        
        let title = attributes.iter().find(|attr| &attr.name.local_name == "title").map(|attr| attr.value.clone());
        let title_bottom = attributes.iter().find(|attr| &attr.name.local_name == "title-bottom").map(|attr| attr.value.clone());
        let block = Block { title, title_bottom, ..Default::default() }
            .borders(styles.borders)
            .border_type(styles.border_type)
        ;
        
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