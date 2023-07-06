use bevy::{
    app::AppExit,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_turborand::prelude::*;
use bevy_vector_shapes::prelude::*;
const PLAYER_SIZE: f32 = 10f32;

const PLAYER_THRUST: f32 = 5.0;
const PLAYER_TURN_SPEED: f32 = std::f32::consts::PI / 24.0;

const PLAYER_SHOOT_DELAY: f32 = 0.5;

const PLAYER_CLEAR_RADIUS: f32 = 100f32;

const BULLET_SPEED: f32 = 300.0;
const BULLET_LIFETIME: f32 = 3.0;
const BULLET_RADIUS: f32 = 1.0;

const ASTEROID_SPEED: f32 = 50.0;
const ASTEROID_RADIUS: f32 = 20.0;
const ASTEROID_COUNT: usize = 10;

#[derive(Component, Debug, Default, PartialEq)]
struct Velocity(Vec2);

#[derive(Component)]
struct Person;

#[derive(Component, Debug, Eq, PartialEq)]
struct Name(String);

#[derive(Component, Debug)]
struct LimitedLifetime {
    timer: Timer,
}

#[derive(Component, Debug)]
struct Bullet;

#[derive(Bundle)]
struct BulletBundle {
    bullet: Bullet,
    position: TransformBundle,
    velocity: Velocity,
    limited_lifetime: LimitedLifetime,
}

#[derive(Component, Debug, Default)]
struct Ship {
    angle: f32,
    thrusting: bool,
    shoot_requested: bool,
    shoot_timer: Timer,
}

#[derive(Bundle)]
struct PlayerBundle {
    name: Name,
    ship: Ship,
    position: TransformBundle,
    velocity: Velocity,
}

#[derive(Component, Debug)]
struct Asteroid {
    radius: f32,
}

#[derive(Bundle)]
struct AsteroidBundle {
    position: TransformBundle,
    velocity: Velocity,
    size: Asteroid,
}

#[derive(Component, Debug, Default)]
struct Game {
    // TODO render score in UI somewhere
    score: i32,
}

#[derive(Bundle)]
struct GameBundle {
    game: Game,
    rng: RngComponent,
}

#[derive(Component)]
struct FpsText;

#[derive(Component, Default)]
struct ScoreText;

impl ScoreText {
    fn new() -> Self {
        Self
    }
}

#[derive(Resource)]
struct GreetTimer(Timer);

fn setup(
    mut commands: Commands,
    mut windows: Query<&mut Window>,
    mut global_rng: ResMut<GlobalRng>,
) {
    let mut rng = RngComponent::from(&mut global_rng);

    let mut window = windows.get_single_mut().unwrap();
    window.title = "Asteroids".to_string();

    let width = window.physical_width();
    let height = window.physical_height();
    println!("width: {}", width);
    println!("height: {}", height);

    commands.spawn(Camera2dBundle::default());

    let player_position = Vec3::new(0.0, 0.0, 0.0);
    commands.spawn(PlayerBundle {
        name: Name("Player".to_string()),
        position: TransformBundle::from_transform(Transform::from_translation(player_position)),
        velocity: Velocity(Vec2::new(5.0, 10.0)),
        ship: Ship::default(),
    });

    for _ in 0..ASTEROID_COUNT {
        let position = loop {
            let pos = Vec3::new(
                rng.f32() * width as f32 - width as f32 / 2.0,
                rng.f32() * height as f32 - height as f32 / 2.0,
                0.0,
            );
            if pos.distance(player_position) - ASTEROID_RADIUS > PLAYER_CLEAR_RADIUS {
                break pos;
            }
        };

        let angle = rng.f32() * std::f32::consts::PI * 2.0;
        commands.spawn(AsteroidBundle {
            position: TransformBundle::from_transform(Transform::from_translation(position)),
            velocity: Velocity(Vec2::new(
                angle.cos() * ASTEROID_SPEED,
                angle.sin() * ASTEROID_SPEED,
            )),
            size: Asteroid {
                radius: ASTEROID_RADIUS,
            },
        });
    }

    commands.spawn(GameBundle {
        game: Game::default(),
        rng,
    });
}

fn ui_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let standard_style = TextStyle {
        font: font.clone(),
        font_size: 32.0,
        color: Color::WHITE,
    };

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new("Score:", standard_style.clone()),
            TextSection::new("0", standard_style.clone()),
        ]),
        ScoreText::new(),
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: font.clone(),
                font_size: 32.0,
                color: Color::GOLD,
            }),
        ])
        .with_text_alignment(TextAlignment::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(15.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        FpsText,
    ));
}

fn quit_on_escape(keyboard_input: Res<Input<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keyboard_input.pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

fn handle_input(keyboard_input: Res<Input<KeyCode>>, mut query: Query<(&mut Ship, &mut Velocity)>) {
    for (ref mut ship, mut velocity) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            ship.angle += PLAYER_TURN_SPEED;
        }
        if keyboard_input.pressed(KeyCode::D) {
            ship.angle -= PLAYER_TURN_SPEED;
        }
        if keyboard_input.pressed(KeyCode::W) {
            velocity.0 += Vec2::new(ship.angle.cos(), ship.angle.sin()) * PLAYER_THRUST;
            ship.thrusting = true;
        } else {
            ship.thrusting = false;
        }
        ship.shoot_requested =
            keyboard_input.pressed(KeyCode::Space) || keyboard_input.pressed(KeyCode::S);
    }
}

fn draw_player(query: Query<(&Ship, &GlobalTransform)>, mut painter: ShapePainter) {
    for (player, position) in &mut query.iter() {
        painter.set_translation(position.translation());
        painter.color = Color::WHITE;
        painter.disable_laa = true;
        painter.thickness_type = ThicknessType::Pixels;
        painter.thickness = 2.0;

        let line_length = 15.0;
        painter.line(
            Vec3::ZERO,
            Vec3::new(player.angle.cos(), player.angle.sin(), 0.0) * line_length,
        );

        let p1 = Vec3::new(
            player.angle.cos() * PLAYER_SIZE,
            player.angle.sin() * PLAYER_SIZE,
            0.0,
        );
        let p2 = Vec3::new(
            (player.angle + std::f32::consts::PI / 4.0 + std::f32::consts::PI).cos() * PLAYER_SIZE,
            (player.angle + std::f32::consts::PI / 4.0 + std::f32::consts::PI).sin() * PLAYER_SIZE,
            0.0,
        );
        let p3 = Vec3::new(
            (player.angle - std::f32::consts::PI / 4.0 + std::f32::consts::PI).cos() * PLAYER_SIZE,
            (player.angle - std::f32::consts::PI / 4.0 + std::f32::consts::PI).sin() * PLAYER_SIZE,
            0.0,
        );
        painter.line(p1, p2);
        painter.line(p2, p3);
        painter.line(p3, p1);
    }
}

fn draw_bullets(query: Query<(&Bullet, &GlobalTransform)>, mut painter: ShapePainter) {
    for (_, position) in &mut query.iter() {
        painter.set_translation(position.translation());
        painter.color = Color::WHITE;

        painter.circle(BULLET_RADIUS);
    }
}

fn draw_asteroids(query: Query<(&Asteroid, &GlobalTransform)>, mut painter: ShapePainter) {
    for (asteroid, position) in &mut query.iter() {
        painter.set_translation(position.translation());
        painter.color = Color::WHITE;

        painter.circle(asteroid.radius);
    }
}

fn despawn_timed_out_entities(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut LimitedLifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn shoot(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Ship, &GlobalTransform, &Velocity)>,
) {
    for (mut ship, transform, velocity) in query.iter_mut() {
        ship.shoot_timer.tick(time.delta());
        if ship.shoot_requested {
            if ship.shoot_timer.finished() {
                ship.shoot_timer = Timer::from_seconds(PLAYER_SHOOT_DELAY, TimerMode::Once);
                ship.shoot_timer.reset();

                commands.spawn(BulletBundle {
                    bullet: Bullet,
                    position: TransformBundle::from_transform(
                        transform.compute_transform().clone(),
                    ),
                    velocity: Velocity(
                        velocity.0
                            + Vec2::new(
                                BULLET_SPEED * ship.angle.cos(),
                                BULLET_SPEED * ship.angle.sin(),
                            ),
                    ),
                    limited_lifetime: LimitedLifetime {
                        timer: Timer::from_seconds(BULLET_LIFETIME, TimerMode::Once),
                    },
                });
            }
        }
    }
}

fn move_objects(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
    windows: Query<&Window>,
) {
    let window = windows.get_single().unwrap();
    let (half_width, half_height) = (window.width() / 2.0, window.height() / 2.0);

    for (ref mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();

        if transform.translation.x < -half_width || transform.translation.x > half_width {
            transform.translation.x *= -1.0;
        }
        if transform.translation.y < -half_height || transform.translation.y > half_height {
            transform.translation.y *= -1.0;
        }
    }
}

fn check_collisions(
    mut commands: Commands,
    mut game: Query<&mut Game>,
    mut asteroid_query: Query<(Entity, &Transform, &Asteroid)>,
    mut bullet_query: Query<(Entity, &Transform, &Bullet)>,
    mut ship_query: Query<(Entity, &Transform, &Ship)>,
) {
    let mut game = game.single_mut();
    for (asteroid_entity, asteroid_transform, asteroid) in asteroid_query.iter_mut() {
        for (bullet_entity, bullet_transform, _bullet) in bullet_query.iter_mut() {
            if asteroid_transform
                .translation
                .distance(bullet_transform.translation)
                < asteroid.radius + BULLET_RADIUS
            {
                commands.entity(asteroid_entity).despawn();
                commands.entity(bullet_entity).despawn();
                game.score += 1;
            }
        }

        for (ship_entity, ship_transform, _ship) in ship_query.iter_mut() {
            if asteroid_transform
                .translation
                .distance(ship_transform.translation)
                < asteroid.radius + PLAYER_SIZE
            {
                // TODO end game and restart it
                commands.entity(asteroid_entity).despawn();
                commands.entity(ship_entity).despawn();
                game.score -= 1;
            }
        }
    }
}

fn update_fps_text(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}

fn update_score_text(
    time: Res<Time>,
    mut text_query: Query<&mut Text, With<ScoreText>>,
    game: Query<&Game>,
) {
    for mut text in &mut text_query {
        let seconds = time.elapsed_seconds();

        text.sections[1].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
        text.sections[1].value = format!("{}", game.single().score);
    }
}

pub struct AsteroidsPlugin;

impl Plugin for AsteroidsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_plugin(RngPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_startup_system(setup)
            .add_startup_system(ui_setup)
            .add_system(handle_input)
            .add_system(update_fps_text)
            .add_system(update_score_text)
            .add_system(quit_on_escape)
            .add_system(move_objects)
            .add_system(despawn_timed_out_entities)
            .add_system(shoot)
            .add_system(check_collisions)
            .add_system(draw_bullets)
            .add_system(draw_asteroids)
            .add_system(draw_player);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Shape2dPlugin::default())
        .add_plugin(AsteroidsPlugin)
        .run();
}
