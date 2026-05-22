use bevy::ecs::{component::Component, entity::Entity, system::EntityCommands};
use ratatui::{text::{Text}, widgets::Widget};
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace};

use crate::{layout::Rect, node::Node, style::{Style, StyleSize, Unit}};

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
        style : &Style
    ) {
        let p = ratatui::widgets::Paragraph::new(self.0.as_str());
        p.style(style.style).block(style.create_block()).render(*area, buf);
    }

    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, namespace : &Namespace) -> Option<Entity> {
        //Special Case that is handled in the main parse method.
        return None;
    }

    fn calc_required_space(&self, rect : &Rect, style : &mut Style) {
        let text = Text::raw(&self.0);
        style.min_size.height = Unit::Px((rect.width as f32 / text.width() as f32) as u16)
    }
}