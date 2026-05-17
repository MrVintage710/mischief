pub mod block;

use std::{collections::{HashSet, VecDeque}, marker::PhantomData};

use bevy::{ecs::{relationship::RelationshipSourceCollection, system::SystemParam}, prelude::*};
use ratatui::buffer::Buffer;
use xml::{attribute::OwnedAttribute, name::OwnedName, namespace::Namespace, reader::XmlEvent};

use crate::{GeneralNodeQuery, NodeQuery, NodeQueryMut, Terminal, find_roots, layout::{GlobalRect, calc_rects}, node::block::Block, style::Style};

//==============================================================================================
//        NodePlugin
//==============================================================================================

pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(NodeRenderPlugin::<Block>::new())
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
            .add_systems(Last, create_render_queue.pipe(render).after(calc_rects))
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn create_render_queue(
    nodes : Query<GeneralNodeQuery>,
) -> VecDeque<Entity> {
    
    fn create_render_queue(node : Entity, nodes : &Query<GeneralNodeQuery>, render_queue : &mut VecDeque<Entity>) {
        let Ok(node) = nodes.get(node) else { return };
        render_queue.push_back(node.entity);
        let Some(children) = node.children else { return };
        for child in children.iter() {
            create_render_queue(child, nodes, render_queue);
        }
    }
    
    let roots = find_roots(&nodes);
    
    // let families = roots.iter().map(|e| {
    //     let mut members = vec![e];
    //     members.extend_from_iter(leaves.an(e));
    //     members
    // }).collect::<Vec<_>>();
    
    // for family in families {
    //     for node in family {
    //         let (node, global_rect, style) = nodes.get(node).unwrap();
    //         let style = style.cloned().unwrap_or_default();
    //         node.render(&global_rect.0, terminal.current_buffer_mut(), style.0);
    //     }
    // }
    
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
    fn render(&self, area: &ratatui::layout::Rect, buf: &mut Buffer, style : ratatui::style::Style);
    
    fn parse(parent : &mut EntityCommands, name : &OwnedName, attributes: &Vec<OwnedAttribute>, namespace : &Namespace) -> Option<Entity>;
}

#[derive(Component)]
pub struct NullComponent;

impl Node for NullComponent {
    fn render(&self, _area: &ratatui::layout::Rect, _buf: &mut Buffer, _style : ratatui::style::Style) {}

    fn parse(_parent : &mut EntityCommands, _name : &OwnedName, _attributes: &Vec<OwnedAttribute>, _namespace : &Namespace) -> Option<Entity> { None }
}

//==============================================================================================
//        NodeRenderer
//==============================================================================================

#[derive(SystemParam)]
pub struct NodeRenderer<'w, 's> {
    terminal : ResMut<'w, Terminal>,
    block_components : Query<'w, 's, NodeQuery<Block>>
}

impl <'w, 's> NodeRenderer<'w, 's> {
    
    pub fn render(&mut self, node : Entity) {
        if let Ok(block) = self.block_components.get(node) {
            block.component.render(&block.global_rect.0, self.terminal.current_buffer_mut(), block.style.cloned().unwrap_or_default().0);
        }
    }
    
}