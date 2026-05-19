use std::{collections::HashMap, marker::PhantomData};

use bevy::{prelude::*};
use ratatui::{widgets::Widget};
use taffy::{LengthPercentage, LengthPercentageAuto, NodeId, Style, TaffyTree};

use crate::{TerminalMessage, node::query::{NodeEntityMut, NodeFindLeavesAbility, NodeFindSiblingAbility}};

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
                    calc_rects.run_if(|states : Query<&LayoutState>| states.iter().any(|s| s.is_dirty()))
                ).chain()
            )
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn calc_rects(
    mut nodes : Query<NodeEntityMut>,
    terminal_size: Res<TerminalSize>
) {
    fn calc_layout(entity : Entity, nodes : &mut Query<NodeEntityMut>, terminal_size : &TerminalSize) -> (GlobalRect, Layout, Padding) {
        
        // Get the global bounds just in case
        let global_bounds = {
            GlobalRect(ratatui::layout::Rect { x: 0, y: 0, width: terminal_size.0, height: terminal_size.1 })
        };
        
        let Some(current) = nodes.get(entity).ok() else { return (global_bounds, Layout::Relative, Padding::default()) };
        
        if current.rect_state.is_ok() { return (current.global_rect.clone(), *current.layout, current.padding.cloned().unwrap_or_default()) }
        
        let parent = current.parent.cloned().unwrap_or(ChildOf(Entity::PLACEHOLDER));
        let (parent_bounds, parent_layout, parent_padding) = calc_layout(parent.0, nodes, terminal_size);
        
        let bounds = ratatui::layout::Rect {
            x: parent_bounds.0.x + parent_padding.left,
            y: parent_bounds.0.y + parent_padding.top,
            width: parent_bounds.0.width.saturating_sub(parent_padding.right.saturating_add(parent_padding.left)),
            height: parent_bounds.0.height.saturating_sub(parent_padding.bottom.saturating_add(parent_padding.top)),
        };
        
        let siblings = nodes.find_siblings(entity);
        let sibling_index = nodes.sibling_index(entity);
        let mut child = nodes.get_mut(entity).unwrap();
        
        match parent_layout {
            Layout::Relative => {
                
                child.global_rect.0 = ratatui::layout::Rect {
                    x: bounds.x + child.rect.x.get_value(bounds.width), 
                    y: bounds.y + child.rect.y.get_value(bounds.height), 
                    width: child.rect.width.get_value(bounds.width), 
                    height: child.rect.height.get_value(bounds.height)
                };
                
                *child.rect_state = LayoutState::Ok;
            },
            Layout::Flex(flex_options) => {
                let FlexOptions { gap, direction } = flex_options;
                match direction {
                    Direction::Vertical => {
                        
                    },
                    Direction::Horizontal => todo!(),
                }
            },
        }
        
        (*child.global_rect, *child.layout, child.padding.as_deref().cloned().unwrap_or_default())
    }
    
    let entities = nodes.iter().map(|node| node.entity).collect::<Vec<_>>();
    
    for e in entities {
        calc_layout(e, &mut nodes, terminal_size.as_ref());
    }
}

pub fn calc_layout(
    mut nodes : Query<NodeEntityMut>,
    terminal_size: Res<TerminalSize>
) {

    pub fn fill_tree_r(current : Entity, nodes : &mut Query<NodeEntityMut>, tree : &mut TaffyTree<Entity>, lookup : &mut HashMap<Entity, NodeId>) -> Option<NodeId> {
        if let Some(node_id) = lookup.get(&current) {return Some(*node_id)}
        let node = nodes.get(current).ok()?;
        
        let padding = node.padding.cloned().unwrap_or_default().into();
        // let inset = node.rect

        let children = node.children_cloned();
        let childrend_nodes = children.into_iter().filter_map(|child| fill_tree_r(child, nodes, tree, lookup)).collect::<Vec<_>>();

        let node_id = tree.new_with_children(Style {
            padding,
            // inset,
            
            ..default()
        }, childrend_nodes.as_slice()).ok()?;
        lookup.insert(current, node_id);

        return Some(node_id)
    }
    
    let mut node_lookup = HashMap::<Entity, NodeId>::new();
    let mut tree : TaffyTree<Entity> = TaffyTree::new();
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
//        WidgetRenderer
//==============================================================================================

#[derive(Component)]
pub struct WidgetRenderer<W : Widget>(PhantomData<W>);

impl <W : Widget> Default for WidgetRenderer<W> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl <W : Widget> From<W> for WidgetRenderer<W> {
    fn from(_: W) -> Self {
        WidgetRenderer::<W>::default()
    }
}

//==============================================================================================
//        Rects
//==============================================================================================

#[derive(Component, Clone, Copy, Debug)]
#[require(GlobalRect, Layout, LayoutState)]
pub struct Rect {
    pub x : Value,
    pub y : Value,
    pub width : Value,
    pub height : Value
}

impl Default for Rect {
    fn default() -> Self {
        Self { 
            x: Value::Px(0), 
            y: Value::Px(0), 
            width: Value::Percent(1.0), 
            height: Value::Percent(1.0) 
        }
    }
}

// impl Into<taffy::Rect<LengthPercentageAuto>> for Rect {
//     fn into(self) -> taffy::Rect<LengthPercentageAuto> {
//         taffy::Rect { left: LengthPercentageAuto::, right: (), top: (), bottom: () }
//     }
// }

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct GlobalRect(pub ratatui::layout::Rect);

//==============================================================================================
//        Layout
//==============================================================================================

#[derive(Component, Clone, Copy, Debug)]
pub enum Layout {
    Relative,
    Flex(FlexOptions),
}

impl Default for Layout {
    fn default() -> Self {
        Layout::Relative
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FlexOptions {
    gap : Value,
    direction : Direction,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal
}

//==============================================================================================
//        Value
//==============================================================================================

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Px(u16),
    Percent(f32),
}

impl Value {
    
    pub fn get_value(&self, parent_dimention : u16) -> u16 {
        match self {
            Value::Px(value) => *value,
            Value::Percent(percent) => (parent_dimention as f32 * (*percent)).floor() as u16,
        }
    }
    
    pub fn custom_calc(&self, callback : impl FnOnce(f32) -> u16 ) -> u16 {
        match self {
            Value::Px(value) => *value,
            Value::Percent(percent) => callback(*percent),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Self::Px(0)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Value::Px(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Percent(value)
    }
}

impl Into<LengthPercentage> for Value {
    fn into(self) -> LengthPercentage {
        match self {
            Value::Px(val) => LengthPercentage::length(val as f32),
            Value::Percent(val) => LengthPercentage::percent(val),
        }
    }
}

//==============================================================================================
//        Padding
//==============================================================================================

#[derive(Default, Component, Clone, Copy, Debug)]
pub struct Padding {
    pub left : Value,
    pub right : Value,
    pub top : Value,
    pub bottom : Value
}

impl Padding {
    pub fn new(left: impl Into<Value>, right: impl Into<Value>, top: impl Into<Value>, bottom: impl Into<Value>) -> Self {
        Self { left : left.into(), right: right.into(), top: top.into(), bottom: bottom.into() }
    }
}

impl Into<taffy::Rect<LengthPercentage>> for Padding {
    fn into(self) -> taffy::Rect<LengthPercentage> {
        taffy::Rect { left: self.left.into(), right: self.right.into(), top: self.top.into(), bottom: self.bottom.into() }
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
}

//==============================================================================================
//        TerminalSize
//==============================================================================================

#[derive(Debug, Resource)]
pub struct TerminalSize(pub u16, pub u16);