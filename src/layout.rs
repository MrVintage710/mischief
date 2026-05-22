use std::{collections::{HashMap, HashSet}, ops::{Deref, DerefMut}};

use bevy::prelude::*;
use taffy::{AvailableSpace, NodeId, TaffyTree};

use crate::{TerminalMessage, node::query::{NodeEntityMut, NodeFindFamilyAbility, NodeFindRootsAbility}};

//==============================================================================================
//        TerminalRenderPlugin
//==============================================================================================

pub struct TerminalLayoutPlugin;

impl Plugin for TerminalLayoutPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Last,
                (
                    resize,
                    calc_layout.run_if(|states : Query<&LayoutState>| states.iter().any(|s| s.is_dirty()))
                ).chain()
            )
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn calc_layout(
    mut nodes : Query<NodeEntityMut>,
    terminal_size: Res<TerminalSize>
) {

    pub fn fill_tree_r(current : Entity, nodes : &mut Query<NodeEntityMut>, tree : &mut TaffyTree<Entity>, lookup : &mut HashMap<Entity, NodeId>) -> Option<NodeId> {
        if let Some(node_id) = lookup.get(&current) {return Some(*node_id)}
        let node = nodes.get(current).ok()?;

        //Get taffy Styleing for style
        let style : taffy::Style = (*node.style).into();

        let children = node.children_cloned();
        let children_nodes = children.into_iter().filter_map(|child| fill_tree_r(child, nodes, tree, lookup)).collect::<Vec<_>>();

        let node_id = if children_nodes.is_empty() {
            tree.new_leaf(style).ok()?
        } else {
            tree.new_with_children(style, children_nodes.as_slice()).ok()?
        };
        
        lookup.insert(current, node_id);

        return Some(node_id)
    }
    
    let mut node_lookup = HashMap::<Entity, NodeId>::new();
    let mut tree : TaffyTree<Entity> = TaffyTree::new();
    tree.enable_rounding();

    let roots = nodes.find_roots();
    
    for node in roots.iter() {
        fill_tree_r(*node, &mut nodes, &mut tree, &mut node_lookup);
    }
    
    let mut nodes_needing_update = HashSet::new();
    
    for root in roots.iter() {
        let Ok(mut node) = nodes.get_mut(*root) else { continue };
        let Some(node_id) = node_lookup.get(&node.entity) else { continue };
        tree.compute_layout(*node_id, taffy::Size { 
            width: AvailableSpace::Definite(terminal_size.0 as f32), 
            height: AvailableSpace::Definite(terminal_size.1 as f32)
        }).unwrap();
        node.set_layout_ok();
        for child in nodes.find_family(*root) {
            nodes_needing_update.insert(child);
        }
    }
    
    for node_entity in nodes_needing_update {
        let Ok(mut node) = nodes.get_mut(node_entity) else { continue };
        let Some(node_id) = node_lookup.get(&node.entity) else { continue };
        let Ok(layout) = tree.layout(*node_id) else { continue };
        let rect = ratatui::layout::Rect { 
            x: layout.location.x as u16, 
            y: layout.location.y as u16, 
            width: layout.size.width as u16, 
            height: layout.size.height as u16 
        };
        node.rect.0 = rect;
    }
}

pub fn resize(
    mut terminal_size : ResMut<TerminalSize>,
    mut event : MessageReader<TerminalMessage>
) {
    for event in event.read() {
        let ratatui::crossterm::event::Event::Resize(width, height) = event.0 else { continue };
        *terminal_size = TerminalSize(width, height);
    }
}

//==============================================================================================
//        Rect
//==============================================================================================

#[derive(Component, Default, Clone, Copy, Debug)]
#[require(LayoutState)]
pub struct Rect(pub ratatui::layout::Rect);

impl DerefMut for Rect {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Rect {
    type Target = ratatui::layout::Rect;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

//==============================================================================================
//        NodeState
//==============================================================================================

#[derive(Component, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayoutState {
    #[default]
    Dirty,
    Ok
}

impl LayoutState {
    /// Returns `true` if the rect state is [`Dirty`].
    ///
    /// [`Dirty`]: RectState::Dirty
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        matches!(self, Self::Dirty)
    }

    /// Returns `true` if the rect state is [`Ok`].
    ///
    /// [`Ok`]: RectState::Ok
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }

    pub fn set_dirty(&mut self) {
        *self = LayoutState::Dirty;
    }

    pub fn set_ok(&mut self) {
        *self = LayoutState::Ok
    }
}

//==============================================================================================
//        TerminalSize
//==============================================================================================

#[derive(Debug, Resource)]
pub struct TerminalSize(pub u16, pub u16);