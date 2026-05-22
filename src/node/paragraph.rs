use bevy::ecs::{component::Component, entity::Entity, system::EntityCommands};
use ratatui::widgets::Widget;
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace};

use crate::node::Node;

//==============================================================================================
//        Paragraph Node
//==============================================================================================

#[derive(Debug, Component)]
pub struct Paragraph(pub String);

impl Node for Paragraph {
    fn render<'a>(
        &'a self, 
        area: &ratatui::layout::Rect, 
        buf: &mut ratatui::prelude::Buffer, 
        style : ratatui::style::Style,
        block : ratatui::widgets::block::Block<'a>
    ) {
        let p = ratatui::widgets::Paragraph::new(self.0.as_str());
        p.style(style).block(block).render(*area, buf);
    }

    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, namespace : &Namespace) -> Option<Entity> {
        //Special Case that is handled in the main parse method.
        return None;
    }
}