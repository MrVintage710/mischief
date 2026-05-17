use std::{collections::{HashSet, VecDeque}, io::BufReader};

use bevy::{asset::{AssetLoader, AsyncReadExt}, prelude::*};
use xml::{EventReader, ParserConfig, reader::XmlEvent};

use crate::{Terminal, node::{Node, block::Block}};

//==============================================================================================
//        TerminalComponentPlugin
//==============================================================================================

pub struct TerminalComponentPlugin;

impl Plugin for TerminalComponentPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset_loader::<TerminalComponentLoader>()
            .init_asset::<TerminalComponentDefinition>()
        
            .add_systems(Last, asset_loaded_or_changed)
        ;
    }
}

//==============================================================================================
//        Systems
//==============================================================================================

pub fn asset_loaded_or_changed(
    mut terminal : ResMut<Terminal>,
    mut commands : Commands,
    mut asset_events : MessageReader<AssetEvent<TerminalComponentDefinition>>,
    components : Query<(Entity, &TerminalComponent)>,
    component_defs : Res<Assets<TerminalComponentDefinition>>
) {
    let ids = asset_events.read().filter_map(|e| match e {
        AssetEvent::Modified { id } => Some(*id),
        AssetEvent::LoadedWithDependencies { id } => Some(*id),
        _ => None
    }).collect::<HashSet<_>>();
    
    if !ids.is_empty() { terminal.clear().unwrap() }
    
    for id in ids.iter().into_iter() {
        let Some(component_def) = component_defs.get(*id) else { continue };
        let Some(component) = components.iter().find(|c| c.1.0.id() == *id).map(|c| c.0) else { continue };
        commands.entity(component).despawn_children();
        
        let mut parent_stack = VecDeque::default();
        parent_stack.push_front(component);
        let mut ignore_depth = 0;
        
        for event in component_def.events.iter() {
            match event {
                XmlEvent::StartElement { name, attributes, namespace } => {
                    
                    let mut entity_commands = commands.entity(parent_stack.front().unwrap().clone());
                    if let Some(child) = Block::parse(&mut entity_commands, name, attributes, namespace) {
                        if let Some(name) = attributes.iter().find_map(|attr| if &attr.name.local_name == "id" { Some(attr.value.clone()) } else { None }  ) {
                            commands.entity(child).insert(Name::new(name));
                        }
                        parent_stack.push_front(child);
                        continue;
                    }
                    
                    ignore_depth += 1
                },
                XmlEvent::EndElement { .. } => { if ignore_depth == 0 { parent_stack.pop_front(); } else { ignore_depth -= 1 } },
                _ => {}
            }
        }
    }
}

//==============================================================================================
//        TerminalComponent
//==============================================================================================

#[derive(Component)]
pub struct TerminalComponent(pub Handle<TerminalComponentDefinition>);

//==============================================================================================
//        TerminalComponent
//==============================================================================================

#[derive(Debug, Asset, TypePath)]
pub struct TerminalComponentDefinition {
    events : Vec<XmlEvent>
}

//==============================================================================================
//        TerminalComponentLoader
//==============================================================================================

#[derive(Debug, TypePath, Default)]
pub struct TerminalComponentLoader;

impl AssetLoader for TerminalComponentLoader {
    type Asset = TerminalComponentDefinition;

    type Settings = ();

    type Error = xml::reader::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        
        let mut string = String::new();
        reader.read_to_string(&mut string).await?;
        
        let xml_reader = EventReader::new_with_config(
            BufReader::new(string.as_bytes()), 
            ParserConfig::new()
                .allow_multiple_root_elements(true)
                .trim_whitespace(true)
        );
        let mut events = Vec::new();
        for event in xml_reader.into_iter() {
            events.push(event?)
        }
        
        Ok(TerminalComponentDefinition { events })
    }
    
    fn extensions(&self) -> &[&str] {
        &["xml"]
    }
}