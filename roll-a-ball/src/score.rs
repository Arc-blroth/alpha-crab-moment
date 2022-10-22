use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Score(u32);

#[derive(Component)]
pub struct ScoreText;

pub fn setup(mut commands: Commands, assets: ResMut<AssetServer>) {
    commands.spawn().insert(Score::default());

    commands
        .spawn_bundle(
            TextBundle::from_section(
                "Score: 0\nTime: 0",
                TextStyle {
                    font: assets.load("LiberationSans-Bold.ttf"),
                    font_size: 14.0,
                    color: Color::WHITE,
                },
            )
            .with_text_alignment(TextAlignment::TOP_RIGHT)
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(ScoreText);
}

pub fn update_score(mut score_text: Query<&mut Text, With<ScoreText>>, time: Res<Time>) {
    score_text.single_mut().sections[0].value = format!("Score: {}\nTime: {:.0}", 0, time.seconds_since_startup());
}
