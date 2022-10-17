use bevy::{prelude::*, render::texture::ImageSettings};
use components::ScoreTextComponent;
use resources::ScoreResource;
use snake_plugin::{
    events::{FoodAteEvent, GameOverEvent},
    SnakePlugin,
};

mod configuration;
pub mod snake;
mod snake_plugin;

pub struct MainPlugin;
impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        let window_description = app.world.resource::<WindowDescriptor>();

        let text_height = 30.;

        let padding = 5.;
        // calculate the region where to put the game
        let top = window_description.height / 2. - text_height;
        let bottom = -window_description.height / 2. + padding;
        let right = window_description.width / 2. - padding;
        let left = -window_description.width / 2. + padding;

        app.insert_resource(ImageSettings::default_nearest())
            .insert_resource(ScoreResource { score: 0 })
            .add_startup_system(setup(text_height))
            .add_system(show_game_over_splash)
            .add_system(increment_score)
            .add_plugin(SnakePlugin {
                rect: UiRect {
                    top,
                    left,
                    right,
                    bottom,
                },
                level: include_str!("../levels/1.level"),
            });
    }
}

fn show_game_over_splash(
    mut commands: Commands,
    mut game_over_event_reader: EventReader<GameOverEvent>,
    asset_server: Res<AssetServer>,
    score: Res<ScoreResource>,
    score_component_query: Query<Entity, With<ScoreTextComponent>>,
) {
    if game_over_event_reader.iter().count() == 0 {
        return;
    }

    warn!("Game over!");

    commands.entity(score_component_query.single()).despawn();

    spawn_game_over_screen(commands, asset_server, score.score);
}

fn increment_score(
    mut game_over_event_reader: EventReader<FoodAteEvent>,
    mut score: ResMut<ScoreResource>,
    mut score_component_query: Query<&mut Text, With<ScoreTextComponent>>,
) {
    let count = game_over_event_reader.iter().count();
    if count == 0 {
        return;
    }

    score.score += count;

    let mut score_text = score_component_query
        .get_single_mut()
        .expect("Score text should exist");
    score_text.sections[0].value = format!("Score: {}", score.score);
}

fn setup(text_height: f32) -> impl Fn(Commands, Res<AssetServer>, Res<ScoreResource>) {
    move |mut commands: Commands, asset_server: Res<AssetServer>, score: Res<ScoreResource>| {
        let font = asset_server.load("RobotoMedium-Owv4.ttf");

        // set up the camera
        let camera = Camera2dBundle::default();
        commands.spawn_bundle(camera);

        let distance_from_border_top = 5.;
        let distance_from_bottom = 5.;
        let font_size = text_height - distance_from_border_top - distance_from_bottom;

        let text_style = TextStyle {
            font,
            font_size,
            color: Color::WHITE,
        };

        commands
            .spawn_bundle(
                TextBundle::from_section(format!("Score: {}", score.score), text_style)
                    .with_text_alignment(TextAlignment::TOP_LEFT)
                    .with_style(Style {
                        align_self: AlignSelf::FlexEnd,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            top: Val::Px(distance_from_border_top),
                            left: Val::Px(15.0),
                            ..default()
                        },
                        ..default()
                    }),
            )
            .insert(ScoreTextComponent);
    }
}

fn spawn_game_over_screen(mut commands: Commands, asset_server: Res<AssetServer>, score: usize) {
    let box_size = Vec2::new(300.0, 300.0);
    let box_position = Vec2::new(0.0, 0.0);

    let font = asset_server.load("RobotoMedium-Owv4.ttf");
    let text_style = TextStyle {
        font,
        font_size: 30.0,
        color: Color::WHITE,
    };

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgba(0.0, 0.0, 0.0, 0.975),
            custom_size: Some(Vec2::new(box_size.x, box_size.y)),
            ..default()
        },
        transform: Transform::from_translation(box_position.extend(5.0)),
        ..default()
    });

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        })
        .with_children(|parent| {
            // left vertical fill (border)
            parent.spawn_bundle(
                TextBundle::from_section(
                    format!("Game over!\nScore: {}", score),
                    text_style.clone(),
                )
                .with_text_alignment(TextAlignment::CENTER)
                .with_style(Style {
                    align_self: AlignSelf::Center,
                    ..default()
                }),
            );
        });
}

mod components {
    use bevy::prelude::Component;

    #[derive(Component)]
    pub struct ScoreTextComponent;
}

mod resources {
    pub struct ScoreResource {
        pub score: usize,
    }
}

#[cfg(test)]
mod tests {
    use bevy::{input::ButtonState, prelude::*, time::TimePlugin};

    use crate::{
        snake::{Position, SnakeGame},
        snake_plugin::events::GameTick,
        MainPlugin,
    };

    #[test]
    fn test_e2e() {
        let mut app = create_app();

        release_keyboard_key(&mut app, KeyCode::Up);
        run(&mut app);

        let world = &mut app.world;
        let game = world.resource::<SnakeGame>();
        let snapshot = game.snapshot();
        assert_eq!(
            snapshot.snake,
            vec![Position::new(2, 3), Position::new(2, 2)]
        );

        // Without pressing new key, the old one is recorded
        // release_keyboard_key(&mut app, KeyCode::Up);
        run(&mut app);

        let world = &mut app.world;
        let game = world.resource::<SnakeGame>();
        let snapshot = game.snapshot();
        assert_eq!(
            snapshot.snake,
            vec![Position::new(2, 4), Position::new(2, 3)]
        );

        info!("go to right!");

        release_keyboard_key(&mut app, KeyCode::Right);
        run(&mut app);
        run(&mut app);

        let world = &mut app.world;
        let game = world.resource::<SnakeGame>();
        let snapshot = game.snapshot();
        assert_eq!(
            snapshot.snake,
            vec![
                Position::new(4, 4),
                Position::new(3, 4),
                Position::new(2, 4)
            ]
        );
    }

    pub fn run(app: &mut App) {
        app.update();
        app.update();
        app.update();

        let world = &mut app.world;
        let mut game_tick = world.get_resource_mut::<Events<GameTick>>().unwrap();

        game_tick.send(GameTick);

        app.update();
        app.update();
        app.update();
    }

    pub fn release_keyboard_key(app: &mut App, code: KeyCode) {
        use bevy::input::keyboard::KeyboardInput;

        let world = &mut app.world;
        let mut keyboard_input = world.get_resource_mut::<Events<KeyboardInput>>().unwrap();
        keyboard_input.send(KeyboardInput {
            scan_code: 0,
            key_code: Some(code),
            state: ButtonState::Released,
        });
    }

    pub fn create_app() -> App {
        use bevy::{
            asset::AssetPlugin,
            core::CorePlugin,
            core_pipeline::CorePipelinePlugin,
            input::InputPlugin,
            log::LogSettings,
            pbr::PbrPlugin,
            render::{camera::RenderTarget, RenderPlugin},
            scene::ScenePlugin,
            sprite::SpritePlugin,
            text::TextPlugin,
            ui::UiPlugin,
            utils::tracing::subscriber::set_global_default,
            window::{WindowId, WindowPlugin},
        };
        use tracing_log::LogTracer;
        use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter};

        if LogTracer::init().is_ok() {
            let filter_layer = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("OFF,bevy_snake=INFO"))
                .unwrap();
            let subscriber = Registry::default().with(filter_layer);
            let fmt_layer = tracing_subscriber::fmt::Layer::default();
            let subscriber = subscriber.with(fmt_layer);
            set_global_default(subscriber).unwrap();
        }

        let mut app = App::new();

        app.world.clear_entities();
        app.world.clear_trackers();

        let mut log_settings = LogSettings::default();
        log_settings.filter = format!("{},bevy_mod_raycast=off", log_settings.filter);
        app.insert_resource(log_settings);

        app.insert_resource(WindowDescriptor {
            title: "Snake at Codemotion!".to_string(),
            width: 300.,
            height: 300.,
            resizable: false,
            cursor_visible: false,
            ..default()
        });

        app.add_plugin(CorePlugin::default());
        app.add_plugin(TimePlugin::default());
        app.add_plugin(TransformPlugin::default());
        app.add_plugin(HierarchyPlugin::default());
        app.add_plugin(InputPlugin::default());
        app.add_plugin(WindowPlugin {
            // add_primary_window: true,
            // exit_on_close: false,
        });
        app.add_plugin(AssetPlugin::default());

        app.add_plugin(ScenePlugin::default());

        app.add_plugin(RenderPlugin::default());
        app.add_plugin(CorePipelinePlugin::default());
        app.add_plugin(SpritePlugin::default());
        app.add_plugin(TextPlugin::default());
        app.add_plugin(UiPlugin::default());
        app.add_plugin(PbrPlugin::default());

        {
            let camera = Camera {
                target: RenderTarget::Window(WindowId::primary()),
                ..Camera::default()
            };
            app.insert_resource(camera);
        }

        app.add_plugin(MainPlugin);

        app
    }
}
