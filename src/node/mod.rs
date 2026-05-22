pub mod block;
pub mod query;
pub mod paragraph;

use std::{collections::VecDeque, marker::PhantomData};

use bevy::{ecs::{relationship::RelationshipSourceCollection, system::SystemParam}, prelude::*};
use ratatui::buffer::Buffer;
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace};

use crate::{Debug, Terminal, layout::calc_layout, node::{block::Block, paragraph::Paragraph, query::{NodeEntity, NodeEntityMut, NodeFindRootsAbility}}, style::{Style, StyleSize}};

//==============================================================================================
//        NodePlugin
//==============================================================================================

pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(NodeRenderPlugin::<Block>::new())
        ;
    }
}

//==============================================================================================
//        NodeRenderPlugin
//==============================================================================================

pub struct NodeRenderPlugin<N : Node>(PhantomData<N>);

impl<N: Node> NodeRenderPlugin<N> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl <N : Node> Plugin for NodeRenderPlugin<N> {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Last, create_render_queue.pipe(render).after(calc_layout)
                .run_if(|debug : Res<Debug>| !debug.0))
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn create_render_queue(
    nodes : Query<NodeEntity>,
) -> VecDeque<Entity> {
    
    fn create_render_queue(node : Entity, nodes : &Query<NodeEntity>, render_queue : &mut VecDeque<Entity>) {
        let Ok(node) = nodes.get(node) else { return };
        render_queue.push_back(node.entity);
        let Some(children) = node.children else { return };
        for child in children.iter() {
            create_render_queue(child, nodes, render_queue);
        }
    }
    
    let roots = nodes.find_roots();
    
    let mut render_queue = VecDeque::new();
    for root in roots.iter() {
        create_render_queue(root, &nodes, &mut render_queue);
    }
    render_queue
}

pub fn render(
    mut render_queue: In<VecDeque<Entity>>,
    mut node_renderer: NodeRenderer
) {
    while let Some(node) = render_queue.pop_front() {
        node_renderer.render(node);
    }
}

//==============================================================================================
//        Node
//==============================================================================================

pub trait Node : Component {
    fn render<'a>(
        &'a self, 
        area: &ratatui::layout::Rect, 
        buf: &mut Buffer, 
        style : &Style,
    );
    
    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, namespace : &Namespace) -> Option<Entity>;

    fn calc_required_space(&self, area : &crate::layout::Rect, _style : &mut Style) { }
}

#[derive(Component)]
pub struct NullComponent;

impl Node for NullComponent {
    fn render<'a>(&'a self, _area: &ratatui::layout::Rect, _buf: &mut Buffer, _style : &Style) {}

    fn parse(_parent : &mut EntityCommands, _name : &OwnedName, _attributes: &Vec<OwnedAttribute>, _namespace : &Namespace) -> Option<Entity> { None }
}

//==============================================================================================
//        NodeRenderer
//==============================================================================================

#[derive(SystemParam)]
pub struct NodeRenderer<'w, 's> {
    terminal : ResMut<'w, Terminal>,
    block_components : Query<'w, 's, (&'static Block, &'static crate::layout::Rect, &'static Style)>,
    paragraph_nodes : Query<'w, 's, (&'static Paragraph, &'static crate::layout::Rect, &'static Style)>
}

impl <'w, 's> NodeRenderer<'w, 's> {
    pub fn render(&mut self, node : Entity) {
        if let Ok((node, rect, style)) = self.block_components.get(node) {
            node.render(&rect.0, self.terminal.current_buffer_mut(), style);
        }

        if let Ok((node, rect, style)) = self.paragraph_nodes.get(node) {
            node.render(&rect.0, self.terminal.current_buffer_mut(), style);
        }
    }
}