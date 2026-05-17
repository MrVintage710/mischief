use bevy::prelude::*;
use mischeif::{MischiefPlugin, component::TerminalComponent, node::block::Block};

pub fn main() {
    let mut app = App::new();

    app.add_plugins(MischiefPlugin::new());

    app.add_systems(Startup, init);
    
    app.run();
}

pub fn init(
    mut commands : Commands,
    asset_server : Res<AssetServer>
) {
    commands.spawn((
        TerminalComponent(asset_server.load("simple.xml")),
    ));

    // commands.spawn((
    //     Block::default().title("title"),
    //     mischeif::layout::Rect::default(),
    // ));
}