use bevy::{ecs::query::QueryData, platform::collections::{Equivalent, HashSet}, prelude::*};

use crate::{layout::{GlobalRect, Layout, Padding, RectState}, node::Node, style::Style};

//==============================================================================================
//        NodeQuery
//==============================================================================================

#[derive(QueryData)]
pub struct NodeQuery {
    pub entity : Entity,
    pub global_rect: &'static GlobalRect,
    pub rect : &'static crate::layout::Rect,
    pub rect_state : &'static RectState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub layout : &'static Layout,
    pub padding : Option<&'static Padding>,
    pub style : Option<&'static Style>,
    pub id : Option<&'static Name>
}

impl <'w, 's> NodeQueryItem<'w, 's> {
    pub fn has_id(&self, id : &str) -> bool {
        let Some(name) = self.id else {return false };
        name.as_str() == id
    }
    
    pub fn parent(&self) -> Option<Entity> {
        self.parent.map(|p| p.0)
    }
    
    pub fn children(&self) -> Option<&Children> {
        self.children
    }
}


//==============================================================================================
//        NodeQueryMut
//==============================================================================================

#[derive(QueryData)]
#[query_data(mutable)]
pub struct NodeQueryMut {
    pub entity : Entity,
    pub global_rect: &'static mut GlobalRect,
    pub rect : &'static mut crate::layout::Rect,
    pub rect_state : &'static mut RectState,
    pub parent : Option<&'static ChildOf>,
    pub children : Option<&'static Children>,
    pub layout : &'static mut Layout,
    pub padding : Option<&'static mut Padding>,
    pub style : Option<&'static mut Style>,
    pub id : Option<&'static Name>
}

impl <'w, 's> NodeQueryMutReadOnlyItem<'w, 's> {
    pub fn has_id(&self, id : &str) -> bool {
        let Some(name) = self.id else {return false };
        name.as_str() == id
    }
    
    pub fn parent(&self) -> Option<Entity> {
        self.parent.map(|p| p.0)
    }
    
    pub fn children(&self) -> Option<&Children> {
        self.children
    }
}

impl <'w, 's> NodeQueryMutItem<'w, 's> {
    pub fn has_id(&self, id : &str) -> bool {
        let Some(name) = self.id else {return false };
        name.as_str() == id
    }
    
    pub fn parent(&self) -> Option<Entity> {
        self.parent.map(|p| p.0)
    }
    
    pub fn children(&self) -> Option<&Children> {
        self.children
    }
}

//==============================================================================================
//        NodeFindRootsAbility
//==============================================================================================

pub trait NodeFindRootsAbility : Sized {
    fn find_roots(&self) -> Vec<Entity>;
}

impl <'w, 's> NodeFindRootsAbility for Query<'w, 's, NodeQuery> {
    fn find_roots(&self) -> Vec<Entity>  {
        fn find_roots_r(current : Entity, query : &Query<'_, '_, NodeQuery>, visited : &mut HashSet<Entity>, roots : &mut Vec<Entity>) {
            if visited.contains(&current) { return }
            let Ok(current_item) = query.get(current) else { return };
            let Some(parent) = current_item.parent else {
                roots.push(current);
                return;
            };
            let Ok(parent) = query.get(parent.0) else {
                roots.push(current);
                return;
            };
            find_roots_r(parent.entity, query, visited, roots);
        }
        
        let mut roots = Vec::new();
        let mut visited = HashSet::<Entity>::new();
        
        for node in self.iter() {
            find_roots_r(node.entity, self, &mut visited, &mut roots);
        }
        
        roots
    }
}

impl <'w, 's> NodeFindRootsAbility for Query<'w, 's, NodeQueryMut> {
    fn find_roots(&self) -> Vec<Entity>  {
        fn find_roots_r(current : Entity, query : &Query<'_, '_, NodeQueryMut>, visited : &mut HashSet<Entity>, roots : &mut Vec<Entity>) {
            if visited.contains(&current) { return }
            let Ok(current_item) = query.get(current) else { return };
            let Some(parent) = current_item.parent else {
                roots.push(current);
                return;
            };
            let Ok(parent) = query.get(parent.0) else {
                roots.push(current);
                return;
            };
            find_roots_r(parent.entity, query, visited, roots);
        }
        
        let mut roots = Vec::new();
        let mut visited = HashSet::<Entity>::new();
        
        for node in self.iter() {
            find_roots_r(node.entity, self, &mut visited, &mut roots);
        }
        
        roots
    }
}

//==============================================================================================
//        NodeFindSiblingsAbility
//==============================================================================================

pub trait NodeFindSiblingAbility : Sized {
    fn find_siblings(&self, entity : Entity) -> Vec<Entity>;
    
    fn sibling_index(&self, entity : Entity) -> usize;
}

impl <'w, 's> NodeFindSiblingAbility for Query<'w, 's, NodeQuery> {
    fn find_siblings(&self, entity : Entity) -> Vec<Entity> {
        let Some(node) = self.get(entity).ok() else { return Vec::new() };
        let Some(parent) = node.parent() else { return Vec::new() };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return Vec::new() };
        let Some(children) = parent.children() else { return Vec::new() };
        children.iter().filter(|c| *c != entity).collect()
    }

    fn sibling_index(&self, entity : Entity) -> usize {
        let Some(node) = self.get(entity).ok() else { return 0 };
        let Some(parent) = node.parent() else { return 0 };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return 0 };
        let Some(children) = parent.children() else { return 0 };
        children.iter().position(|c| c == entity).unwrap_or_default()
    }
}

impl <'w, 's> NodeFindSiblingAbility for Query<'w, 's, NodeQueryMut> {
    fn find_siblings(&self, entity : Entity) -> Vec<Entity> {
        let Some(node) = self.get(entity).ok() else { return Vec::new() };
        let Some(parent) = node.parent() else { return Vec::new() };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return Vec::new() };
        let Some(children) = parent.children() else { return Vec::new() };
        children.iter().filter(|c| *c != entity).collect()
    }
    
    fn sibling_index(&self, entity : Entity) -> usize {
        let Some(node) = self.get(entity).ok() else { return 0 };
        let Some(parent) = node.parent() else { return 0 };
        drop(node);
        let Some(parent) = self.get(parent).ok() else { return 0 };
        let Some(children) = parent.children() else { return 0 };
        children.iter().position(|c| c == entity).unwrap_or_default()
    }
}